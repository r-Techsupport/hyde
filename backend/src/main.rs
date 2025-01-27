#![warn(clippy::all, clippy::nursery, clippy::cargo)]
// While it would be ideal if this wasn't an issue, we don't have the dev team to do this
#![allow(clippy::multiple_crate_versions)]
// A lot of database methods have been preemptively implemented
mod app_conf;
#[allow(dead_code)]
mod db;
mod gh;
pub mod git;
mod handlers_prelude;
pub mod perms;

use axum::{
    extract::MatchedPath,
    http::{HeaderValue, Request},
    response::Response,
    Router,
};
use clap::{
    builder::{PossibleValuesParser, TypedValueParser},
    Parser,
};
use color_eyre::eyre::Context;
use color_eyre::Result;
use db::Database;
use gh::GitHubClient;
use handlers_prelude::*;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, TokenUrl,
};
use reqwest::{
    header::{ACCEPT, ALLOW, CONTENT_TYPE},
    Client, Method,
};
use std::env::current_exe;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{debug, info, info_span, warn};
use tracing::{Level, Span};

use crate::app_conf::AppConf;
use tokio::task;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};

static CONFIG: LazyLock<Arc<AppConf>> = LazyLock::new(|| {
    let args = Args::parse();
    AppConf::load(&args.cfg).expect("Failed to load configuration")
});

/// Global app state passed to handlers by axum
#[derive(Clone)]
pub struct AppState {
    pub config: &'static AppConf,
    git: git::Interface,
    oauth: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    reqwest_client: Client,
    gh_client: GitHubClient,
    db: Database,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, help = "The port the application listens on.", default_value_t = String::from("8080"))]
    port: String,
    #[arg(
        short = 'v',
        long = "verbosity",
        default_value_t = Level::INFO,
        help = "The logging level for the server.",
        value_parser = PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<Level>().unwrap())
    )]
    logging_level: Level,
    #[arg(
        short = 'c',
        long = "config",
        help = "Pass your own .toml config file to Hyde.",
        default_value_t = String::from("hyde-data/"),
    )]
    cfg: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Parse command line arguments
    let cli_args = Args::parse();
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(cli_args.logging_level)
        .without_time()
        .init();
    debug!("Initialized logging");

    if cfg!(debug_assertions) {
        info!(
            "Server running in development mode, version v{}",
            env!("CARGO_PKG_VERSION")
        );
    } else {
        info!(
            "Server running in release mode, version v{}",
            env!("CARGO_PKG_VERSION")
        );
    }

    // Initialize app and config
    let state: AppState = init_state(&cli_args)
        .await
        .wrap_err("Failed to initialize app state")?;

    debug!("Initialized app state");
    // https://github.com/r-Techsupport/hyde/issues/27
    // In docker, because the process is running with a PID of 1,
    // we need to implement our own SIGINT/TERM handlers
    #[cfg(target_family = "unix")]
    {
        use tokio::signal::unix::{signal, SignalKind};
        use tracing::error;
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

    start_server(state, cli_args).await?;
    Ok(())
}

/// Initialize an instance of [`AppState`]
#[tracing::instrument]
async fn init_state(cli_args: &Args) -> Result<AppState> {
    let repo_url = CONFIG.files.repo_url.clone();
    let repo_path = CONFIG.files.repo_path.clone();
    let docs_path = CONFIG.files.docs_path.clone();
    let asset_path = CONFIG.files.asset_path.clone();

    let git =
        task::spawn(async { git::Interface::new(repo_url, repo_path, docs_path, asset_path) })
            .await??;
    let reqwest_client = Client::new();

    let oauth = BasicClient::new(ClientId::new(CONFIG.oauth.discord.client_id.clone()))
        .set_client_secret(ClientSecret::new(CONFIG.oauth.discord.secret.clone()))
        .set_auth_uri(AuthUrl::new(CONFIG.oauth.discord.url.clone())?)
        .set_token_uri(TokenUrl::new(CONFIG.oauth.discord.token_url.clone())?);

    Ok(AppState {
        config: &CONFIG,
        git,
        oauth,
        reqwest_client: reqwest_client.clone(),
        gh_client: GitHubClient::new(
            CONFIG.files.repo_url.clone(),
            reqwest_client.clone(),
            CONFIG.oauth.github.client_id.clone(),
        ),
        db: Database::new().await?,
    })
}

async fn start_server(state: AppState, cli_args: Args) -> Result<()> {
    // files are served relative to the location of the executable, not where the
    // executable was run from
    let mut frontend_dir = current_exe()?;
    // current_exe returns the path of the file, we need the dir the file is in
    frontend_dir.pop();
    frontend_dir.push("web");
    let config = state.config;
    let asset_path = &config.files.asset_path;

    // Initialize the handler and router
    let api_routes = Router::new()
        .merge(create_oauth_route().await)
        .merge(create_user_route().await)
        .merge(create_group_route().await)
        .merge(create_logout_route().await)
        .merge(create_reclone_route().await)
        .merge(create_github_route().await)
        .merge(create_tree_route().await)
        .merge(github_routes().await);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(if cfg!(debug_assertions) {
            CorsLayer::new()
                // If this isn't set, cookies won't be sent across ports
                .allow_credentials(true)
                .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([ALLOW, ACCEPT, CONTENT_TYPE])
        } else {
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([ALLOW, ACCEPT, CONTENT_TYPE])
        })
        .with_state(state)
        // Serve the assets folder from the repo
        .nest_service(
            &format!("/{asset_path}"),
            ServeDir::new(format!("repo/{asset_path}")),
        )
        // Serve the frontend files
        .fallback_service(
            ServeDir::new(frontend_dir)
                .precompressed_br()
                .precompressed_gzip(),
        )
        // Enable support for routes that have or don't have a trailing slash
        .layer(NormalizePathLayer::trim_trailing_slash())
        // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path=matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    // I don't know if this is strictly needed, should be tested
                    // Commented out because it was creating duplicate logs for endpoints
                    // let _span = span.clone().entered();
                    // let latency_ms = format!("{}ms", latency.as_millis());
                    // info!(latency=%latency_ms, status=%response.status());
                }),
        );

    let address = if cfg!(debug_assertions) {
        format!("localhost:{}", cli_args.port)
    } else {
        format!("0.0.0.0:{}", cli_args.port)
    };
    let listener = tokio::net::TcpListener::bind(&address).await?;
    info!("Application starting, listening at {:?}", address);
    axum::serve(listener, app).await?;
    unreachable!();
}
