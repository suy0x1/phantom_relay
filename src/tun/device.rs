use anyhow::Result;
use tun::{Configuration, Device};

pub fn create_tun() -> Result<Device> {
    let mut config = Configuration::default();

    config
    .address("10.0.0.1")
    .netmask("255.255.255.0")
    .up();
    
    let dev = tun::create(&config)?;
    
    println!("TUN device created successfully");

    Ok(dev)
}