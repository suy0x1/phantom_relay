use anyhow::Result;
use std::io::Read;
use tun::Device;
use crate::packet::parser::{parse_ipv4,parse_icmp,parse_tcp,parse_udp};
use crate::packet::protocol::Protocol;

pub fn read_packets(mut dev: Device) -> Result<()> {
    let mut buf = [0u8; 1504];
    loop {
        let n = dev.read(&mut buf)?;
        
        let packet = &buf[..n];

        let ipv4 = match parse_ipv4(packet) {
            Ok(header) => header,
            Err(_) => continue,
        };

        let ip_header_len = (ipv4.ihl * 4) as usize;

        match ipv4.protocol {
            Protocol::TCP => {
                let tcp = parse_tcp(packet, ip_header_len)?;

                println!(
                    "TCP {}:{} -> {}:{}",
                    ipv4.src,
                    tcp.src_port,
                    ipv4.dst,
                    tcp.dst_port
                );
            }

            Protocol::UDP => {
                let udp = parse_udp(packet, ip_header_len)?;

                println!(
                    "UDP {}:{} -> {}:{}",
                    ipv4.src,
                    udp.src_port,
                    ipv4.dst,
                    udp.dst_port
                );
            }

            Protocol::ICMP => {
                let icmp = parse_icmp(packet, ip_header_len)?;

                println!(
                    "ICMP {} -> {} type={}",
                    ipv4.src,
                    ipv4.dst,
                    icmp.icmp_type
                );
            }

            Protocol::Unknown(_) => ()
        }
    }
}