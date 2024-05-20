mod handlers;

use axum::{routing::get, Router};
use clap::{Parser, builder::{PossibleValuesParser, TypedValueParser}};
use log::{info, LevelFilter};
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};
use std::env::current_exe;

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
    logging_level: LevelFilter
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli_args = Args::parse();
    // Initialize logging
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    // files are served relative to the location of the executable, not where the 
    // executable was run from
    let mut frontend_dir = current_exe()?;
    // current_exe returns the path of the file, we need the dir the file is in
    frontend_dir.pop();
    frontend_dir.push("web");
    // Initialize the handler and router
    let app = Router::new()
        .route("/hello", get(|| async { "Hello world" }))
        // Serve the frontend files
        .nest_service("/", ServeDir::new(frontend_dir))
        // Enable support for routes that have or don't have a trailing slash
        .layer(NormalizePathLayer::trim_trailing_slash());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli_args.port)).await?;
    info!("Application starting, listening on port: {}", cli_args.port);
    axum::serve(listener, app).await?;

    Ok(())
}
