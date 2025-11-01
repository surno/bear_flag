//! Tokio/Axum server bootstrap utilities.

use std::net::SocketAddr;

use tracing::info;

use crate::{app::create_router, error::ServerError};

/// Default server bind address.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";

/// Runs the HTTP server using the default bind address.
///
/// # Errors
///
/// Propagates [`ServerError`] if the bind address is invalid, binding fails, or the
/// Axum server returns an error.
pub async fn run() -> Result<(), ServerError> {
    serve(DEFAULT_BIND_ADDR).await
}

/// Runs the HTTP server on the provided address string.
///
/// # Errors
///
/// Propagates [`ServerError`] if the bind address is invalid, binding fails, or the
/// Axum server returns an error.
pub async fn serve(address: &str) -> Result<(), ServerError> {
    let addr = address
        .parse()
        .map_err(|source| ServerError::BindAddressParse {
            address: address.to_string(),
            source,
        })?;

    serve_socket(addr).await
}

/// Runs the HTTP server on the provided socket address.
///
/// # Errors
///
/// Returns [`ServerError::Bind`] if the listener cannot be created, or
/// [`ServerError::Serve`] if the Axum server exits with an error.
pub async fn serve_socket(addr: SocketAddr) -> Result<(), ServerError> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind {
            address: addr,
            source,
        })?;

    let actual_addr = listener.local_addr().unwrap_or(addr);

    serve_listener(listener, actual_addr).await
}

async fn serve_listener(
    listener: tokio::net::TcpListener,
    actual_addr: SocketAddr,
) -> Result<(), ServerError> {

    info!("Starting Bear Flag API server on {actual_addr}");
    info!("Available endpoints:");
    info!("  GET /flag - Generate bear pride flag");
    info!("  GET /health - Health check");
    info!("");
    info!("Example requests:");
    info!("  curl 'http://{actual_addr}/flag?preset=desktop-4k' -o flag.png");
    info!("  curl 'http://{actual_addr}/flag?width=1920&height=1080&format=jpeg' -o flag.jpg");
    info!(
        "  curl 'http://{actual_addr}/flag?preset=iphone-14-pro-max&transparent=true' -o flag.png"
    );

    let app = create_router();
    axum::serve(listener, app)
        .await
        .map_err(|source| ServerError::Serve { source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::timeout;

    #[tokio::test]
    async fn serve_rejects_invalid_address() {
        let err = serve("invalid-address").await.unwrap_err();
        match err {
            ServerError::BindAddressParse { .. } => {}
            other => panic!("Expected bind address parse error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn serve_listener_handles_health_request() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let actual_addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move { serve_listener(listener, actual_addr).await });

        let mut stream = timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect(actual_addr),
        )
        .await
        .expect("connect timed out")
        .unwrap();

        let request = format!(
            "GET /health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            actual_addr
        );
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.unwrap();

        assert!(response.starts_with(b"HTTP/1.1 200 OK"));

        server.abort();
        let _ = server.await;
    }
}
