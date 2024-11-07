//! Abstractions and interfaces over the git repository

use color_eyre::eyre::{bail, ContextCompat, WrapErr, Result};
use fs_err as fs;
use git2::{AnnotatedCommit, FetchOptions, IndexAddOption, Oid, Repository, Signature, Status, BranchType, build::CheckoutBuilder};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::path::Path;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::{debug, info, warn};

/// Interacts with a Jekyll repo's version control and filesystem.
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
    /// The remote URL of the repository.
    ///
    /// EG `https://github.com/foo/bar`
    repo_url: String,
    // TODO: if we move the github token generator here then we can clean up the interface massively
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
            repo_url,
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
    /// This function will return an error if filesystem operations fail, or if any of the git
    ///operations fail.
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop,
    // but when applied, it creates errors that note the destructor for other values failing
    // because of it (tree)
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
        Self::git_push(&repo, token, &self.repo_url)?;
        info!(
            "Document {:?} edited, committed to branch '{branch}' and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );

        Ok(())
    }

    /// Create or overwrite the asset at the provided `path`
    /// with `contents`. `message` will be included in the commit
    /// message, and `token` is a valid github auth token.
    ///
    /// # Arguments
    /// - `path` - the path of the asset to put relative to the assets folder
    /// - `contents` - A buffer containing the new asset data
    /// - `message` - textual context included with the git commit message
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
        Self::git_push(&repo, token, &self.repo_url)?;
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
        Self::git_push(&repo, token, &self.repo_url)?;
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
        path: P,
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_asset: PathBuf = PathBuf::from(&self.asset_path);
        path_to_asset.push(path);
        let msg = format!("[Hyde]: {message}");
        // Standard practice is to stage commits by adding them to an index.
        Self::delete_file(&path_to_asset)?;
        Self::git_add(&repo, ".")?;
        let commit_id = Self::git_commit(&repo, msg, None)?;
        debug!("New commit made with ID: {:?}", commit_id);
        Self::git_push(&repo, token, &self.repo_url)?;
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
    pub fn reclone(&self) -> Result<()> {
        // First clone a repo into `repo__tmp`, open that, swap out
        // TODO: nuke `repo__tmp` if it exists already
        let repo_path = Path::new("./repo"); // TODO: Possibly implement this path into new config?
        let tmp_path = Path::new("./repo__tmp"); // TODO: Same here?
        info!("Re-cloning repository, temporary repo will be created at {tmp_path:?}");
        let tmp_repo = Repository::clone(&self.repo_url, tmp_path)?;
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

    /// Checks out an existing branch or creates a new branch based on the given name.
    ///
    /// This function attempts to switch to a branch specified by `branch_name`. If the branch
    /// does not exist, it creates a new branch at the current HEAD commit. It handles both
    /// scenarios, logging the actions taken and returning an error if any operation fails.
    ///
    /// # Arguments
    /// - `branch_name` - A string slice that holds the name of the branch to check out or create.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The current HEAD reference cannot be retrieved.
    /// - The branch cannot be found or created.
    /// - The HEAD cannot be set to the specified branch.
    pub fn checkout_or_create_branch(&self, branch_name: &str) -> Result<()> {
        debug!("Attempting to checkout or create branch: {}", branch_name);
    
        // Lock the repository
        let repo = self.repo.lock().unwrap();

        // Use the repo within this scope
        {
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

    /// Pushes commits to a specified branch or all branches if no branch name is provided.
    ///
    /// # Arguments
    /// - `repo` - A reference to the `Repository` to push to.
    /// - `repo_url` - The URL of the remote repository.
    /// - `branch_name` - An optional name of the branch to push to. If `None`, pushes all branches.
    /// - `token` - The authentication token for the remote repository.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn git_push(
        repo: &Repository,
        repo_url: &str,
        branch_name: Option<&str>,
        token: &str,
    ) -> Result<()> {
        let authenticated_url = repo_url.replace("https://", &format!("https://x-access-token:{token}@"));
        repo.remote_set_pushurl("origin", Some(&authenticated_url))?;

        let mut remote = repo.find_remote("origin")?;
        remote.connect(git2::Direction::Push)?;

        match branch_name {
            Some(branch) => {
                // Push only the specified branch
                remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch, branch)], None)?;
            },
            None => {
                // Push all branches
                remote.push(&["+refs/heads/*:refs/heads/*"], None)?;
            }
        }

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
        let fetch_head = Self::git_fetch(repo, None)?;
        info!("Successfully fetched latest changes, merging...");
        Self::git_merge(repo, "master", fetch_head)?;
        info!("Successfully merged latest changes");
        Ok(())
    }

   /// Pull the latest changes for a specified branch in the repository.
    ///
    /// This function performs a series of operations that mimic the behavior of the `git pull` command:
    /// 1. **Reset Local Changes**: Discards any local changes in the working directory to ensure that the 
    ///    repository is in a clean state before pulling new changes.
    /// 2. **Check Branch Existence**: Verifies whether the specified local branch exists. If it does not,
    ///    an error is returned.
    /// 3. **Set Upstream Tracking**: Attempts to set the upstream tracking reference for the specified branch 
    ///    if it is not already set. This allows the local branch to track changes from the corresponding 
    ///    remote branch.
    /// 4. **Fetch Changes**: Retrieves the latest changes from the remote repository for the specified branch.
    /// 5. **Reset Local Branch**: Resets the local branch to match the state of the upstream branch, effectively 
    ///    discarding any local commits that are not present in the upstream branch.
    ///
    /// # Parameters
    /// - `branch`: A string slice that represents the name of the local branch to pull changes for.
    ///
    /// # Errors
    /// This function will return an error if any of the following occur:
    /// - The specified branch does not exist.
    /// - There are issues with resetting the repository or finding references.
    /// - Fetching changes from the remote repository fails.
    #[tracing::instrument(skip(self))]
    pub fn git_pull_branch(&self, branch: &str) -> Result<()> {
        // Lock and check the repository
        let repo = self.repo.lock().unwrap();
        
        debug!("Current repository state: {:?}", repo.state()); //leaving here incase of errors

        // Discard any local changes
        self.git_reset(&repo)?;

        // Check if the local branch exists
        let _branch_reference = {
            let branch = repo.find_branch(branch, git2::BranchType::Local)?;
            branch.get().peel_to_commit() // Get the commit for the branch
        };

        // Attempt to set upstream for the branch if it isn't already set
        self.set_branch_upstream(&repo, branch)?;

        // Fetch changes from the remote for this branch
        Self::git_fetch(&repo, Some(branch))?;
        info!("Successfully fetched latest changes for branch '{}'.", branch);

        // Prepare to reset the local branch to match the upstream branch
        let upstream_ref = format!("refs/remotes/origin/{}", branch);

        // Reset the local branch to match the upstream branch
        {
            let upstream_commit = repo.find_reference(&upstream_ref)?.peel_to_commit()?;
            let upstream_object = upstream_commit.as_object();
            repo.reset(upstream_object, git2::ResetType::Hard, None)?;
        } // `repo` will be dropped here after its last use

        info!("Local branch '{}' has been reset to match upstream branch '{}'.", branch, upstream_ref);

        Ok(())
    }

    /// Sets the upstream tracking branch for a given local branch.
    ///
    /// This function checks if the specified local branch has an upstream branch set. 
    /// If not, it attempts to fetch the latest changes from the specified remote repository 
    /// (defaulting to "origin") and sets the upstream to the corresponding remote branch.
    ///
    /// # Arguments
    /// - `repo` - A mutable reference to the `git2::Repository`.
    /// - `branch_name` - The name of the local branch for which to set the upstream.
    ///
    /// # Errors
    /// Returns an error if the upstream branch cannot be set or if the remote branch does not exist.
    fn set_branch_upstream(&self, repo: &git2::Repository, branch_name: &str) -> Result<()> {
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
    /// - `repo` - A reference to the local Git repository.
    /// - `branch_name` - The name of the branch to fetch from the remote.
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
    /// - `repo` - A reference to the local Git repository.
    /// - `branch_name` - The name of the local branch for which the upstream is being set.
    /// - `remote_name` - The name of the remote (typically "origin") from which the upstream is being set.
    ///
    /// # Returns
    /// This function returns a `Result` indicating success or failure. 
    /// If the remote branch does not exist, or if an error occurs while setting the upstream,
    /// it will return a `color_eyre::eyre::Result`, which contains context about the failure.
    fn set_upstream_if_exists(
        &self,
        repo: &git2::Repository,
        branch_name: &str,
        remote_name: &str,
    ) -> Result<()> {
        let remote_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);

        // Check if the remote branch exists
        let remote_branch = repo.find_reference(&remote_ref)
            .context(format!("Remote branch '{}' not found", remote_ref))?;

        // Get the shorthand branch name
        let remote_branch_name = remote_branch.shorthand()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get shorthand name for remote branch '{}'", remote_ref))?;

        info!(
            "Setting upstream for local branch '{}' to remote '{}'",
            branch_name, remote_branch_name
        );

        let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)
            .context("Failed to find local branch")?;

        branch.set_upstream(Some(remote_branch_name))
            .context("Failed to set upstream")?;

        info!("Successfully set upstream for branch '{}' to '{}'", branch_name, remote_branch_name);

        Ok(())
    }

    /// A code level re-implementation of `git fetch`.
    /// `git fetch` will sync your local `origin/[BRANCH]` with the remote, but it won't
    /// merge those changes into your local branch.
    /// 
    /// This implementation can fetch either all branches or a specified branch.
    ///
    /// Returns a reference to the latest commit fetched from remote (`FETCH_HEAD`).
    fn git_fetch<'a>(repo: &'a Repository, branch: Option<&'a str>) -> Result<AnnotatedCommit<'a>> {
        let mut remote = repo.find_remote("origin")?;

        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);
        
        match branch {
            Some(branch_name) => {
                // Fetch only the specified branch
                remote.fetch(
                    &[&format!("refs/heads/{branch_name}:refs/remotes/origin/{branch_name}")],
                    Some(&mut fetch_options),
                    None,
                )?;
            },
            None => {
                // Fetch all branches
                remote.fetch(
                    &["+refs/heads/*:refs/remotes/origin/*"],
                    Some(&mut fetch_options),
                    None,
                )?;
            },
        }
        drop(remote);

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_head_name = fetch_head.name().unwrap_or("unknown");
        debug!("Fetched HEAD: {}", fetch_head_name);
        Ok(repo.reference_to_annotated_commit(&fetch_head)?)
    }

    /// A code level re-implementation of `git merge`. It accepts a [`git2::AnnotatedCommit`]. The interface
    /// is specifically written as the second half of `git pull`, so it would probably need to be modified to support
    fn git_merge(
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

    /// Resets the working directory to the last committed state, discarding any uncommitted changes.
    /// 
    /// This function checks the status of the repository and, if any uncommitted 
    /// changes are detected, it resets the working directory to the last committed state. 
    /// This ensures that the working directory is clean before pulling changes from the remote.
    /// 
    /// This function is equivalent to running `git reset --hard`.
    /// 
    /// # Errors
    /// This function will return an error if retrieving the status fails or if 
    /// checking out the head fails.
    fn git_reset(&self, repo: &git2::Repository) -> Result<()> {
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
            repo.checkout_head(Some(&mut checkout_builder))
                .wrap_err("Failed to checkout HEAD and discard uncommitted changes")?;
            info!("Discarded uncommitted changes and reset to the last commit.");
        }

        Ok(())
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
