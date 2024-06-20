//! Abstractions and interfaces over the git repository

use color_eyre::{eyre::Context, Result};
use git2::{Repository, Signature};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Interface {
    #[allow(dead_code)] // Will be used later
    repo: Arc<Mutex<Repository>>,
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
    pub fn lazy_init() -> Result<Self> {
        let mut doc_path = PathBuf::from("./repo");
        doc_path.push(env::var("DOC_PATH").unwrap_or_else(|_| {
            warn!("The `DOC_PATH` environment variable was not set, defaulting to `docs/`");
            "docs".to_string()
        }));
        if let Ok(repo) = Repository::open("./repo") {
            info!("Existing repository detected, fetching latest changes...");
            let mut remote = repo.find_remote("origin")?;
            remote.fetch(&["master"], None, None)?;
            // Stuff with C bindings will sometimes require manual dropping if
            // there's references and stuff
            drop(remote);
            info!("Successfully fetched latest changes");
            return Ok(Self {
                repo: Arc::new(Mutex::new(repo)),
                doc_path,
            });
        }

        let repository_url = env::var("REPO_URL")
            .wrap_err("The `REPO_URL` environment url was not set, this is required.")?;
        let output_path = Path::new("./repo");
        info!(
            "No repo detected, cloning {repository_url:?} into {:?}...",
            output_path.display()
        );
        // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation#about-authentication-as-a-github-app-installation
        // TODO
        let repo = Repository::clone(&repository_url, "./repo")?;
        info!("Successfully cloned repo");
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
    pub fn get_doc<P: AsRef<Path>>(&self, path: P) -> Result<Option<String>> {
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

    /// Replace the document at the provided path
    /// (relative to the root of the documents folder) with a new document
    /// # Panics
    /// This function will panic if it's called when the repo mutex is already held by the current thread
    ///
    /// # Errors
    /// This function will return an error if filesystem operations fail, or if any of the git operations fail
    // This lint gets upset that `repo` isn't dropped early because it's a performance heavy drop, but when applied,
    // it creates errors that note the destructor for other values failing because of it (tree)
    #[allow(clippy::significant_drop_tightening)]
    pub fn put_doc<P: AsRef<Path> + Copy>(
        &self,
        path: P,
        new_doc: &str,
        message: &str,
        token: &str,
    ) -> Result<()> {
        let repo = self.repo.lock().unwrap();
        let mut path_to_doc: PathBuf = PathBuf::from(".");
        path_to_doc.push(&self.doc_path);
        path_to_doc.push(path);
        // wipe the file
        let mut file = fs::File::create(path_to_doc).wrap_err_with(|| {
            format!(
                "Failed to wipe requested file for rewrite: {:?}",
                path.as_ref()
            )
        })?;
        // write the new contents in
        file.write_all(new_doc.as_bytes()).wrap_err_with(|| {
            format!(
                "Failed to write new contents into file: {:?}",
                path.as_ref()
            )
        })?;
        let sig = Signature::now("Hyde", "hyde")?;
        let msg = format!("[CMS]: {message}");
        // adapted from https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html
        let mut index = repo.index()?;
        // File paths are relative to the root of the repository for `add_path`
        let mut relative_path = PathBuf::from(
            env::var("DOC_PATH").wrap_err("The `DOC_PATH` environment variable was not set")?,
        );
        let tree = {
            // Standard practice is to stage commits by adding them to an index.
            relative_path.push(path);
            index.add_path(&relative_path)?;
            let oid = index.write_tree()?;
            repo.find_tree(oid)?
        };
        let parent_commit = find_last_commit(&repo)?;
        // TODO: parent commit, staging?
        let commit_id = repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&parent_commit])?;
        debug!("New commit made with ID: {:?}", commit_id);
        // assuming github
        let repository_url = env::var("REPO_URL").wrap_err("Repo url not set in env")?;
        let authenticated_url =
            repository_url.replace("https://", &format!("https://x-access-token:{token}@"));
        repo.remote_set_pushurl("origin", Some(&authenticated_url))?;
        let mut remote = repo.find_remote("origin")?;
        remote.connect(git2::Direction::Push)?;
        // Push master here, to master there
        remote.push(&["refs/heads/master:refs/heads/master"], None)?;
        info!(
            "Document {:?} edited and pushed to GitHub with message: {message:?}",
            path.as_ref()
        );
        remote.disconnect()?;
        index.remove_path(&relative_path)?;
        index.write()?;
        debug!("Commit cleanup completed");
        Ok(())
    }
}

/// This function is needed because a lot of git functionality (adding new commits, et cetera) requires knowing the latest commit.
///
/// <https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html>
fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}
