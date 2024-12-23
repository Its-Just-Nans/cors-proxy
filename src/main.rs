//! # Main entry point for the application
//!
//! ```shell
//! cors-proxy
//! ```
//!
//! Options are available in [`crate::cli::CliOptions`]

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    cors_proxy::cli_main().await
}
