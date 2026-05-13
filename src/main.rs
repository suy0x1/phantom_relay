use anyhow::Result;

use phantom_relay::tun::device::create_tun;
use phantom_relay::tun::reader::read_packets;

fn main() -> Result<()>{
    println!("Starting phantom-relay...");

    let dev = create_tun()?;

    println!("waiting for packets...");

    read_packets(dev)?;

    Ok(())

}
