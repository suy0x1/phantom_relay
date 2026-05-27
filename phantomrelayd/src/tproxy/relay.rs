use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::TelemetryEvent;
use crate::subsystems::rotation::route::RouteContext;
use anyhow::Result;
use std::net::IpAddr;
use std::time::SystemTime;
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

/// Handles a transparent proxy connection, retrieves original destination, bridges through proxy, and emits telemetry.
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

    let conn = connect_target(
        current,
        &original.ip().to_string(),
        original.port(),
        bus.clone(),
    )
    .await
    .emit_to_bus(&bus)?;

    let proxy_used = ProxyConnection {
        started: Instant::now(),
        proxy: conn.host.clone(),
    };

    map.connections.insert(key.clone(), proxy_used);

    let mut remote = conn.stream;

    bus.emit_telemetry(TelemetryEvent::ConnectionOpened {
        host: IpAddr::V4(*original.ip()),
        port: original.port(),
        proxy: IpAddr::V4(conn.host.parse().emit_to_bus(&bus)?),
        proxy_port: conn.port,
        timestamp: SystemTime::now(),
    })
    .await;
    copy_bidirectional(&mut client, &mut remote)
        .await
        .emit_to_bus(&bus)?;
    bus.emit_telemetry(TelemetryEvent::ConnectionClosed {
        host: IpAddr::V4(*original.ip()),
        port: original.port(),
        proxy: IpAddr::V4(conn.host.parse().emit_to_bus(&bus)?),
        proxy_port: conn.port,
        timestamp: SystemTime::now(),
    })
    .await;
    map.connections.remove(&key);
    Ok(())
}
