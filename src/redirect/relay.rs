use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::{sync::Arc, time::Instant};
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{ConnectionOpened, ConnectionClosed};
use chrono::Local;
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

pub async fn handle_connection(mut client: TcpStream, map: Arc<ConnectionManager>, bus: Arc<Bus>) -> Result<()> {
    let original = get_original_dst(&client)?;

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
    
    bus.emit(ConnectionOpened { host: IpAddr::V4(*original.ip()), port: original.port(), proxy: IpAddr::V4(Ipv4Addr::new(127,0,0,1)), proxy_port: 9050, timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;
    copy_bidirectional(&mut client, &mut remote).await?;
    bus.emit(ConnectionClosed { host: IpAddr::V4(*original.ip()), port: original.port(), proxy: IpAddr::V4(Ipv4Addr::new(127,0,0,1)), proxy_port: 9050, timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;
    map.connections.remove(&key);
    Ok(())
}
