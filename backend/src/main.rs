pub mod auth;
pub mod git;
mod handlers;

use auth::OAathClient;
use axum::{http::HeaderValue, routing::get, Router};
use clap::{
    builder::{PossibleValuesParser, TypedValueParser},
    Parser,
};
use color_eyre::eyre::Context;
use git::GitInterface;
use handlers::*;
use log::{info, LevelFilter};
use std::env::current_exe;
use tower_http::cors::CorsLayer;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};

/// Global app state passed to handlers by axum
#[derive(Clone)]
struct AppState {
    git: GitInterface,
    oauth: OAathClient,
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
async fn main() -> color_eyre::Result<()> {
    // Parse command line arguments
    let cli_args = Args::parse();
    // Read environment variables from dotenv file
    dotenvy::dotenv().context("No dotenv file found, or failed to read from it")?;
    // Initialize logging
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    // Initialize app state
    let state: AppState = AppState {
        git: GitInterface::lazy_init()?,
        oauth: OAathClient::new()?,
    };

    println!("{:?}", state.oauth.get_auth_url().0.as_str());
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
