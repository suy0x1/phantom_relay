use std::process::Command;

use anyhow::Result;

pub fn network_setup(proxy_port: u16, dns_port: u16) -> Result<()> {
    Command::new("nft")
        .args(["add", "table", "inet", "phantomrelay"])
        .status()?;

    Command::new("nft")
        .args([
            "add",
            "chain",
            "inet",
            "phantomrelay",
            "output",
            "{",
            "type",
            "nat",
            "hook",
            "output",
            "priority",
            "dstnat",
            ";",
            "}",
        ])
        .status()?;

    Command::new("nft")
        .args([
            "add",
            "rule",
            "inet",
            "phantomrelay",
            "output",
            "udp",
            "dport",
            "443",
            "reject",
        ])
        .status()?;

    Command::new("nft")
        .args([
            "add",
            "rule",
            "inet",
            "phantomrelay",
            "output",
            "ip",
            "daddr",
            "127.0.0.0/8",
            "return",
        ])
        .status()?;

    Command::new("nft")
        .args([
            "add",
            "rule",
            "inet",
            "phantomrelay",
            "output",
            "udp",
            "dport",
            "53",
            "redirect",
            "to",
            &format!(":{}", dns_port),
        ])
        .status()?;

    Command::new("nft")
        .args([
            "add",
            "rule",
            "inet",
            "phantomrelay",
            "output",
            "tcp",
            "dport",
            "!=",
            &proxy_port.to_string(),
            "redirect",
            "to",
            &format!(":{}", proxy_port),
        ])
        .status()?;

    Ok(())
}

pub fn cleanup() -> Result<()> {
    Command::new("nft")
        .args(["delete", "table", "inet", "phantomrelay"])
        .status()?;

    Ok(())
}
