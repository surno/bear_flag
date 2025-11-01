use std::process;

use tracing::error;

/// Application entry point.
#[tokio::main]
async fn main() {
    if let Err(err) = bear_flag::telemetry::init_tracing() {
        eprintln!("Failed to initialize tracing subscriber: {err}");
    }

    if let Err(err) = bear_flag::server::run().await {
        error!(error = ?err, "Server terminated with error");
        process::exit(1);
    }
}
