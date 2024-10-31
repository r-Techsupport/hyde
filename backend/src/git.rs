//! Abstractions and interfaces over the git repository

use color_eyre::eyre::{bail, ContextCompat, WrapErr, Result};
use git2::{AnnotatedCommit, FetchOptions, Oid, Repository, Signature, Status, BranchType, build::CheckoutBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::{debug, info, warn, error};

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
    pub fn new(repo_url: String, repo_path: String, docs_path: String) -> Result<Self> {
        let mut doc_path = PathBuf::from(&repo_path);
        doc_path.push(docs_path);
        let repo = Self::load_repository(&repo_url, &repo_path)?;
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
    /// This function will return an error if filesystem operations fail, or if any of the git operations fail
    /// This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop, but when applied,
    /// it creates errors that note the destructor for other values failing because of it (tree)
    #[allow(clippy::significant_drop_tightening)]
    #[tracing::instrument(skip_all)]
    pub fn put_doc<P: AsRef<Path> + Copy + std::fmt::Debug>(
        &self,
        repo_url: &str,
        path: P,
        new_doc: &str,
        message: &str,
        token: &str,
        branch: &str,  // Pass the branch name here
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();

        // Step 1: Checkout or create the branch
        self.checkout_or_create_branch(&repo, branch)?;

        // Step 2: Write the document to the specified path
        let mut path_to_doc: PathBuf = PathBuf::from("./");
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

        // Stage the changes for commit
        Self::git_add(&repo, ".")?;

        // Step 3: Commit the changes to the branch
        Self::git_commit(&repo, msg, None)?;

        // Step 4: Push the branch to the remote repository
        Self::git_push_to_branch(&repo, repo_url, branch, token)?;

        info!(
            "Document {:?} edited, committed to branch '{branch}' and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );

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
    /// This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop, but when applied,
    /// it creates errors that note the destructor for other values failing because of it (tree)
    pub fn delete_doc<P: AsRef<Path> + Copy>(
        &self,
        doc_path: &str,
        repo_url: &str,
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
        let mut relative_path = PathBuf::from(doc_path);
        // Standard practice is to stage commits by adding them to an index.
        relative_path.push(path);
        fs::remove_file(&path_to_doc).wrap_err_with(|| format!("Failed to remove document the document at {path_to_doc:?}"))?;
        Self::git_add(&repo, ".")?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token, repo_url)?;
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
    fn load_repository(repo_url: &str, repo_path: &str) -> Result<Repository> {
        if let Ok(repo) = Repository::open(repo_path) {
            info!("Existing repository detected, fetching latest changes");
            Self::git_pull(&repo)?;
            return Ok(repo);
        }

        let output_path = Path::new(repo_path);
        info!(
            "No repo detected, cloning {repo_url:?} into {:?}...",
            output_path.display()
        );
        let repo = Repository::clone(repo_url, output_path)?;
        info!("Successfully cloned repo");
        Ok(repo)
    }

    /// Completely clone and open a new repository, deleting the old one.
    #[tracing::instrument(skip_all)]
    pub fn reclone(&self, repo_url: &str) -> Result<()> {
        // First clone a repo into `repo__tmp`, open that, swap out
        // TODO: nuke `repo__tmp` if it exists already
        let repo_path = Path::new("./repo"); // TODO: Possibly implement this path into new config?
        let tmp_path = Path::new("./repo__tmp"); // TODO: Same here?
        info!("Re-cloning repository, temporary repo will be created at {tmp_path:?}");
        let tmp_repo = Repository::clone(repo_url, tmp_path)?;
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
    pub fn checkout_or_create_branch(&self, repo: &Repository, branch_name: &str) -> Result<()> {
        debug!("Attempting to checkout or create branch: {}", branch_name);
    
        // Get the current head reference
        let head = repo.head().wrap_err("Failed to get the head reference")?;
    
        // Peel the head to get the commit
        let commit = head.peel_to_commit().wrap_err("Failed to peel the head to commit")?;
        debug!("Current commit for head: {:?}", commit.id());
    
        // Check if the branch already exists
        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(_branch) => {
                info!("Branch '{}' already exists. Checking it out...", branch_name);
                // If the branch exists, check it out
                repo.set_head(&format!("refs/heads/{}", branch_name)).wrap_err_with(|| {
                    format!("Failed to set head to branch {}", branch_name)
                })?;
                info!("Checked out to existing branch '{}'", branch_name);
            }
            Err(_) => {
                info!("Branch '{}' does not exist. Creating new branch...", branch_name);
                // If the branch does not exist, create it
                repo.branch(branch_name, &commit, false).wrap_err_with(|| {
                    format!("Failed to create branch {}", branch_name)
                })?;
                info!("Successfully created new branch '{}'. Now checking it out...", branch_name);
    
                // Now check out the newly created branch
                repo.set_head(&format!("refs/heads/{}", branch_name)).wrap_err_with(|| {
                    format!("Failed to set head to new branch {}", branch_name)
                })?;
                info!("Checked out to newly created branch '{}'", branch_name);
            }
        }
    
        debug!("Successfully checked out or created branch: {}", branch_name);
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
    fn git_push(repo: &Repository, token: &str, repo_url: &str) -> Result<()> {
        let authenticated_url =
            repo_url.replace("https://", &format!("https://x-access-token:{token}@"));
        repo.remote_set_pushurl("origin", Some(&authenticated_url))?;
        let mut remote = repo.find_remote("origin")?;
        remote.connect(git2::Direction::Push)?;
        // Push master here, to master there
        remote.push(&["refs/heads/master:refs/heads/master"], None)?;
        remote.disconnect()?;
        Ok(())
    }

    /// Pushes the commits to a branch instead of the master branch.
    pub fn git_push_to_branch(
        repo: &Repository,
        repo_url: &str,
        branch_name: &str,
        token: &str,
    ) -> Result<()> {
        let authenticated_url = repo_url.replace("https://", &format!("https://x-access-token:{token}@"));
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

    /// A code level re-implementation of `git pull` for a specified branch.
    /// 
    /// Under the hood, `git pull` is shorthand for `git fetch`, followed by `git merge FETCH_HEAD`,
    /// where `FETCH_HEAD` is a reference to the latest commit that has just been fetched from the remote repository.
    #[tracing::instrument(skip(self))]
    pub fn git_pull_branch(&self, branch: &str) -> Result<()> {
        // Lock and check the repository
        let repo = self.lock_and_check_repo()?;
    
        // Update the index to ensure it's in sync
        self.update_index(&repo)?;
    
        // Check for uncommitted changes and reset if necessary
        self.check_and_reset_uncommitted_changes(&repo)?;
    
        // Find the local branch
        let branch_name = {
            let branch = repo.find_branch(branch, git2::BranchType::Local)?;
            branch.name()?.unwrap_or("unknown").to_string()
        };
    
        // Attempt to set upstream for the branch
        self.set_branch_upstream(&repo, &branch_name)?;
    
        // Fetch and pull changes
        let fetch_head = Self::git_fetch_branch(&repo, branch_name.as_str())?;
        info!("Successfully fetched latest changes for branch '{}', merging...", branch_name);
    
        // Check for divergence
        let local_commit = repo.head()?.peel_to_commit()?;
        let remote_commit = fetch_head.id();
    
        if local_commit.id() != remote_commit {
            // If there are diverged commits, perform a merge
            info!("Local branch and remote branch have diverged. Performing merge...");
            Self::git_merge_from_branch(&repo, branch_name.as_str(), fetch_head)?;
        } else {
            info!("Local branch is up to date with remote branch.");
        }
    
        // Checkout the latest commit
        self.checkout_latest_commit(&repo)?;
    
        Ok(())
    }

    /// Sets the upstream tracking branch for a given local branch.
    ///
    /// This function checks if the specified local branch has an upstream branch set. 
    /// If not, it attempts to fetch the latest changes from the specified remote repository 
    /// (defaulting to "origin") and sets the upstream to the corresponding remote branch.
    ///
    /// # Arguments
    ///
    /// * `repo` - A mutable reference to the `git2::Repository`.
    /// * `branch_name` - The name of the local branch for which to set the upstream.
    ///
    /// # Errors
    ///
    /// Returns an error if the upstream branch cannot be set or if the remote branch does not exist.
    fn set_branch_upstream(&self, repo: &git2::Repository, branch_name: &str) -> Result<(), git2::Error> {
        // Get the local branch
        let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
    
        // Check if upstream is already set
        if branch.upstream().is_ok() {
            info!("Upstream is already set for branch '{}'", branch_name);
            return Ok(());
        }
    
        // Fetch latest changes from remote
        let remote_name = "origin";
        self.fetch_remote_branch(repo, branch_name)?;
    
        // Attempt to set upstream for the branch
        self.set_upstream_if_exists(repo, branch_name, remote_name)
    }
    
    /// Fetches the specified branch from the remote repository.
    ///
    /// This function connects to the remote named "origin" and fetches the latest updates for the given
    /// branch. If the fetch is successful, the branch will be updated with the latest changes from the remote.
    ///
    /// # Arguments
    ///
    /// * `repo` - A reference to the local Git repository.
    /// * `branch_name` - The name of the branch to fetch from the remote.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` indicating success or failure. If the fetch operation fails,
    /// it will return a `git2::Error`.
    fn fetch_remote_branch(&self, repo: &git2::Repository, branch_name: &str) -> Result<(), git2::Error> {
        let mut remote = repo.find_remote("origin")?;
        remote.fetch::<&str>(&[branch_name], None, None)?;
        Ok(())
    }
    
    /// Sets the upstream for a local branch to a corresponding remote branch if it exists.
    ///
    /// This function checks if the specified remote branch exists. If it does, it sets the upstream
    /// of the local branch to the remote branch. The upstream branch is used for tracking remote changes.
    ///
    /// # Arguments
    ///
    /// * `repo` - A reference to the local Git repository.
    /// * `branch_name` - The name of the local branch for which the upstream is being set.
    /// * `remote_name` - The name of the remote (typically "origin") from which the upstream is being set.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` indicating success or failure. If the remote branch does not exist
    /// or setting the upstream fails, it will return a `git2::Error`.
    fn set_upstream_if_exists(&self, repo: &git2::Repository, branch_name: &str, remote_name: &str) -> Result<(), git2::Error> {
        let remote_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);
    
        // Check if the remote branch exists
        match repo.find_reference(&remote_ref) {
            Ok(remote_branch) => {
                // Get the shorthand branch name
                if let Some(remote_branch_name) = remote_branch.shorthand() {
                    info!("Setting upstream for local branch '{}' to remote '{}'", branch_name, remote_branch_name);
                    let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
                    branch.set_upstream(Some(remote_branch_name))?;
                    info!("Successfully set upstream for branch '{}' to '{}'", branch_name, remote_branch_name);
                } else {
                    error!("Failed to get shorthand name for remote branch '{}'", remote_ref);
                    return Err(git2::Error::from_str("Remote branch shorthand retrieval failed."));
                }
            }
            Err(err) => {
                error!("Remote branch '{}' not found; cannot set upstream: {:?}", remote_ref, err);
                return Err(git2::Error::from_str("Remote branch not found."));
            }
        }
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

    /// A code level re-implementation of `git fetch` for a specified branch.
    /// `git fetch` will sync your local `origin/[BRANCH]` with the remote, but it won't
    /// merge those changes into your local branch.
    /// 
    /// This implementation fetches only the specified branch.
    /// 
    /// Returns a reference to the latest commit fetched from remote (`FETCH_HEAD`). This is done if you'd like to merge the remote changes into a local branch.
    fn git_fetch_branch<'a>(repo: &'a Repository, branch: &'a str) -> Result<AnnotatedCommit<'a>> {
        let mut remote = repo.find_remote("origin")?;
    
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
    
        // Fetch only the specified branch
        remote.fetch(
            &[&format!("refs/heads/{branch}:refs/remotes/origin/{branch}")],
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

    /// A code level re-implementation of `git merge` for a specified remote branch.
    /// This function handles the merge process for pulling changes specifically from the given branch.
    fn git_merge_from_branch(
        repo: &Repository,
        remote_branch: &str,
        fetch_commit: AnnotatedCommit<'_>,
    ) -> Result<()> {
        // First perform a merge analysis to understand how to proceed
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        // Handle fast-forward merges
        if analysis.0.is_fast_forward() {
            debug!("Performing fast forward merge from branch '{}'", remote_branch);
            let refname = format!("refs/heads/{}", remote_branch);
            let mut reference = repo.find_reference(&refname)?;
            Self::fast_forward(repo, &mut reference, &fetch_commit)?;
        } 
        // Handle normal merges
        else if analysis.0.is_normal() {
            debug!("Performing normal merge from branch '{}'", remote_branch);
            let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
            Self::normal_merge(repo, &fetch_commit, &head_commit)?;
        } 
        // If no merging is needed
        else {
            debug!("No work needed to merge from branch '{}'", remote_branch);
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

    /// Checks out the latest commit in the specified repository.
    ///
    /// This function retrieves the most recent commit from the repository's head and performs a 
    /// checkout to that commit. It ensures that the working directory reflects the state of the 
    /// latest commit in the current branch. If the checkout operation fails, an error will be 
    /// returned.
    ///
    /// # Arguments
    /// * `repo` - A reference to the `git2::Repository` from which the latest commit is retrieved.
    ///
    /// # Errors
    /// Returns an error if the repository cannot be accessed, if the commit cannot be found, or 
    /// if the checkout operation fails.
    fn checkout_latest_commit(&self, repo: &Repository) -> Result<()> {
        // Reuse find_last_commit here as well
        let head_commit = Self::find_last_commit(repo)?;
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_tree(head_commit.as_object(), Some(&mut checkout_builder))?;
        info!("Checked out to the latest commit.");
        
        Ok(())
    }

    /// Provides a mutable reference to the locked repository.
    ///
    /// This function locks the repository and returns a `MutexGuard`, allowing 
    /// access to the repository for operations that modify its state. 
    ///
    /// # Panics
    /// This function will panic if the lock cannot be acquired.
    pub fn get_repo(&self) -> std::sync::MutexGuard<'_, Repository> {
        self.repo.lock().unwrap()
    }

    /// Locks the repository and checks its validity.
    ///
    /// This function ensures that the repository is not bare and is initialized correctly.
    ///
    /// # Errors
    /// Returns an error if the repository is not initialized correctly.
    fn lock_and_check_repo(&self) -> Result<std::sync::MutexGuard<'_, git2::Repository>> {
        let repo = self.repo.lock().unwrap();
        debug!("Repository path: {:?}", repo.path());
    
        if repo.is_bare() {
            info!("Repository is not initialized correctly, aborting operation.");
            bail!("Repository is not initialized correctly");
        }
    
        Ok(repo)
    }

    /// Updates the repository index to ensure it is in sync.
    ///
    /// This function adds all changes in the working directory to the index. 
    /// It ensures that the index is updated before performing any pull operations.
    ///
    /// # Errors
    /// This function will return an error if updating the index fails.
    fn update_index(&self, repo: &git2::Repository) -> Result<()> {
        let mut index = repo.index()?;
        index.add_all(std::iter::once(&"*"), git2::IndexAddOption::DEFAULT, None)?;
        
        Ok(())
    }

    /// Checks for uncommitted changes in the repository and resets if found.
    ///
    /// This function checks the status of the repository and, if any uncommitted 
    /// changes are detected, it discards those changes by checking out the last 
    /// committed state. This ensures that the working directory is clean before 
    /// pulling changes from the remote.
    ///
    /// # Errors
    /// This function will return an error if retrieving the status fails or if 
    /// checking out the head fails.
    fn check_and_reset_uncommitted_changes(&self, repo: &git2::Repository) -> Result<()> {
        // Get the current status of the repository
        let status = repo.statuses(None)?;
        
        // Log the status of each file
        for entry in status.iter() {
            debug!("File: {:?}, Status: {:?}", entry.path(), entry.status());
        }
    
        // Check for uncommitted changes
        if status.iter().any(|s| s.status() != git2::Status::CURRENT) {
            info!("Uncommitted changes found. Discarding changes before pulling.");
            
            // Create a checkout builder to discard changes
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force();
            
            // Checkout HEAD to discard uncommitted changes
            repo.checkout_head(Some(&mut checkout_builder))?;
            info!("Discarded uncommitted changes and reset to the last commit.");
        }
    
        Ok(())
    }
}
