use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliOptions {
    #[arg(long, default_value = "127.0.0.1")]
    host: Ipv4Addr,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

pub async fn cli_main() -> Result<(), std::io::Error> {
    let args = CliOptions::parse();
    crate::server::serve_proxy(args.host, args.port).await
}
