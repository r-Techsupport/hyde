mod gh;
pub mod git;
mod handlers;

use axum::{http::HeaderValue, routing::get, Router};
use clap::{
    builder::{PossibleValuesParser, TypedValueParser},
    Parser,
};
use color_eyre::eyre::Context;
use color_eyre::Result;
use gh::GithubAccessToken;
use git::GitInterface;
use handlers::*;
use log::{info, LevelFilter};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use reqwest::Client;
use std::env::{self, current_exe};
use tokio::task;
use tower_http::cors::CorsLayer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};

/// Global app state passed to handlers by axum
#[derive(Clone)]
struct AppState {
    git: GitInterface,
    oauth: BasicClient,
    reqwest_client: Client,
    gh_credentials: GithubAccessToken,
}

#[derive(Parser)]
struct Args {
    #[arg(short = 'p', long = "port", default_value_t = String::from("8080"))]
    port: String,
    #[arg(
        short = 'v',
        long = "verbosity",
        default_value_t = LevelFilter::Info,
        value_parser = PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<LevelFilter>().unwrap())
    )]
    logging_level: LevelFilter,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli_args = Args::parse();
    // Read environment variables from dotenv file
    dotenvy::from_path("cms-data/.env")
        .context("No .env file was found in `cms-data/`, or failed to read from it.")?;
    // Initialize logging
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    // Initialize app state
    let mut state: AppState = init_state()
        .await
        .wrap_err("Failed to initialize app state")?;
    state.git.put_doc("backups/backups.md", "Hey mammi!".to_string(), "Git is weird".to_string(), state.gh_credentials.get(&state.reqwest_client).await?)?;
    // files are served relative to the location of the executable, not where the
    // executable was run from
    let mut frontend_dir = current_exe()?;
    // current_exe returns the path of the file, we need the dir the file is in
    frontend_dir.pop();
    frontend_dir.push("web");
    // Initialize the handler and router
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello world" }))
        .route("/api/doc", get(get_doc_handler))
        .route("/api/tree", get(get_tree_handler))
        .route("/api/oauth", get(get_oauth2_handler))
        .route("/api/oauth/url", get(get_oauth2_url))
        .layer(CorsLayer::new().allow_origin("*".parse::<HeaderValue>().unwrap()))
        .with_state(state)
        // Serve the frontend files
        .nest_service("/", ServeDir::new(frontend_dir))
        // Enable support for routes that have or don't have a trailing slash
        .layer(NormalizePathLayer::trim_trailing_slash());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli_args.port)).await?;
    info!("Application starting, listening on port {}", cli_args.port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn init_state() -> Result<AppState> {
    let git = task::spawn(async { GitInterface::lazy_init() });
    let oauth = {
        let client_id = env::var("OAUTH_CLIENT_ID").wrap_err("OAUTH_CLIENT_ID not set in env")?;
        let client_secret = env::var("OAUTH_SECRET").wrap_err("OAUTH_SECRET not sent in env")?;
        let auth_url = env::var("OAUTH_URL").wrap_err("OAUTH_URL not set in env")?;
        let token_url = env::var("OAUTH_TOKEN_URL").wrap_err("OAUTH_TOKEN_URL not set in env")?;
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url)?,
            Some(TokenUrl::new(token_url)?),
        )
    };
    let reqwest_client = Client::new();
    Ok(AppState {
        git: git.await??,
        oauth,
        reqwest_client,
        gh_credentials: GithubAccessToken::new()
    })
}
