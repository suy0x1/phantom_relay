use anyhow::Result;

use std::{
    mem,
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::AsRawFd
};

use tokio::net::TcpStream;

pub fn get_original_dst(stream: &TcpStream) -> Result<SocketAddrV4> {
    let fd = stream.as_raw_fd();
    let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
    let mut len = mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

    let ret = unsafe{
        libc::getsockopt(
            fd,
            libc::SOL_IP,
            libc::SO_ORIGINAL_DST,
            &mut addr as *mut _ as *mut _,
            &mut len
        )
    };

    if ret != 0 {
        return  Err(std::io::Error::last_os_error().into())
    }

    Ok(SocketAddrV4::new(
        Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)), u16::from_be(addr.sin_port)
    ))

}   