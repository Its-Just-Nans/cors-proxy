use hyper::server::conn::http1;
use hyper_util::rt::tokio::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::proxy::ProxyService;

pub async fn serve_proxy(host: std::net::Ipv4Addr, port: u16) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from((host, port));
    println!("Listening on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            return Err(err);
        }
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Server shutting down...");
                drop(listener);
                break;
            }
            accepted = listener.accept() => {
                let stream = match accepted {
                    Ok((stream, _addr)) => stream,
                    Err(err) => {
                        eprintln!("Error accepting connection: {}", err);
                        continue;
                    }
                };
                let conn = http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), ProxyService);
                tokio::spawn(async move {
                    if let Err(err) = conn.await {
                        eprintln!("Error serving connection: {}", err);
                    }
                });
            }
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
}
