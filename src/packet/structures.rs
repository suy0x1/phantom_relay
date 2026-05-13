use std::net::Ipv4Addr;

use crate::packet::protocol::Protocol;

#[derive(Debug)]
pub struct Ipv4Header {
    pub version: u8,
    pub ihl: u8,
    pub ttl: u8,
    pub protocol: Protocol,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
}

#[derive(Debug)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub sequence: u32,
    pub acknowledgement: u32,
    pub flags: u16, 
}

#[derive(Debug)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
}

#[derive(Debug)]
pub struct IcmpHeader {
    pub icmp_type: u8,
    pub code: u8,
}

#[derive(Debug)]
pub enum TransportHeader {
    TCP(TcpHeader),
    UDP(UdpHeader),
    ICMP(IcmpHeader),
}