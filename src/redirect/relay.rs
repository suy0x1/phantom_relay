use anyhow::Result;
use std::{sync::Arc, time::Instant};

use tokio::{io::copy_bidirectional, net::TcpStream};

use crate::{
    redirect::original_dst::get_original_dst,
    relay::{
        connection::{ConnectionKey, ProxyConnection},
        manager::ConnectionManager,
        proxy::ProxyProvider,
        socks5::Socks5Proxy,
    },
};

pub async fn handle_connection(mut client: TcpStream, map: Arc<ConnectionManager>) -> Result<()> {
    let original = get_original_dst(&client)?;
    println!("original dst was {}", original);

    let proxy = Socks5Proxy {
        proxy_addr: "127.0.0.1:9050".to_string(),
    };

    let key = ConnectionKey {
        dst_ip: *original.ip(),
        dst_port: original.port(),
    };

    let proxy_used = ProxyConnection {
        started: Instant::now(),
        proxy: "127.0.0.1:9050".to_string(),
    };

    map.connections.insert(key.clone(), proxy_used);

    let mut remote = proxy
        .connect(&original.ip().to_string(), original.port())
        .await?;

    copy_bidirectional(&mut client, &mut remote).await?;

    map.connections.remove(&key);
    Ok(())
}
