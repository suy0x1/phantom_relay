use std::{collections::HashMap, process::Command, sync::Arc};

use anyhow::{Result, anyhow};
use std::time::SystemTime;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    config::{dns::DNSConfig, tproxy::TProxyConfig},
    monitor::{bus::Bus, error_ext::BusErrorExt, events::CriticalEvent},
};

const TABLE: &str = "phantomrelay";
const CHAIN: &str = "output";

const RULE_QUIC: &str = "phantom_quic_block";
const RULE_LOCALHOST: &str = "phantom_localhost_bypass";
const RULE_DNS: &str = "phantom_dns_redirect";
const RULE_TCP: &str = "phantom_tcp_redirect";
const RULE_MARK_BYPASS: &str = "phantom_mark_bypass";

fn emit(bus: Arc<Bus>, change: &str) -> Result<()> {
    bus.emit_critical(CriticalEvent::NetworkChange {
        change: change.to_string(),
        timestamp: SystemTime::now(),
    })?;

    Ok(())
}

fn run_nft(bus: Arc<Bus>, args: &[&str]) -> Result<()> {
    let status = Command::new("nft").args(args).status().emit_to_bus(&bus)?;

    if !status.success() {
        return Err(anyhow!("nft command failed: {:?}", args)).emit_to_bus(&bus)?;
    }

    Ok(())
}

fn nft_output(bus: Arc<Bus>, args: &[&str]) -> Result<String> {
    let output = Command::new("nft").args(args).output().emit_to_bus(&bus)?;

    if !output.status.success() {
        return Err(anyhow!("nft command failed: {:?}", args)).emit_to_bus(&bus)?;
    }

    Ok(String::from_utf8(output.stdout).emit_to_bus(&bus)?)
}

fn get_rule_handles(bus: Arc<Bus>) -> Result<HashMap<String, u64>> {
    let output = nft_output(
        bus.clone(),
        &["--json", "-a", "list", "chain", "inet", TABLE, CHAIN],
    )?;

    let json: Value = serde_json::from_str(&output).emit_to_bus(&bus)?;

    let mut map = HashMap::new();

    let nftables = json["nftables"]
        .as_array()
        .ok_or_else(|| anyhow!("invalid nft json"))
        .emit_to_bus(&bus)?;

    for item in nftables {
        let Some(rule) = item.get("rule") else {
            continue;
        };

        let Some(handle) = rule.get("handle").and_then(|h| h.as_u64()) else {
            continue;
        };

        let Some(comment) = rule.get("comment").and_then(|c| c.as_str()) else {
            continue;
        };

        map.insert(comment.to_string(), handle);
    }

    Ok(map)
}

fn delete_rule_by_comment(bus: Arc<Bus>, comment: &str) -> Result<()> {
    let handles = get_rule_handles(bus.clone())?;

    let handle = handles
        .get(comment)
        .ok_or_else(|| anyhow!("rule not found: {}", comment))
        .emit_to_bus(&bus)?;

    run_nft(
        bus,
        &[
            "delete",
            "rule",
            "inet",
            TABLE,
            CHAIN,
            "handle",
            &handle.to_string(),
        ],
    )?;

    Ok(())
}

pub fn create_table(bus: Arc<Bus>) -> Result<()> {
    run_nft(bus.clone(), &["add", "table", "inet", TABLE])?;

    emit(bus, "created table phantomrelay")?;

    Ok(())
}

pub fn create_nat_chain(bus: Arc<Bus>) -> Result<()> {
    run_nft(
        bus.clone(),
        &[
            "add", "chain", "inet", TABLE, CHAIN, "{", "type", "nat", "hook", "output", "priority",
            "dstnat", ";", "}",
        ],
    )?;

    emit(bus, "created NAT chain")?;

    Ok(())
}

pub fn ensure_base_stack(bus: Arc<Bus>) -> Result<()> {
    create_table(bus.clone())?;
    create_nat_chain(bus.clone())?;

    Ok(())
}

pub fn block_quic(bus: Arc<Bus>) -> Result<()> {
    run_nft(
        bus.clone(),
        &[
            "add", "rule", "inet", TABLE, CHAIN, "udp", "dport", "443", "reject", "comment",
            RULE_QUIC,
        ],
    )?;

    emit(bus, "blocked QUIC")?;

    Ok(())
}

pub fn ignore_localhost(bus: Arc<Bus>) -> Result<()> {
    run_nft(
        bus.clone(),
        &[
            "add",
            "rule",
            "inet",
            TABLE,
            CHAIN,
            "ip",
            "daddr",
            "127.0.0.0/8",
            "return",
            "comment",
            RULE_LOCALHOST,
        ],
    )?;

    emit(bus, "ignored localhost traffic")?;

    Ok(())
}

pub fn mark_bypass(bus: Arc<Bus>) -> Result<()> {
    run_nft(
        bus.clone(),
        &[
            "add",
            "rule",
            "inet",
            TABLE,
            CHAIN,
            "meta",
            "mark",
            "0x1",
            "return",
            "comment",
            RULE_MARK_BYPASS,
        ],
    )?;

    emit(bus, "added mark bypass rule")?;

    Ok(())
}

pub fn remove_mark_bypass(bus: Arc<Bus>) -> Result<()> {
    delete_rule_by_comment(bus.clone(), RULE_MARK_BYPASS)?;

    emit(bus, "removed mark bypass rule")?;

    Ok(())
}

pub async fn redirect_dns(config: Arc<Mutex<DNSConfig>>, bus: Arc<Bus>) -> Result<()> {
    let port = config.lock().await.port;

    run_nft(
        bus.clone(),
        &[
            "add",
            "rule",
            "inet",
            TABLE,
            CHAIN,
            "udp",
            "dport",
            "53",
            "redirect",
            "to",
            &format!(":{}", port),
            "comment",
            RULE_DNS,
        ],
    )?;

    emit(bus, "redirected DNS")?;

    Ok(())
}

pub fn redirect_tcp(config: Arc<TProxyConfig>, bus: Arc<Bus>) -> Result<()> {
    run_nft(
        bus.clone(),
        &[
            "add",
            "rule",
            "inet",
            TABLE,
            CHAIN,
            "tcp",
            "dport",
            "!=",
            &config.port.to_string(),
            "redirect",
            "to",
            &format!(":{}", config.port),
            "comment",
            RULE_TCP,
        ],
    )?;

    emit(bus, "redirected TCP")?;

    Ok(())
}

pub fn unblock_quic(bus: Arc<Bus>) -> Result<()> {
    delete_rule_by_comment(bus.clone(), RULE_QUIC)?;

    emit(bus, "unblocked QUIC")?;

    Ok(())
}

pub fn remove_localhost_bypass(bus: Arc<Bus>) -> Result<()> {
    delete_rule_by_comment(bus.clone(), RULE_LOCALHOST)?;

    emit(bus, "removed localhost bypass")?;

    Ok(())
}

pub async fn remove_dns_redirect(_config: Arc<Mutex<DNSConfig>>, bus: Arc<Bus>) -> Result<()> {
    delete_rule_by_comment(bus.clone(), RULE_DNS)?;

    emit(bus, "removed DNS redirect")?;

    Ok(())
}

pub fn remove_tcp_redirect(_config: Arc<TProxyConfig>, bus: Arc<Bus>) -> Result<()> {
    delete_rule_by_comment(bus.clone(), RULE_TCP)?;

    emit(bus, "removed TCP redirect")?;

    Ok(())
}

pub fn remove_nat_chain(bus: Arc<Bus>) -> Result<()> {
    run_nft(bus.clone(), &["flush", "chain", "inet", TABLE, CHAIN])?;

    run_nft(bus.clone(), &["delete", "chain", "inet", TABLE, CHAIN])?;

    emit(bus, "removed NAT chain")?;

    Ok(())
}

pub fn remove_table(bus: Arc<Bus>) -> Result<()> {
    run_nft(bus.clone(), &["delete", "table", "inet", TABLE])?;

    emit(bus, "removed table phantomrelay")?;

    Ok(())
}
