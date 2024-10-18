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
mod app_conf;

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
use gh::GithubAccessToken;
use handlers_prelude::*;
#[cfg(target_family = "unix")]
use tracing::{debug, info, info_span, warn, error};
// use tracing_subscriber::filter::LevelFilter;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use reqwest::{
    header::{ACCEPT, ALLOW, CONTENT_TYPE},
    Client, Method,
};
use std::env::{self, current_exe};
use std::sync::Arc;
use std::time::Duration;
#[cfg(target_family = "unix")]
use tokio::signal::unix::{signal, SignalKind};
use tracing::{Level, Span};
use tracing_subscriber::fmt::format::FmtSpan;

use tokio::task;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};
use crate::app_conf::AppConf;

/// Global app state passed to handlers by axum
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConf>,
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
        default_value_t = Level::INFO,
        help = "The logging level for the server.",
        value_parser = PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<Level>().unwrap())
    )]
    logging_level: Level,
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
    // Load in any config settings passed by cli
    for (key, value) in &cli_args.cfg {
        env::set_var(key, value);
    }
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(cli_args.logging_level)
        .with_span_events(FmtSpan::CLOSE)
        .init();
    debug!("Initialized logging");

    if cfg!(debug_assertions) {
        info!("Server running in development mode, version v{}", env!("CARGO_PKG_VERSION"));
    } else {
        info!("Server running in release mode, version v{}", env!("CARGO_PKG_VERSION"));
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

    start_server(state, cli_args).await?;
    Ok(())
}

/// Initialize an instance of [`AppState`]
#[tracing::instrument]
async fn init_state() -> Result<AppState> {
    let config = AppConf::load();

    let repo_url = config.files.repo_url.clone();
    let repo_path = config.files.repo_path.clone();
    let docs_path = config.files.docs_path.clone();
    
    let git = task::spawn(async { git::Interface::new(repo_url, repo_path, docs_path)}).await??;
    let reqwest_client = Client::new();
    
    let oauth = BasicClient::new(
        ClientId::new(config.oauth.discord.client_id.clone()),
        Some(ClientSecret::new(config.oauth.discord.secret.clone())),
        AuthUrl::new(config.oauth.discord.url.clone())?,
        Some(TokenUrl::new(config.oauth.discord.token_url.clone())?),
    );

    Ok(AppState {
        config,
        git,
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

async fn start_server(state: AppState, cli_args: Args) -> Result<()> {
    // files are served relative to the location of the executable, not where the
    // executable was run from
    let mut frontend_dir = current_exe()?;
    // current_exe returns the path of the file, we need the dir the file is in
    frontend_dir.pop();
    frontend_dir.push("web");
    let config = Arc::clone(&state.config);
    let asset_path = &config.files.asset_path;

    // Initialize the handler and router
    let api_routes = Router::new()
        .merge(create_oauth_route().await)
        .merge(create_user_route().await)
        .merge(create_group_route().await)
        .merge(create_logout_route().await)
        .merge(create_reclone_route().await)
        .merge(create_github_route().await)
        .merge(create_doc_route().await)
        .merge(create_tree_route().await);

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
        .nest_service(
            "/",
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

    // `localhost` works on macos, but 0.0.0.0 breaks it, but 0.0.0.0 works everywhere but macos and in production
    // TODO: figure it out
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
