use crate::monitor::bus::Bus;
use crate::monitor::events::DiagnosticEvent;
use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Fetches SOCKS5 proxies from public API with retry logic. Returns list of proxy addresses.
pub async fn get_proxy(bus: &Bus, cancel: CancellationToken) -> Result<Vec<String>> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    let url = "https://api.proxyscrape.com/v4/free-proxy-list/get?request=display_proxies&proxy_format=protocolipport&format=text&protocol=socks5&anonymity=elite";

    for attempt in 1..=3 {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err(anyhow::anyhow!("proxy fetch cancelled"));
            }

            res = client
                .get(url)
                .header("accept", "application/json")
                .send() => {

                match res {
                    Ok(res) => {
                        if let Ok(body) = res.text().await {
                            return Ok(body.lines().map(String::from).collect());
                        }
                    }

                    Err(e) => {
                        _ = bus.emit_diagnostic(DiagnosticEvent::Error {
                            err: format!("Fetch attempt {} failed: {:#?}", attempt, e),

                        });

                        if attempt < 3 {
                            tokio::select! {
                                _ = cancel.cancelled() => {
                                    return Err(anyhow::anyhow!("proxy fetch cancelled"));
                                }

                                _ = sleep(Duration::from_mins(2)) => {}
                            }
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Failed to fetch proxies after 3 attempts"))
}
