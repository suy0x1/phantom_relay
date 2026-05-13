#[derive(Debug)]
pub enum Protocol {
    ICMP,
    TCP,
    UDP,
    Unknown(u8),
}

impl From<u8> for Protocol {
    fn from(value: u8) -> Self {
        match value {
            1 => Protocol::ICMP,
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            other => Protocol::Unknown(other)
        }
    }
}