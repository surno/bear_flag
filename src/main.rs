//! Bear Flag API Service - Entry Point
//!
//! Axum web service that generates high-quality gay bear pride flags with smooth
//! color gradients and a centered bear paw overlay.

use bear_flag::{config::DEFAULT_BIND_ADDR, create_router};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = create_router();

    let addr: SocketAddr = DEFAULT_BIND_ADDR
        .parse()
        .expect("Failed to parse bind address");

    info!("Starting Bear Flag API server on {}", addr);
    info!("Available endpoints:");
    info!("  GET /flag - Generate bear pride flag");
    info!("  GET /health - Health check");
    info!("");
    info!("Example requests:");
    info!("  curl 'http://localhost:3000/flag?preset=desktop-4k' -o flag.png");
    info!("  curl 'http://localhost:3000/flag?width=1920&height=1080&format=jpeg' -o flag.jpg");
    info!(
        "  curl 'http://localhost:3000/flag?preset=iphone-14-pro-max&transparent=true' -o flag.png"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
