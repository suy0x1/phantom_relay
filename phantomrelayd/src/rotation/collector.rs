use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub async fn get_proxy() -> Result<Vec<String>> {
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let res = client
    .get("https://api.proxyscrape.com/v4/free-proxy-list/get?request=display_proxies&proxy_format=protocolipport&format=text&protocol=socks5&anonymity=elite")
    .header("accept", "application/json")
    .send()
    .await?;
    let body = res.text().await?;
    let lines: Vec<&str> = body.lines().collect();
    let ret: Vec<String> = lines.into_iter().map(String::from).collect();
    Ok(ret)
}
