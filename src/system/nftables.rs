use std::process::Command;
use std::sync::Arc;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::NetworkChange;
use chrono::Local;

use anyhow::Result;

pub fn network_setup(proxy_port: u16, dns_port: u16, bus: Arc<Bus>) -> Result<()> {

    Command::new("nft")
        .args(["add", "table", "inet", "phantomrelay"])
        .status()?;
    bus.emit(NetworkChange { change: "created table phantomrelay".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

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
    bus.emit(NetworkChange { change: "created NAT rule".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

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
    bus.emit(NetworkChange { change: "blocked QUIC".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

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
    bus.emit(NetworkChange { change: "ignored connetions from localhost".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

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
    bus.emit(NetworkChange { change: "redirected DNS".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

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
    bus.emit(NetworkChange { change: "redirected TCP".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

    Ok(())
}

pub fn cleanup(bus: Arc<Bus>) -> Result<()> {
    Command::new("nft")
        .args(["delete", "table", "inet", "phantomrelay"])
        .status()?;
    bus.emit(NetworkChange { change: "deleted table phantomrelay".to_string(), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() })?;

    Ok(())
}
