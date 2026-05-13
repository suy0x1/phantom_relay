use std::net::Ipv4Addr;
use anyhow::{Ok, Result, anyhow};

use crate::packet::structures::{Ipv4Header,TcpHeader,UdpHeader,IcmpHeader};
use crate::packet::protocol::Protocol;

pub fn parse_ipv4(packet: &[u8]) -> Result<Ipv4Header> {
    if packet.len() < 20 {
        return Err(anyhow!("packet too small"));    
    }

    let version = packet[0] >> 4;
    if version != 4 {
        return Err(anyhow!("not IPv4"))
    }
    let ihl = packet[0] & 0x0F;

    let ttl = packet[8];

    let protocol = Protocol::from(packet[9]);

    let src = Ipv4Addr::new(
        packet[12],
        packet[13],
        packet[14],
        packet[15]
    );

    let dst = Ipv4Addr::new(
        packet[16],
        packet[17],
        packet[18],
        packet[19]
    );

    Ok(Ipv4Header { version,ihl, ttl, protocol, src, dst })

}

pub fn parse_tcp(packet: &[u8], ip_header_len: usize) -> Result<TcpHeader> {
    let start = ip_header_len;
    if packet.len() < start + 20 {
        anyhow::bail!("tcp packet too small");
    }

    let src_port = u16::from_be_bytes([packet[start],packet[start + 1]]);
    let dst_port = u16::from_be_bytes([packet[start + 2],packet[start + 3]]);

    let sequence = u32::from_be_bytes([
        packet[start + 4],
        packet[start + 5],
        packet[start + 6],
        packet[start + 7],
    ]);
    let acknowledgement = u32::from_be_bytes([
        packet[start + 8],
        packet[start + 9],
        packet[start + 10],
        packet[start + 11],
    ]);

    let flags = u16::from_be_bytes([packet[start + 12], packet[start + 13]]) & 0x01FF;

    Ok(TcpHeader { src_port, dst_port, sequence, acknowledgement, flags })
}

pub fn parse_udp(packet: &[u8], ip_header_len: usize) -> Result<UdpHeader> {
    let start = ip_header_len;

    if packet.len() < start + 8 {
        anyhow::bail!("udp packet too small");
    }

    let src_port = u16::from_be_bytes([packet[start],packet[start + 1]]);
    let dst_port = u16::from_be_bytes([packet[start + 2],packet[start + 3]]);

    let length = u16::from_be_bytes([packet[start + 4],packet[start + 5]]);

    Ok(UdpHeader { src_port, dst_port, length })
}

pub fn parse_icmp(packet: &[u8], ip_header_len: usize) -> Result<IcmpHeader> {
    let start = ip_header_len;

    if packet.len() < start + 4 {
        anyhow::bail!("icmp packet too small");
    }

    Ok(IcmpHeader { icmp_type: packet[start], code: packet[start + 1] })
}