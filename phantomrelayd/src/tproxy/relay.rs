use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::Event::{ConnectionClosed, ConnectionOpened};
use crate::subsystems::rotation::route::RouteContext;
use anyhow::Result;
use chrono::Local;
use std::net::IpAddr;
use std::{sync::Arc, time::Instant};
use tokio::{io::copy_bidirectional, net::TcpStream};

use crate::{
    routing::{
        connect::connect_target,
        connection::{ConnectionKey, ProxyConnection},
        manager::ConnectionManager,
    },
    tproxy::original_dst::get_original_dst,
};

pub async fn handle_connection(
    current: RouteContext,
    mut client: TcpStream,
    map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
) -> Result<()> {
    let original = get_original_dst(&client).emit_to_bus(&bus)?;

    let key = ConnectionKey {
        dst_ip: original.ip().to_string().parse().emit_to_bus(&bus)?,
        dst_port: original.port(),
    };

    let conn = connect_target(current, &original.ip().to_string(), original.port(), bus.clone())
        .await
        .emit_to_bus(&bus)?;

    let proxy_used = ProxyConnection {
        started: Instant::now(),
        proxy: conn.host.clone(),
    };

    map.connections.insert(key.clone(), proxy_used);

    let mut remote = conn.stream;

    bus.emit(ConnectionOpened {
        host: IpAddr::V4(*original.ip()),
        port: original.port(),
        proxy: IpAddr::V4(conn.host.parse().emit_to_bus(&bus)?),
        proxy_port: conn.port,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;
    copy_bidirectional(&mut client, &mut remote).await.emit_to_bus(&bus)?;
    bus.emit(ConnectionClosed {
        host: IpAddr::V4(*original.ip()),
        port: original.port(),
        proxy: IpAddr::V4(conn.host.parse().emit_to_bus(&bus)?),
        proxy_port: conn.port,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;
    map.connections.remove(&key);
    Ok(())
}
