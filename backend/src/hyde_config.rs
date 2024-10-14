use std::fs;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct HydeConfig {
    pub files: Files,
    pub discord: Discord,
    pub oauth: OAuth,
    pub database: Database,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Files {
    pub asset_path: String,
    pub docs_path: String,
    pub repo_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Discord {
    pub admin_username: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OAuth {
    pub discord: DiscordOAuth,
    pub github: GitHubOAuth,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscordOAuth {
    pub client_id: String,
    pub secret: String,
    pub url: String,
    pub token_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubOAuth {
    pub client_id: String,
    // Uncomment this if needed
    // pub secret: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Database {
    pub url: String,
}

impl HydeConfig {
    pub fn load() -> Arc<Self> {
        let file = fs::read_to_string("default.toml").expect("Unable to read config");
        let config: Self = toml::from_str(&file).expect("Unable to parse config");
        Arc::new(config)
    }

    pub fn check() {}
}
