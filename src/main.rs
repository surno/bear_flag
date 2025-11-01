//! Pride Flag API Service
//!
//! Axum web service that generates high-quality pride flags with smooth color gradients.
//! Supports multiple pride flag presets (rainbow, bear, bisexual, transgender, pansexual,
//! lesbian, asexual, nonbinary, progress) or custom colors and stripe counts. Bear flags
//! include an optional centered bear paw overlay with proper alpha compositing.

mod constants;
mod flag;
mod handlers;
mod rendering;
mod router;
mod types;

use crate::constants::DEFAULT_BIND_ADDR;
use crate::router::create_router;
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

    info!("Starting Pride Flag API server on {}", addr);
    info!("Available endpoints:");
    info!("  GET /flag - Generate pride flags");
    info!("  GET /health - Health check");
    info!("");
    info!("Example requests:");
    info!("  curl 'http://localhost:3000/flag?pride=rainbow&preset=desktop-4k' -o flag.png");
    info!("  curl 'http://localhost:3000/flag?pride=transgender&width=1920&height=1080' -o flag.png");
    info!("  curl 'http://localhost:3000/flag?colors=FF0000,00FF00,0000FF&stripe_count=6' -o flag.png");
    info!(
        "  curl 'http://localhost:3000/flag?pride=bear&preset=iphone-14-pro-max&transparent=true' -o flag.png"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
