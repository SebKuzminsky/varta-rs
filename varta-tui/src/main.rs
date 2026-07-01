use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    /// The CAN interface to connect to (e.g. 'can0' or 'vcan0').
    #[arg(short, long, default_value_t=String::from("can0"))]
    can_interface: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (tx, rx) = zencan_client::open_socketcan(&args.can_interface).expect(&format!(
        "Failed to open CAN interface {}",
        args.can_interface
    ));
    let mut varta = varta_easyblade::Varta::new(tx, rx);
    varta.scan().await;
}
