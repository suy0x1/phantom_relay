use crate::redirect::relay::handle_connection;
use crate::relay::manager::ConnectionManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn start_listener(conn_map: Arc<ConnectionManager>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9001").await?;
    println!("started TCP listener");

    loop {
        let (stream, addr) = listener.accept().await?;

        println!("redirected conn from {}", addr);
        let conn_map = conn_map.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, conn_map).await {
                println!("relay error {}", e);
            }
        });
    }
}

// sudo iptables -t nat -F
