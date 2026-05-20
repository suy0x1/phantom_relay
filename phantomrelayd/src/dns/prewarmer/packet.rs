use rand::random;

pub fn build_dns_query(domain: &str, qtype: u16) -> Vec<u8> {
    let mut packet = Vec::with_capacity(512);
    packet.extend_from_slice(&random::<u16>().to_be_bytes());

    packet.extend_from_slice(&0x0100u16.to_be_bytes());

    packet.extend_from_slice(&1u16.to_be_bytes());

    packet.extend_from_slice(&0u16.to_be_bytes());

    packet.extend_from_slice(&0u16.to_be_bytes());

    packet.extend_from_slice(&0u16.to_be_bytes());

    for label in domain.split('.') {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0);

    packet.extend_from_slice(&qtype.to_be_bytes());

    packet.extend_from_slice(&1u16.to_be_bytes());

    packet
}
