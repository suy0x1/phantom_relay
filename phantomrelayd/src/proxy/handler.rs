use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use chrono::Local;

use fast_socks5::{Socks5Command, server::Socks5ServerProtocol};

use crate::monitor::bus::Bus;
use crate::routing::connect::connect_target;
use crate::routing::manager::ConnectionManager;
use crate::routing::connection::{ConnectionKey, ProxyConnection};
use crate::monitor::events::Event::{ConnectionOpened, ConnectionClosed};
use crate::subsystems::rotation::route::RouteContext;
use std::time::Instant;

pub async fn handle_client(
    current: RouteContext,
    stream: TcpStream,
    map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
) -> Result<()> {
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(stream)
        .await?
        .read_command()
        .await?;

    if cmd != Socks5Command::TCPConnect {
        return Ok(());
    }

    let addr: SocketAddr = target_addr.to_string().parse()?;

    let key = ConnectionKey {
        dst_ip: addr.ip().to_string().parse()?,
        dst_port: addr.port(),
    };

    let conn = connect_target(current, &addr.ip().to_string(), addr.port()).await?;

    let proxy_used = ProxyConnection {
        started: Instant::now(),
        proxy: conn.host.clone(),
    };

    map.connections.insert(key.clone(), proxy_used);

    let mut outbound = conn.stream;

    bus.emit(ConnectionOpened {
        host: addr.ip(),
        port: addr.port(),
        proxy: conn.host.parse()?,
        proxy_port: conn.port,
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    })?;

    let mut client = proto.reply_success(outbound.local_addr()?).await?;

    copy_bidirectional(&mut client, &mut outbound).await?;

    bus.emit(ConnectionClosed {
        host: addr.ip(),
        port: addr.port(),
        proxy: conn.host.parse()?,
        proxy_port: conn.port,
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    })?;

    map.connections.remove(&key);

    Ok(())
}
