use anyhow::Result;
use std::net::IpAddr;
use std::{sync::Arc, time::Instant};
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{ConnectionOpened, ConnectionClosed};
use chrono::Local;
use tokio::{io::copy_bidirectional, net::TcpStream};

use crate::{
    tproxy::original_dst::get_original_dst,
    routing::{
        connection::{ConnectionKey, ProxyConnection},
        manager::ConnectionManager,
        connect::connect_target,
    },
};

pub async fn handle_connection(mut client: TcpStream, map: Arc<ConnectionManager>, bus: Arc<Bus>) -> Result<()> {
    let original = get_original_dst(&client)?;

    let key = ConnectionKey {
        dst_ip: original.ip().to_string().parse()?,
        dst_port: original.port(),
    };

    let conn = connect_target(&original.ip().to_string(), original.port()).await?;
    
    let proxy_used = ProxyConnection {
        started: Instant::now(),
        proxy: conn.host.clone(),
    };

    map.connections.insert(key.clone(), proxy_used);
    

    let mut remote = conn.stream;
    
    bus.emit(ConnectionOpened { host: IpAddr::V4(*original.ip()), port: original.port(), proxy: IpAddr::V4(conn.host.parse()?), proxy_port: conn.port, timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;
    copy_bidirectional(&mut client, &mut remote).await?;
    bus.emit(ConnectionClosed { host: IpAddr::V4(*original.ip()), port: original.port(), proxy: IpAddr::V4(conn.host.parse()?), proxy_port: conn.port, timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;
    map.connections.remove(&key);
    Ok(())
}
