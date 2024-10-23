use std::fs;
use std::process::exit;
use std::sync::Arc;
use serde::Deserialize;
use tracing::{info, error, trace};

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

    pub fn load(file: &str) -> Arc<Self> {
        info!("Loading configuration file: {:?}", file);

        if fs::metadata(file).is_err() {
            error!("Configuration file {:?} does not exist", file);
            exit(1)
        }

        let config: Self = toml::from_str(&fs::read_to_string(file).unwrap()).expect("Unable to parse config");

        trace!("Loaded config: {:#?}", config);

        config.validate("config").expect("Invalid config");
        
        Arc::new(config)
    }
}
