use clap::Parser;

use zencan_client::common::{
    traits::AsyncCanSender,
};
use zencan_client::open_socketcan;
use zencan_client::BusManager;

#[derive(Parser)]
struct Args {
    /// The CAN interface to connect to (e.g. 'can0' or 'vcan0')
    can_interface: String,
}

async fn scan<S: AsyncCanSender + Sync + Send>(manager: &mut BusManager<S>) {
    match manager.scan_nodes().await {
        Ok(nodes) => {
            for n in &nodes {
                println!("{n}");
            }
        }
        Err(e) => {
            println!("Error during scan: ");
            println!("{e}");
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (tx, rx) = open_socketcan(&args.can_interface).expect(&format!(
        "Failed to open CAN interface {}",
        args.can_interface
    ));
    let mut manager = BusManager::new(tx, rx);

    scan(&mut manager).await;
}
