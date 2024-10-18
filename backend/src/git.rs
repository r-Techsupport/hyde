//! Abstractions and interfaces over the git repository

use color_eyre::eyre::{bail, ContextCompat, WrapErr, Result};
use git2::{AnnotatedCommit, FetchOptions, Oid, Repository, Signature, Status, BranchType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct Interface {
    repo: Arc<Mutex<Repository>>,
    /// The path to the documents folder, relative to the server executable.
    /// 
    /// EG: `./repo/docs`
    doc_path: PathBuf,
}

/// This is used for `get_doc_tree`
#[derive(Debug, Deserialize, Serialize)]
pub struct INode {
    name: String,
    children: Vec<INode>,
}

impl Interface {
    /// Clone the repository into `./repo`, or run `fetch` if an existing repo
    /// was detected
    ///
    /// # Errors
    /// This function will return an error if any of the git initialization steps fail, or if
    /// the required environment variables are not set.
    pub fn new() -> Result<Self> {
        let mut doc_path = PathBuf::from("repo");
        doc_path.push(env::var("DOC_PATH").unwrap_or_else(|_| {
            warn!("The `DOC_PATH` environment variable was not set, defaulting to `docs/`");
            "docs".to_string()
        }));
        let repo = Self::load_repository("repo")?;
        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            doc_path,
        })
    }

    /// Return the document from the provided `path`, where `path` is the
    /// path to the markdown file relative to the root of the documents folder.
    ///
    /// The return type is a little bit messy, but I needed to differentiate between
    /// "file not found", and "failed to read file"
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail.
    #[tracing::instrument(skip(self))]
    pub fn get_doc<P: AsRef<Path> + std::fmt::Debug>(&self, path: P) -> Result<Option<String>> {
        let mut path_to_doc: PathBuf = PathBuf::from(".");
        path_to_doc.push(&self.doc_path);
        path_to_doc.push(path);
        if !path_to_doc.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(path_to_doc)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(Some(s))
    }

    /// Read the document folder into a tree-style structure.
    ///
    /// # Errors
    /// This function fails if filesystem ops fail (reading file, reading directory)
    #[tracing::instrument(skip(self))]
    pub fn get_doc_tree(&self) -> Result<INode> {
        fn recurse_tree(dir: &Path, node: &mut INode) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let entry_name = entry.file_name().to_string_lossy().to_string();
                // path is a directory, recurse over children
                if path.is_dir() {
                    let mut inner_node = INode {
                        name: entry_name,
                        children: Vec::new(),
                    };
                    recurse_tree(&path, &mut inner_node)?;
                    node.children.push(inner_node);
                } else {
                    // path is a file, add to children
                    node.children.push(INode {
                        name: entry_name,
                        children: Vec::new(),
                    });
                }
            }
            Ok(())
        }

        let mut root_node = INode {
            name: String::from("documents"),
            children: Vec::new(),
        };

        recurse_tree(Path::new(&self.doc_path), &mut root_node)?;
        Ok(root_node)
    }

    /// Create or overwrite the document at the provided `path` and populate it with the value of `new_doc`.
    /// `message` will be included in the commit message, and `branch` specifies which branch to commit to.
    /// `token` is a valid github auth token.
    ///
    /// # Errors
    /// /// This function will return an error if filesystem operations fail, or if any of the git operations fail
    /// This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop, but when applied,
    /// it creates errors that note the destructor for other values failing because of it (tree)
    #[allow(clippy::significant_drop_tightening)]
    #[tracing::instrument(skip_all)]
    pub fn put_doc<P: AsRef<Path> + Copy + std::fmt::Debug>(
        &self,
        path: P,
        new_doc: &str,
        message: &str,
        token: &str,
        branch: &str,  // Pass the branch name here
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();

        // Step 1: Checkout or create the branch
        Self::checkout_or_create_branch(&repo, branch)?;

        // Step 2: Write the document to the specified path
        let mut path_to_doc: PathBuf = PathBuf::from(".");
        path_to_doc.push(&self.doc_path);
        path_to_doc.push(path);

        // Wipe and write the new contents
        let mut file = fs::File::create(path_to_doc).wrap_err_with(|| {
            format!("Failed to wipe requested file for rewrite: {:?}", path.as_ref())
        })?;
        file.write_all(new_doc.as_bytes()).wrap_err_with(|| {
            format!("Failed to write new contents into file: {:?}", path.as_ref())
        })?;

        let msg = format!("[Hyde]: {message}");

        // Relative path from the repo root (./docs instead of ./repo/docs)
        let mut relative_path = PathBuf::from(
            env::var("DOC_PATH").wrap_err("The `DOC_PATH` environment variable was not set")?,
        );
        relative_path.push(path);

        // Stage the changes for commit
        Self::git_add(&repo, relative_path)?;

        // Step 3: Commit the changes to the branch
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);

        // Step 4: Push the branch to the remote repository
        Self::git_push_to_branch(&repo, branch, token)?;

        // Optional: Step 5: Create a pull request on GitHub and merge the branch into the main branch
        // You can use a GitHub API call to create the pull request and merge it.
        // Assuming you have a function to do this:
        // Self::create_and_merge_pull_request(branch, message, token)?;

        info!(
            "Document {:?} edited, committed to branch '{branch}' and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );
        debug!("Commit cleanup completed");

        Ok(())
    }


    /// Delete the document at the specified `path`.
    /// `message` will be included in the commit message, and `token` is a valid github auth token.
    ///
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current thread
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git operations fail
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop, but when applied,
    // it creates errors that note the destructor for other values failing because of it (tree)
    pub fn delete_doc<P: AsRef<Path> + Copy>(
        &self,
        path: P,
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_doc: PathBuf = PathBuf::new();
        path_to_doc.push(&self.doc_path);
        path_to_doc.push(path);
        let msg = format!("[Hyde]: {message}");
        // Relative to the root of the repo, not the current dir, so typically `./docs` instead of `./repo/docs`
        let mut relative_path = PathBuf::from(
            env::var("DOC_PATH").wrap_err("The `DOC_PATH` environment variable was not set")?,
        );
        // Standard practice is to stage commits by adding them to an index.
        relative_path.push(path);
        fs::remove_file(&path_to_doc).wrap_err_with(|| format!("Failed to remove document the document at {path_to_doc:?}"))?;
        Self::git_add(&repo, ".")?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token)?;
        drop(repo);
        info!(
            "Document {:?} removed and changes synced to Github with message: {message:?}",
            path.as_ref()
        );
        debug!("Commit cleanup completed");
        Ok(())
    }

    /// If the repository at the provided path exists, open it and fetch the latest changes from the `master` branch.
    /// If not, clone into the provided path.
    #[tracing::instrument]
    fn load_repository<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Repository> {
        if let Ok(repo) = Repository::open("./repo") {
            Self::git_pull(&repo)?;
            info!("Existing repository detected, fetching latest changes...");
            return Ok(repo);
        }

        let repository_url = env::var("REPO_URL")
            .wrap_err("The `REPO_URL` environment url was not set, this is required.")?;
        let output_path = Path::new("./repo");
        info!(
            "No repo detected, cloning {repository_url:?} into {:?}...",
            output_path.display()
        );
        let repo = Repository::clone(&repository_url, output_path)?;
        info!("Successfully cloned repo");
        Ok(repo)
    }

    /// Completely clone and open a new repository, deleting the old one.
    #[tracing::instrument(skip_all)]
    pub fn reclone(&self) -> Result<()> {
        // First clone a repo into `repo__tmp`, open that, swap out
        // TODO: nuke `repo__tmp` if it exists already
        let repo_path = Path::new("./repo");
        let tmp_path = Path::new("./repo__tmp");
        info!("Re-cloning repository, temporary repo will be created at {tmp_path:?}");
        let repository_url = env::var("REPO_URL")
            .wrap_err("The `REPO_URL` environment url was not set, this is required.")?;
        let tmp_repo = Repository::clone(&repository_url, tmp_path)?;
        info!("Pointing changes to new temp repository");
        let mut lock = self.repo.lock().unwrap();
        *lock = tmp_repo;
        info!("Deleting the old repo...");
        fs::remove_dir_all(repo_path)?;
        info!("Moving the temp repo to take the place of the old one");
        fs::rename(tmp_path, repo_path)?;
        *lock = Repository::open(repo_path)?;
        info!("Re-clone succeeded");
        drop(lock);
        Ok(())
    }

    /// Pull changes from upstream
    pub fn pull(&self) -> Result<()> {
        let guard = self.repo.lock().unwrap();
        Self::git_pull(&guard)
    }

    /// A code level re-implementation of `git add`.
    #[tracing::instrument(skip(repo))]
    fn git_add<P: AsRef<Path> + std::fmt::Debug>(repo: &Repository, path: P) -> Result<()> {
        let mut index = repo.index()?;
        // index.add_path(path.as_ref())?;
        let callback = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = repo.status_file(path).unwrap();
            let actions = vec![
                (Status::WT_DELETED, "deleted"),
                (Status::WT_MODIFIED, "modified"),
                (Status::WT_NEW, "added"),
                (Status::WT_RENAMED, "renamed")
            ];

           for (action, msg) in actions {
                if status.contains(action) {
                    info!("Index updated, {path:?} will be {msg} in the next commit");
                }
            }
            0
        };

        index.update_all([path.as_ref()], Some(callback))?;
        index.write()?;
        Ok(())
    }

    // /// A code level re-implementation of `git rm`.
    // fn git_rm<P: AsRef<Path>>(repo: &Repository, path: P) -> Result<()> {
    //     let mut index = repo.index()?;
    //     // index.add_path(path.as_ref())?;
    //     index.remove(path.as_ref(), 1)?;
    //     index.write()?;
    //     Ok(())
    // }

    /// A code level re-implementation of `git commit`.
    /// A function used to checkout or create a new branch based on the name.
    pub fn checkout_or_create_branch(repo: &Repository, branch_name: &str) -> Result<()> {
        // Get the current head reference
        let head = repo.head().wrap_err("Failed to get the head reference")?;
        // Peel the head to get the commit
        let commit = head.peel_to_commit().wrap_err("Failed to peel the head to commit")?;

        // Check if the branch already exists
        if repo.find_branch(branch_name, BranchType::Local).is_ok() {
            // If the branch exists, check it out
            repo.set_head(&format!("refs/heads/{}", branch_name)).wrap_err_with(|| {
                format!("Failed to set head to branch {branch_name}")
            })?;
        } else {
            // If the branch does not exist, create it
            repo.branch(branch_name, &commit, false).wrap_err_with(|| {
                format!("Failed to create branch {branch_name}")
            })?;

            // Now check out the newly created branch
            repo.set_head(&format!("refs/heads/{}", branch_name)).wrap_err_with(|| {
                format!("Failed to set head to new branch {branch_name}")
            })?;
        }

        Ok(())
    }

    
    /// Writes the current index as a commit, updating HEAD. This means it will only commit changes
    /// tracked by the index. If an author is not specified, the commit will be attributed to `Hyde`. Returns
    /// the id (A full or partial hash associated with a git object) tied to that commit.
    fn git_commit(repo: &Repository, message: String, author: Option<Signature>) -> Result<Oid> {
        let sig = match author {
            Some(sig) => sig,
            None => Signature::now("Hyde", "Hyde")?,
        };
        let tree = {
            let mut index = repo.index()?;
            let oid = index.write_tree()?;
            repo.find_tree(oid)?
        };
        let parent_commit = Self::find_last_commit(repo)?;
        Ok(repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent_commit])?)
    }

    /// A code level re-implementation of `git push`.
    ///
    /// Pushes the latest commit on the `master` branch to `origin/master`.
    ///
    /// `token` is a valid Github auth token.
    // TODO: stop hardcoding refspec and make it an argument.
    fn git_push(repo: &Repository, token: &str) -> Result<()> {
        let repository_url = env::var("REPO_URL").wrap_err("Repo url not set in env")?;
        let authenticated_url =
            repository_url.replace("https://", &format!("https://x-access-token:{token}@"));
        repo.remote_set_pushurl("origin", Some(&authenticated_url))?;
        let mut remote = repo.find_remote("origin")?;
        remote.connect(git2::Direction::Push)?;
        // Push master here, to master there
        remote.push(&["refs/heads/master:refs/heads/master"], None)?;
        remote.disconnect()?;
        Ok(())
    }

    /// Pushes the commits to a branch instead of the master branch.
    pub fn git_push_to_branch(repo: &Repository, branch_name: &str, token: &str) -> Result<()> {
        let repository_url = env::var("REPO_URL").wrap_err("Repo url not set in env")?;
        let authenticated_url = repository_url.replace("https://", &format!("https://x-access-token:{token}@"));
        repo.remote_set_pushurl("origin", Some(&authenticated_url))?;
        
        let mut remote = repo.find_remote("origin")?;
        remote.connect(git2::Direction::Push)?;

        remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], None)?;
        remote.disconnect()?;
        
        Ok(())
}

    /// A code level re-implementation of `git pull`, currently only pulls the `master` branch.
    ///
    /// Under the hood, `git pull` is shorthand for `git fetch`, followed by `git merge FETCH_HEAD`,
    /// where`FETCH_HEAD` is a reference to the latest commit that has just been fetched from the remote repository.
    fn git_pull(repo: &Repository) -> Result<()> {
        // https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
        // TODO: configure branch via environment variables
        let fetch_head = Self::git_fetch(repo)?;
        info!("Successfully fetched latest changes, merging...");
        Self::git_merge(repo, "master", fetch_head)?;
        info!("Successfully merged latest changes");
        Ok(())
    }

    /// A code level re-implementation of `git fetch`. `git fetch` will sync your local `origin/[BRANCH]` with the remote, but it won't
    /// merge those changes into `main`.
    ///
    /// This implementation fetches all branches.
    ///
    /// Returns a reference to the latest commit fetched from remote (`FETCH_HEAD`). This is done if you'd like to merge the remote changes into a local branch.
    fn git_fetch(repo: &Repository) -> Result<AnnotatedCommit<'_>> {
        let mut remote = repo.find_remote("origin")?;
        // "Always fetch all tags."
        // In Git, a `tag` is just a way to mark specific points in a repository's history. They're typically used for releases, eg `v1.0`.
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
        // https://git-scm.com/book/en/v2/Git-Internals-The-Refspec
        remote.fetch(
            &["+refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )?;
        drop(remote);

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        Ok(repo.reference_to_annotated_commit(&fetch_head)?)
    }

    /// A code level re-implementation of `git merge`. It accepts a [`git2::AnnotatedCommit`]. The interface
    /// is specifically written as the second half of `git pull`, so it would probably need to be modified to support
    /// more than that.
    fn git_merge(
        repo: &Repository,
        remote_branch: &str,
        fetch_commit: AnnotatedCommit<'_>,
    ) -> Result<()> {
        // First perform a merge analysis, this is done so that we know whether or not to fast forward
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        // Then select the appropriate merge, either a fast forward or a normal merge
        if analysis.0.is_fast_forward() {
            debug!("Performing fast forward merge");
            let refname = format!("refs/heads/{}", remote_branch);
            // This code will return early with an error if pulling into an empty repository.
            // That *should* never happen, so that handling was omitted, but if it's needed,
            // an example can be found at:
            // https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs#L160
            let mut reference = repo.find_reference(&refname)?;
            Self::fast_forward(repo, &mut reference, &fetch_commit)?;
        } else if analysis.0.is_normal() {
            debug!("Performing normal merge");
            let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
            Self::normal_merge(repo, &fetch_commit, &head_commit)?;
        } else {
            debug!("No work needed to merge");
        }
        Ok(())
    }

    /// This is a helper function called by [`Self::git_merge`], you probably don't want to call this
    /// directly.
    ///
    /// Merge the the `source` reference commit into on top of the reference `destination` commit.
    /// This is considered a "normal merge", as opposed to a fast forward merge. See [`Self::fast_forward`]
    /// for more info.
    fn normal_merge(
        repo: &Repository,
        source: &AnnotatedCommit,
        destination: &AnnotatedCommit,
    ) -> Result<()> {
        let source_tree = repo.find_commit(source.id())?.tree()?;
        let destination_tree = repo.find_commit(destination.id())?.tree()?;
        // The ancestor is the most recent commit that the source and destination share.
        let ancestor = repo
            .find_commit(repo.merge_base(source.id(), destination.id())?)?
            .tree()?;
        // A git index (or staging area) is where changes are written before they're committed.
        let mut idx = repo.merge_trees(&ancestor, &source_tree, &destination_tree, None)?;
        if idx.has_conflicts() {
            bail!("Unable to merge changes from {:?} into {:?} because there are merge conflicts and method is currently implemented to handle merge conflicts.", source.refname().unwrap(), destination.refname().unwrap());
        }
        // Write the changes to disk, then create and attach a merge commit to that tree then update the working tree to the latest commit.
        let result_tree = repo.find_tree(idx.write_tree()?)?;
        let _merge_commit = {
            let msg = format!("Merge: {} into {}", source.id(), destination.id());
            let sig = repo.signature()?;
            let destination_commit_parent = repo.find_commit(destination.id())?;
            let source_commit_parent = repo.find_commit(source.id())?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &msg,
                &result_tree,
                &[&destination_commit_parent, &source_commit_parent],
            )?
        };
        // Now update the working tree
        repo.checkout_head(None)?;

        Ok(())
    }

    /// This is a helper function used by [`Self::git_merge`], you probably don't want to call it
    /// directly.
    ///
    /// In some cases, a merge can be simplified by just moving the `HEAD` pointer forwards if the new
    /// commits are direct ancestors of the old `HEAD`.
    fn fast_forward(
        repo: &Repository,
        local_branch: &mut git2::Reference,
        remote_commit: &AnnotatedCommit,
    ) -> Result<()> {
        let lb_name = local_branch
            .name()
            .wrap_err("Local branch name isn't valid UTF-8")?
            .to_string();
        let msg = format!(
            "Fast forwarding: Setting {lb_name} to id: {}",
            remote_commit.id()
        );
        debug!("{msg}");
        local_branch.set_target(remote_commit.id(), &msg)?;
        repo.set_head(&lb_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(())
    }
    /// Returns the latest commit from `HEAD`.
    ///
    /// <https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html>
    pub fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
}
