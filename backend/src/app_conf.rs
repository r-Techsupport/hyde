use color_eyre::eyre::ContextCompat;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::{fs, path::Path};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{info, trace};
use color_eyre::Result;

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct AppConf {
    pub files: Files,
    pub discord: Discord,
    pub oauth: OAuth,
    pub database: Database,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Files {
    pub asset_path: String,
    pub docs_path: String,
    pub repo_path: String,
    pub repo_url: String,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Discord {
    pub admin_username: String,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct OAuth {
    pub discord: DiscordOAuth,
    pub github: GitHubOAuth,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscordOAuth {
    pub client_id: String,
    pub secret: String,
    pub url: String,
    pub token_url: String,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct GitHubOAuth {
    pub client_id: String,
    // Uncomment this if needed
    // pub secret: String,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Database {
    pub url: String,
}

// Trait to validate fields in each struct
trait ValidateFields {
    fn validate(&self, path: &str) -> Result<(), String>;
}

// Macro to validate all fields for each struct
// TODO: Make it recognise if the type of the value supplied is also incorrect
macro_rules! impl_validate {
    ($struct_name:ident, $( $field:ident ),* ) => {
        impl ValidateFields for $struct_name {
            fn validate(&self, path: &str) -> Result<(), String> {
                $(
                    let field_path = format!("{}.{}", path, stringify!($field));
                    if self.$field.is_empty() {
                        return Err(format!("Field '{}' is empty", field_path));
                    }
                )*
                Ok(())
            }
        }
    };
}

impl_validate!(Files, asset_path, docs_path, repo_path, repo_url);
impl_validate!(Discord, admin_username);
impl_validate!(DiscordOAuth, client_id, secret, url, token_url);
impl_validate!(GitHubOAuth, client_id);
impl_validate!(Database, url);

impl ValidateFields for OAuth {
    fn validate(&self, path: &str) -> Result<(), String> {
        self.discord.validate(&format!("{}.discord", path))?;
        self.github.validate(&format!("{}.github", path))?;
        Ok(())
    }
}

impl ValidateFields for AppConf {
    fn validate(&self, path: &str) -> Result<(), String> {
        self.files.validate(&format!("{}.files", path))?;
        self.discord.validate(&format!("{}.discord", path))?;
        self.oauth.validate(&format!("{}.oauth", path))?;
        self.database.validate(&format!("{}.database", path))?;
        Ok(())
    }
}
impl AppConf {
    /// Deserializes the config located at `path`.
    /// 
    /// If a file is passed, it will load that file. If a directory is passed,
    /// then it'll search that directory for any `.toml` file.
    pub fn load<P: AsRef<Path> + Copy + Debug>(path: P) -> Result<Arc<Self>> {
        let file_metadata = fs::metadata(path)?;
        let config_path: PathBuf = if file_metadata.is_file() {
            path.as_ref().to_path_buf()
        } else {
            locate_config_file(path)?.wrap_err_with(|| format!("No config was found in the {path:?} directory"))?
        };
        let serialized_config = fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&serialized_config)?;
        trace!("Loaded config: {:#?}", config);

        config.validate("config").expect("Invalid config");

        Ok(Arc::new(config))
    }
}

/// Returns the first toml config file in the provided directory, relative to the executable.
fn locate_config_file<P: AsRef<Path> + Copy + Debug>(path: P) -> Result<Option<PathBuf>> {
    info!("Searching directory {path:?} for a config file");
    // Search the directory for a toml file
    let dir = fs::read_dir(path).expect("Failed to read path");
    for entry in dir {
        let entry = entry?;
        if entry.path().extension() == Some(OsStr::new("toml")) {
            info!("Using config at {:?}", entry.path());
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}
