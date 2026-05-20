use crate::dns::cache::CacheKey;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn extract_cache_key(packet: &[u8]) -> Option<CacheKey> {
    if packet.len() < 17 {
        return None;
    }

    let mut pos = 12;

    let mut labels = Vec::new();

    loop {
        if pos >= packet.len() {
            return None;
        }

        let len = packet[pos] as usize;

        if len == 0 {
            pos += 1;
            break;
        }

        pos += 1;

        if pos + len > packet.len() {
            return None;
        }

        let label = std::str::from_utf8(&packet[pos..pos + len]).ok()?;

        labels.push(label);

        pos += len;
    }

    if pos + 4 > packet.len() {
        return None;
    }

    let qtype = u16::from_be_bytes([packet[pos], packet[pos + 1]]);

    let qclass = u16::from_be_bytes([packet[pos + 2], packet[pos + 3]]);

    Some(CacheKey {
        domain: labels.join("."),
        qtype,
        qclass,
    })
}

fn skip_name(packet: &[u8], pos: &mut usize) -> Option<()> {
    loop {
        let len = *packet.get(*pos)?;

        // compressed pointer
        if (len & 0xC0) == 0xC0 {
            *pos += 2;
            return Some(());
        }

        // end of name
        if len == 0 {
            *pos += 1;
            return Some(());
        }

        // normal label
        *pos += 1 + len as usize;
    }
}

pub fn extract_min_ttl(packet: &[u8]) -> Option<u32> {
    if packet.len() < 12 {
        return None;
    }

    let qdcount = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    let ancount = u16::from_be_bytes([packet[6], packet[7]]) as usize;

    let mut pos = 12;

    // skip questions
    for _ in 0..qdcount {
        skip_name(packet, &mut pos)?;

        // QTYPE + QCLASS
        pos += 4;

        if pos > packet.len() {
            return None;
        }
    }

    let mut min_ttl: Option<u32> = None;

    // parse answers
    for _ in 0..ancount {
        skip_name(packet, &mut pos)?;

        // TYPE + CLASS + TTL + RDLENGTH
        if pos + 10 > packet.len() {
            return None;
        }

        pos += 2; // TYPE
        pos += 2; // CLASS

        let ttl = u32::from_be_bytes([
            packet[pos],
            packet[pos + 1],
            packet[pos + 2],
            packet[pos + 3],
        ]);

        pos += 4;

        let ttl = ttl.clamp(30, 86400);

        min_ttl = Some(match min_ttl {
            Some(current) => current.min(ttl),
            None => ttl,
        });

        let rdlength = u16::from_be_bytes([packet[pos], packet[pos + 1]]) as usize;

        pos += 2;

        if pos + rdlength > packet.len() {
            return None;
        }

        pos += rdlength;
    }

    min_ttl
}

pub fn extract_ips(packet: &[u8]) -> Vec<IpAddr> {
    let mut ips = Vec::new();

    if packet.len() < 12 {
        return ips;
    }

    let qdcount = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    let ancount = u16::from_be_bytes([packet[6], packet[7]]) as usize;

    let mut pos = 12;

    // skip questions
    for _ in 0..qdcount {
        if skip_name(packet, &mut pos).is_none() {
            return ips;
        }

        pos += 4;

        if pos > packet.len() {
            return ips;
        }
    }

    // parse answers
    for _ in 0..ancount {
        if skip_name(packet, &mut pos).is_none() {
            return ips;
        }

        if pos + 10 > packet.len() {
            return ips;
        }

        let rr_type =
            u16::from_be_bytes([packet[pos], packet[pos + 1]]);

        pos += 2; // TYPE
        pos += 2; // CLASS
        pos += 4; // TTL

        let rdlength =
            u16::from_be_bytes([packet[pos], packet[pos + 1]]) as usize;

        pos += 2;

        if pos + rdlength > packet.len() {
            return ips;
        }

        match rr_type {
            // A
            1 if rdlength == 4 => {
                let ip = Ipv4Addr::new(
                    packet[pos],
                    packet[pos + 1],
                    packet[pos + 2],
                    packet[pos + 3],
                );

                ips.push(IpAddr::V4(ip));
            }

            // AAAA
            28 if rdlength == 16 => {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&packet[pos..pos + 16]);

                ips.push(IpAddr::V6(Ipv6Addr::from(octets)));
            }

            _ => {}
        }

        pos += rdlength;
    }

    ips
}