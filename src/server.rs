use hyper::server::conn::http1;
use hyper_util::rt::tokio::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::proxy::ProxyService;

async fn serve(listener: TcpListener) {
    loop {
        // Accept a single connection
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
                break;
            }
        };

        tokio::task::spawn(async move {
            // Process the connection with our service
            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), ProxyService {})
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}

pub async fn serve_proxy(host: std::net::Ipv4Addr, port: u16) -> Result<(), ()> {
    let addr = SocketAddr::from((host, port));
    println!("Listening on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(_err) => {
            return Err(());
        }
    };

    let handle = tokio::spawn(async move { serve(listener).await });
    tokio::pin!(handle);
    // Run the server and await signals
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Server shutting down...");
                break;
            }
            _ = &mut handle => {
                continue;
            }
        }
    }
    Ok(())
}
