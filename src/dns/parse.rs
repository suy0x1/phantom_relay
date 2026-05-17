pub fn extract_domain(
    packet: &[u8]
) -> Option<String> {

    if packet.len() < 13 {
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
            break;
        }

        pos += 1;

        if pos + len > packet.len() {
            return None;
        }

        let label =
            std::str::from_utf8(
                &packet[pos..pos + len]
            ).ok()?;

        labels.push(label);

        pos += len;
    }

    Some(labels.join("."))
}