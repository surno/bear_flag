//! Tracing initialization utilities.

use tracing_subscriber::EnvFilter;

/// Initializes the global tracing subscriber using environment configuration.
///
/// The subscriber respects `RUST_LOG` if set and defaults to `info` level otherwise.
///
/// # Errors
///
/// Returns an error if a global subscriber has already been installed.
///
/// # Examples
///
/// ```
/// # fn main() {
/// let _ = bear_flag::telemetry::init_tracing();
/// # }
/// ```
pub fn init_tracing(
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_handles_multiple_calls() {
        let _ = init_tracing();
        let _ = init_tracing();
    }
}
