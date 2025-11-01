//! Pride Flag API Service
//!
//! Axum web service that generates high-quality pride flags with smooth color gradients.
//! Supports various pride flag presets (rainbow, trans, bi, pan, lesbian, bear, etc.)
//! as well as custom colors and stripe counts. The bear flag includes an optional
//! bear paw overlay with proper alpha compositing for professional results.

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
    info!("  GET /flag - Generate pride flags with presets or custom colors");
    info!("  GET /health - Health check");
    info!("");
    info!("Example requests:");
    info!("  # Rainbow flag");
    info!("  curl 'http://localhost:3000/flag?flag=rainbow&preset=desktop-4k' -o rainbow.png");
    info!("  # Transgender flag");
    info!("  curl 'http://localhost:3000/flag?flag=trans&width=1920&height=1080' -o trans.png");
    info!("  # Bear flag with paw");
    info!("  curl 'http://localhost:3000/flag?flag=bear&preset=iphone-14-pro-max' -o bear.png");
    info!("  # Custom colors");
    info!("  curl 'http://localhost:3000/flag?colors=FF0000,00FF00,0000FF&width=800&height=600' -o custom.png");
    info!("");
    info!("Supported flags: rainbow, trans, bisexual, pansexual, lesbian, asexual,");
    info!("                 nonbinary, genderqueer, genderfluid, aromantic, agender,");
    info!("                 polyamory, bear, progress, and more");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
