mod handlers;

use axum::{routing::get, Router};
use log::info;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir};
use std::env::current_exe;

// TODO: configure basic clap set up for command line config, you can
// use hwapi as a reference.

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
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

    let port = "3000";
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("Application starting, listening on port: {port}");
    axum::serve(listener, app).await?;

    Ok(())
}
