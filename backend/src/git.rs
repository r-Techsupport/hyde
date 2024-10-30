//! Abstractions and interfaces over the git repository

use color_eyre::eyre::{bail, ContextCompat};
use color_eyre::{eyre::Context, Result};
use fs_err as fs;
use git2::{AnnotatedCommit, FetchOptions, IndexAddOption, Oid, Repository, Signature, Status};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::path::Path;
use std::{
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
    /// The path to the assets folder, relative to the server executable.
    ///
    /// EG: `./repo/assets`
    asset_path: PathBuf,
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
    pub fn new(
        repo_url: String,
        repo_path: String,
        docs_path: String,
        assets_path: String,
    ) -> Result<Self> {
        let doc_path = PathBuf::from(docs_path);
        let asset_path = PathBuf::from(assets_path);
        let repo = Self::load_repository(&repo_url, &repo_path)?;
        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            doc_path,
            asset_path,
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
        let mut path_to_doc: PathBuf = PathBuf::from(&self.doc_path);
        path_to_doc.push(path);
        let doc = Self::get_file(&path_to_doc)?.map(|v| String::from_utf8(v).unwrap());
        Ok(doc)
    }

    /// Return the asset from the provided `path`, where `path` is the
    /// path to the markdown file relative to the root of the assets folder.
    ///
    /// The return type is a little bit messy, but I needed to differentiate between
    /// "file not found", and "failed to read file"
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail.
    #[tracing::instrument(skip(self))]
    pub fn get_asset<P: AsRef<Path> + std::fmt::Debug>(&self, path: P) -> Result<Option<Vec<u8>>> {
        let mut path_to_asset: PathBuf = PathBuf::from(".");
        path_to_asset.push(&self.asset_path);
        path_to_asset.push(path);
        let asset = Self::get_file(&path_to_asset)?;
        Ok(asset)
    }

    /// Read the document folder into a tree-style structure.
    ///
    /// # Errors
    /// This function fails if filesystem ops fail (reading file, reading directory)
    #[tracing::instrument(skip(self))]
    pub fn get_doc_tree(&self) -> Result<INode> {
        let doc_tree = Self::get_file_tree(&self.doc_path)?;
        Ok(doc_tree)
    }

    /// Read the assets folder into a tree-style structure.
    ///
    /// # Errors
    /// This function fails if filesystem ops fail (reading file, reading directory)
    #[tracing::instrument(skip(self))]
    pub fn get_asset_tree(&self) -> Result<INode> {
        let asset_tree = Self::get_file_tree(&self.asset_path)?;
        Ok(asset_tree)
    }

    /// Create or overwrite the document at the provided `path`
    /// and populate it with the value of `new_doc`.`message` will be included in the commit
    /// message, and `token` is a valid github auth token.
    ///
    /// # Arguments
    /// - `repo_url` - the URL of the remote for the wiki repository
    /// - `path` - the path of the document to put relative to the documents folder
    /// - `new_doc` - contents of the new document
    /// - `message` - textual context associated with the message
    /// - `token` - github authentication token
    ///
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current
    /// thread.
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git
    ///operations fail.
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop,
    // but when applied, it creates errors that note the destructor for other values failing
    // because of it (tree)
    #[allow(clippy::significant_drop_tightening)]
    #[tracing::instrument(skip_all)]
    pub fn put_doc<P: AsRef<Path> + Copy + std::fmt::Debug>(
        &self,
        repo_url: &str,
        path: P,
        new_doc: &str,
        message: &str,
        token: &str,
    ) -> Result<()> {
        // TODO: refactoring hopefully means that all paths can just assume that it's relative to
        // the root of the repo
        let repo = self.repo.lock().unwrap();
        let mut path_to_doc: PathBuf = PathBuf::from(&self.doc_path);
        path_to_doc.push(path.as_ref());
        Self::put_file(&path_to_doc, new_doc.as_bytes())?;
        let msg = format!("[Hyde]: {message}");
        // Self::git_add(&repo, ".")?;
        Self::git_add(&repo, ".")?;
        // Self::git_add(&repo, &path_to_doc)?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token, repo_url)?;
        info!(
            "Document {:?} edited and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );
        debug!("Commit cleanup completed");
        Ok(())
    }

    /// Create or overwrite the asset at the provided `path`
    /// with `contents`. `message` will be included in the commit
    /// message, and `token` is a valid github auth token.
    ///
    /// # Arguments
    /// - `repo_url` - the URL of the remote for the wiki repository
    /// - `path` - the path of the asset to put relative to the assets folder
    /// - `contents` - contents of the new document
    /// - `message` - textual context associated with the message
    /// - `token` - github authentication token
    ///
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current
    /// thread.
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git
    ///operations fail.
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop,
    // but when applied, it creates errors that note the destructor for other values failing
    // because of it (tree)
    #[allow(clippy::significant_drop_tightening)]
    #[tracing::instrument(skip_all)]
    pub fn put_asset<P: AsRef<Path> + Copy + std::fmt::Debug>(
        &self,
        repo_url: &str,
        path: P,
        contents: &[u8],
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_asset: PathBuf = PathBuf::from(&self.asset_path);
        path_to_asset.push(path.as_ref());
        Self::put_file(&path_to_asset, contents)?;
        let msg = format!("[Hyde]: {message}");
        Self::git_add(&repo, ".")?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token, repo_url)?;
        info!(
            "Asset {:?} edited and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );
        debug!("Commit cleanup completed");
        Ok(())
    }

    /// Delete the document at the specified `path`.
    /// `message` will be included in the commit message, and `token` is a valid github auth token.
    ///
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current
    /// thread.
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git
    /// operations fail.
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop,
    // but when applied, it creates errors that note the destructor for other values failing
    // because of it (tree)
    pub fn delete_doc<P: AsRef<Path> + Copy>(
        &self,
        repo_url: &str,
        path: P,
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_doc: PathBuf = PathBuf::from(&self.doc_path);
        path_to_doc.push(path);
        let msg = format!("[Hyde]: {message}");
        Self::delete_file(&path_to_doc)?;
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

    /// Delete the document at the specified `path`.
    /// and `token` is a valid github auth token.
    ///
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current
    /// thread.
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git
    /// operations fail.
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop,
    // but when applied, it creates errors that note the destructor for other values failing
    // because of it (tree)
    pub fn delete_asset<P: AsRef<Path> + Copy>(
        &self,
        asset_folder_path: &str,
        repo_url: &str,
        path: P,
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_asset: PathBuf = PathBuf::from(asset_folder_path);
        path_to_asset.push(path);
        let msg = format!("[Hyde]: {message}");
        // Relative to the root of the repo, not the current dir, so typically `./docs` instead of `./repo/docs`
        let mut relative_path = PathBuf::from(asset_folder_path);
        // Standard practice is to stage commits by adding them to an index.
        relative_path.push(path);
        Self::delete_file(&path_to_asset)?;
        Self::git_add(&repo, ".")?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token, repo_url)?;
        drop(repo);
        info!(
            "Asset {:?} removed and changes synced to Github with message: {message:?}",
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
    #[tracing::instrument(skip(repo), err)]
    fn git_add<P: AsRef<Path> + std::fmt::Debug>(repo: &Repository, path: P) -> Result<()> {
        let mut index = repo.index()?;
        let callback = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            debug!("Processing file: {path:?}");
            let status = repo.status_file(path).unwrap();
            let actions = vec![
                (Status::WT_DELETED, "deleted"),
                (Status::WT_MODIFIED, "modified"),
                (Status::WT_NEW, "added"),
                (Status::WT_RENAMED, "renamed"),
            ];

            for (action, msg) in actions {
                if status.contains(action) {
                    info!("Index updated, {path:?} will be {msg} in the next commit");
                }
            }
            0
        };
        // So as far as I can tell, `update_all` doesn't catch
        // *new* files, so add is called first.
        info!("Adding everything to the index");
        index.add_all(["*"], IndexAddOption::DEFAULT, Some(callback))?;
        info!("Updating the index for {path:?}");
        index.update_all([path.as_ref()], Some(callback))?;
        index.write()?;
        Ok(())
    }

    /// A code level re-implementation of `git commit`.
    ///
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

    /// A code level re-implementation of `git pull`, currently only pulls the `master` branch.
    ///
    /// Under the hood, `git pull` is shorthand for `git fetch`, followed by `git merge FETCH_HEAD`,
    /// where `FETCH_HEAD` is a reference to the latest commit that has just been fetched from the remote repository.
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
    fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
}

impl RepoFileSystem for Interface {
    fn get_file<P: AsRef<Path> + Copy>(path: P) -> Result<Option<Vec<u8>>> {
        let mut path_to_file: PathBuf = PathBuf::from("./repo");
        path_to_file.push(path);
        if !path_to_file.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(path_to_file)?;
        let mut o: Vec<u8> = Vec::new();
        file.read_to_end(&mut o)?;
        Ok(Some(o))
    }

    #[tracing::instrument(skip(contents))]
    fn put_file<P: AsRef<Path> + Copy + Debug>(path: P, contents: &[u8]) -> Result<()> {
        let mut path_to_file: PathBuf = PathBuf::from("./repo");
        path_to_file.push(path);
        // wipe the file
        let mut file = fs::File::create(path_to_file).wrap_err_with(|| {
            format!(
                "Failed to wipe requested file for rewrite: {:?}",
                path.as_ref()
            )
        })?;
        // write the new contents in
        file.write_all(contents).wrap_err_with(|| {
            format!(
                "Failed to write new contents into file: {:?}",
                path.as_ref()
            )
        })?;
        Ok(())
    }

    fn delete_file<P: AsRef<Path> + Copy>(path: P) -> Result<()> {
        let mut path_to_file: PathBuf = PathBuf::from("./repo");
        path_to_file.push(path);
        fs::remove_file(&path_to_file)
            .wrap_err_with(|| format!("Failed to remove the document at {path_to_file:?}"))?;
        Ok(())
    }

    fn get_file_tree<P: AsRef<Path> + Copy>(path: P) -> Result<INode> {
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
            // Sort entries alphabetically
            node.children.sort_by_cached_key(|e| e.name.clone());
            Ok(())
        }

        let mut root_node = INode {
            name: path
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            children: Vec::new(),
        };
        let mut trunk_path = PathBuf::from("./repo");
        trunk_path.push(path.as_ref());
        recurse_tree(&trunk_path, &mut root_node)?;
        Ok(root_node)
    }
}

/// An abstraction over the filesystem for the git repository. Does not implement the version
/// control side of things
trait RepoFileSystem {
    /// Read the file at the provided location, relative to the root of the repo
    fn get_file<P: AsRef<Path> + Copy + Debug>(path: P) -> Result<Option<Vec<u8>>>;

    /// Create a file at the provided location, or overwrite it if it exists, relative to
    /// the root of the repo
    fn put_file<P: AsRef<Path> + Copy + Debug>(path: P, contents: &[u8]) -> Result<()>;

    /// Delete the file at the provided location, relative to the root of the repo
    fn delete_file<P: AsRef<Path> + Copy + Debug>(path: P) -> Result<()>;

    /// Read the directory at the provided location and create a representation of that dir's
    /// filesystem tree.
    fn get_file_tree<P: AsRef<Path> + Copy + Debug>(path: P) -> Result<INode>;
}

// TODO: Split git code out into a new (hopefully git backend agnostic) trait so that the impl block
// isn't so massive
// trait Git {}

// TODO: unit tests for get_inode_path and that sort of thing
