use std::io::{self, Result};

use log::{error, info, warn};
use tokio::net::TcpListener;

static DEFAULT_ADDR: &str = "127.0.0.1";
static DEFAULT_PORT: &str = "23541"; // The Signal

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::init();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let addr = format!("{}:{}", DEFAULT_ADDR, DEFAULT_PORT);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to open tcp listener!");
    listener.set_ttl(60).unwrap_or_else(|e| error!("{e:?}"));

    info!("Listening on -> {}", addr);

    match listener.accept().await {
        Ok((stream, addr)) => {
            info!("Accepted new stream -> {addr}");

            loop {
                stream.readable().await.expect("Socket not readable...");

                let mut buf = [0; 4096];
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(bytes) => info!("Received bytes...\n{bytes}"),
                    Err(e) => match e.kind() {
                        io::ErrorKind::WouldBlock => continue,
                        _ => warn!("Unexpected behaviour occured on the TCP socket..."),
                    },
                }
            }
        }
        Err(e) => panic!("Failed to get client -> {e:?}"),
    }
}
