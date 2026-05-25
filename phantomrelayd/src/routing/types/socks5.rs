use crate::routing::proxy::ProxyProvider;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    os::fd::{AsRawFd, RawFd},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Socks5Proxy {
    pub proxy_addr: String,
}

const PROXY_MARK: u32 = 0x1;

fn set_mark(fd: RawFd, mark: u32) -> std::io::Result<()> {
    let mark = mark as libc::c_int;

    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_MARK,
            (&mark as *const libc::c_int).cast(),
            std::mem::size_of_val(&mark) as libc::socklen_t,
        )
    };

    if ret == -1 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}

fn connect_marked_socket(proxy_addr: &str, mark: u32) -> Result<std::net::TcpStream> {
    let proxy = proxy_addr
        .to_socket_addrs()?
        .find(|x| matches!(x, SocketAddr::V4(_)))
        .ok_or_else(|| anyhow!("proxy has no ipv4 address"))?;

    let sock = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    set_mark(sock.as_raw_fd(), mark)?;

    sock.connect(&SockAddr::from(proxy))?;

    let stream: std::net::TcpStream = sock.into();

    stream.set_nonblocking(true)?;

    Ok(stream)
}

async fn socks5_connect(stream: &mut TcpStream, host: &str, port: u16) -> Result<()> {
    stream.write_all(&[0x05, 0x01, 0x00]).await?;

    let mut auth_reply = [0u8; 2];

    stream.read_exact(&mut auth_reply).await?;

    if auth_reply != [0x05, 0x00] {
        return Err(anyhow!("proxy rejected no-auth socks5"));
    }

    let mut req = Vec::new();

    req.push(0x05);
    req.push(0x01);
    req.push(0x00);

    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&ip.octets());
    } else {
        let bytes = host.as_bytes();

        if bytes.len() > 255 {
            return Err(anyhow!("domain too long"));
        }

        req.push(0x03);
        req.push(bytes.len() as u8);
        req.extend_from_slice(bytes);
    }

    req.extend_from_slice(&port.to_be_bytes());

    stream.write_all(&req).await?;

    let mut head = [0u8; 4];

    stream.read_exact(&mut head).await?;

    if head[0] != 0x05 {
        return Err(anyhow!("invalid socks version"));
    }

    if head[1] != 0x00 {
        return Err(anyhow!("socks connect failed rep={:02x}", head[1]));
    }

    match head[3] {
        0x01 => {
            let mut skip = [0u8; 6];
            stream.read_exact(&mut skip).await?;
        }

        0x03 => {
            let mut len = [0u8; 1];

            stream.read_exact(&mut len).await?;

            let mut skip = vec![0u8; len[0] as usize + 2];

            stream.read_exact(&mut skip).await?;
        }

        0x04 => {
            return Err(anyhow!("ipv6 upstream not supported"));
        }

        _ => {
            return Err(anyhow!("invalid atyp"));
        }
    }

    Ok(())
}

#[async_trait]
impl ProxyProvider for Socks5Proxy {
    async fn connect(&self, host: &str, port: u16) -> Result<TcpStream> {
        if let Ok(ip) = host.parse::<IpAddr>() {
            if matches!(ip, IpAddr::V6(_)) {
                return Err(anyhow!("ipv6 upstream not supported"));
            }
        }

        let proxy_addr = self.proxy_addr.clone();

        let std_stream =
            tokio::task::spawn_blocking(move || connect_marked_socket(&proxy_addr, PROXY_MARK))
                .await??;

        let mut stream = TcpStream::from_std(std_stream)?;

        socks5_connect(&mut stream, host, port).await?;

        Ok(stream)
    }
}
