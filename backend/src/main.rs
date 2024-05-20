mod handlers;

use axum::{routing::get, Router};
use log::info;
use tower_http::normalize_path::NormalizePathLayer;

// TODO: configure basic clap set up for command line config, you can
// use hwapi as a reference.

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    // Initialize the handler and router
    let app = Router::new()
        .route("/hello", get(|| async { "Hello world" }))
        // Enable support for routes that have or don't have a trailing slash
        .layer(NormalizePathLayer::trim_trailing_slash());

    let port = "3000";
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("Application starting, listening on port: {port}");
    axum::serve(listener, app).await?;

    Ok(())
}
