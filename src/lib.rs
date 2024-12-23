mod proxy;
mod server;

use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: Ipv4Addr,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

pub async fn cli_main() -> Result<(), ()> {
    let args = Cli::parse();
    server::serve_proxy(args.host, args.port).await
}
