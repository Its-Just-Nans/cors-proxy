#[tokio::main]
async fn main() -> Result<(), ()> {
    cors_proxy::cli_main().await
}
