//! Abstractions and interfaces over the git repository

use color_eyre::{eyre::Context, Result};
use git2::Repository;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct GitInterface {
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

impl GitInterface {
    /// Clone the repository into `./repo`, or run `fetch` if an existing repo
    /// was detected
    pub fn lazy_init() -> Result<GitInterface> {
        let mut doc_path = PathBuf::from("./repo");
        doc_path.push(env::var("DOC_PATH").wrap_err("The DOC_PATH env var was not set")?);
        if let Ok(repo) = Repository::open("./repo") {
            info!("Existing repository detected, fetching latest changes...");
            let mut remote = repo.find_remote("origin")?;
            remote.fetch(&["main"], None, None)?;
            // Stuff with C bindings will sometimes require manual dropping if
            // there's references and stuff
            drop(remote);
            info!("Successfully fetched latest changes");
            return Ok(Self {
                repo: Arc::new(Mutex::new(repo)),
                doc_path,
            });
        }

        let repository_url = env::var("REPO_URL").wrap_err("Repo url not set in env")?;
        let ouput_path = Path::new("./repo");
        info!(
            "No repo detected, cloning {repository_url:?} into {:?}...",
            ouput_path.display()
        );
        let repo = Repository::clone(&repository_url, "./repo").unwrap();
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
    pub fn get_doc(&self, path: &str) -> Result<Option<String>> {
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
    pub fn get_doc_tree(&self) -> Result<INode> {
        let mut root_node = INode {
            name: String::from("documents"),
            children: Vec::new(),
        };

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
                    })
                }
            }
            Ok(())
        }
        recurse_tree(Path::new(&self.doc_path), &mut root_node)?;
        Ok(root_node)
    }
}
