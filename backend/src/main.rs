#![warn(clippy::all, clippy::nursery, clippy::cargo)]
// While it would be ideal if this wasn't an issue, we don't have the dev team to do this
#![allow(clippy::multiple_crate_versions)]
// A lot of database methods have been preemptively implemented
#[allow(dead_code)]
mod db;
mod gh;
pub mod git;
mod handlers_prelude;
pub mod perms;

use axum::{
    http::HeaderValue,
    routing::{delete, get, put},
    Router,
};
use clap::{
    builder::{PossibleValuesParser, TypedValueParser},
    Parser,
};
use color_eyre::eyre::Context;
use color_eyre::Result;
use db::Database;
use gh::GithubAccessToken;
use handlers_prelude::*;
#[cfg(target_family = "unix")]
use log::error;
use log::{debug, info, warn, LevelFilter};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use reqwest::{
    header::{ACCEPT, ALLOW, CONTENT_TYPE},
    Client, Method,
};
use std::env::{self, current_exe};
#[cfg(target_family = "unix")]
use tokio::signal::unix::{signal, SignalKind};

use tokio::task;
use tower_http::cors::CorsLayer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};

/// Global app state passed to handlers by axum
#[derive(Clone)]
struct AppState {
    git: git::Interface,
    oauth: BasicClient,
    reqwest_client: Client,
    gh_credentials: GithubAccessToken,
    db: Database,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, help = "The port the application listens on.", default_value_t = String::from("8080"))]
    port: String,
    #[arg(
        short = 'v',
        long = "verbosity",
        default_value_t = LevelFilter::Info,
        help = "The logging level for the server.",
        value_parser = PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<LevelFilter>().unwrap())
    )]
    logging_level: LevelFilter,
    #[arg(
        short,
        long,
        help = "A list of config options as key value pairs supplied by passing this flag multiple times or providing a comma delimited list. \
        This will set supplied config options as environment variables.",
        value_parser = parse_key_val,
        value_delimiter = ','
    )]
    cfg: Vec<(String, String)>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Parse command line arguments
    let cli_args = Args::parse();
    // Read environment variables from dotenv file
    let dotenv_path = "hyde-data/.env";
    // Load in any config settings passed by cli
    for (key, value) in cli_args.cfg {
        env::set_var(key, value);
    }
    // Initialize logging
    env_logger::builder()
        .filter(None, cli_args.logging_level)
        .init();
    debug!("Initialized logging");
    dotenvy::from_path(dotenv_path).unwrap_or_else(|_| {
        warn!("Failed to read dotenv file located at {dotenv_path}, please ensure all config values are manually set");
    });
    if cfg!(debug_assertions) {
        info!("Server running in development mode");
    } else {
        info!("Server running in release mode");
    }
    // Initialize app state
    let state: AppState = init_state()
        .await
        .wrap_err("Failed to initialize app state")?;
    debug!("Initialized app state");
    // https://github.com/r-Techsupport/hyde/issues/27
    // In docker, because the process is running with a PID of 1,
    // we need to implement our own SIGINT/TERM handlers
    #[cfg(target_family = "unix")]
    {
        debug!("Unix environment detected, starting custom interrupt handler");
        for sig in [SignalKind::interrupt(), SignalKind::terminate()] {
            task::spawn(async move {
                let mut listener = signal(sig).expect("Failed to initialize a signal handler");
                listener.recv().await;
                // At this point we've received SIGINT/SIGKILL and we can shut down
                error!("SIGINT or SIGTERM received, terminating.");
                std::process::exit(0);
            });
        }
    }

    // files are served relative to the location of the executable, not where the
    // executable was run from
    let mut frontend_dir = current_exe()?;
    // current_exe returns the path of the file, we need the dir the file is in
    frontend_dir.pop();
    frontend_dir.push("web");
    let asset_path = env::var("ASSET_PATH")
        .wrap_err("The `ASSET_PATH` environment variable was not set in the env")?;
    // Initialize the handler and router
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello world" }))
        .route("/api/doc", get(get_doc_handler))
        .route("/api/doc", put(put_doc_handler))
        .route("/api/tree", get(get_tree_handler))
        .route("/api/oauth", get(get_oauth2_handler))
        .route("/api/oauth/url", get(get_oauth2_url))
        .route("/api/users", get(get_users_handler))
        .route("/api/users/me", get(get_current_user_handler))
        .route(
            "/api/users/groups/:user_id",
            put(put_user_membership_handler),
        )
        .route(
            "/api/users/groups/:user_id",
            delete(delete_user_membership_handler),
        )
        .route("/api/users/:user_id", delete(delete_user_handler))
        .route("/api/users/me", delete(delete_current_user))
        .layer(
            // TODO: create a separate CORS layer for debug and release mode (credentials, allow origin)
            if cfg!(debug_assertions) {
                CorsLayer::new()
                    // If this isn't set, cookies won't be sent across ports
                    .allow_credentials(true)
                    .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
                    .allow_methods([Method::GET, Method::PUT])
                    .allow_headers([ALLOW, ACCEPT, CONTENT_TYPE])
            } else {
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::PUT])
                    .allow_headers([ALLOW, ACCEPT, CONTENT_TYPE])
            },
        )
        .with_state(state)
        // Serve the frontend files
        .nest_service("/", ServeDir::new(frontend_dir))
        // Serve the assets folder from the repo
        .nest_service(
            &format!("/{asset_path}"),
            ServeDir::new(format!("repo/{asset_path}")),
        )
        // Enable support for routes that have or don't have a trailing slash
        .layer(NormalizePathLayer::trim_trailing_slash());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli_args.port)).await?;
    info!("Application starting, listening on port {}", cli_args.port);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize an instance of [`AppState`]
async fn init_state() -> Result<AppState> {
    let git = task::spawn(async { git::Interface::lazy_init() });
    let oauth = {
        let client_id = env::var("OAUTH_CLIENT_ID").unwrap_or_else(|_| {
            warn!("The `OAUTH_CLIENT_ID` environment variable is not set, oauth functionality will be broken");
            String::new()
        });
        let client_secret = env::var("OAUTH_SECRET").unwrap_or_else(|_| {
            warn!("The `OAUTH_SECRET` environment variable is not set, oauth functionality will be broken");
            String::new()
        });
        // The oauth constructor does some url parsing on startup so these need valid urls
        let auth_url = env::var("OAUTH_URL").unwrap_or_else(|_| {
            warn!("The `OAUTH_URL` environment variable is not set, oauth functionality will be broken");
            String::from("https://example.com/")
        });
        let token_url = env::var("OAUTH_TOKEN_URL").unwrap_or_else(|_| {
            warn!("The `OAUTH_TOKEN_URL` environment variable is not set, oauth functionality will be broken");
            String::from("https://example.com/")
        });
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
        gh_credentials: GithubAccessToken::new(),
        db: Database::new().await?,
    })
}

/// Parse a single key-value pair for clap list parsing
///
/// https://github.com/clap-rs/clap_derive/blob/master/examples/keyvalue.rs
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
