use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    /// The CAN interface to connect to (e.g. 'can0' or 'vcan0').
    #[arg(short, long, default_value_t=String::from("can0"))]
    can_interface: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut varta = varta_easyblade::Varta::new(&args.can_interface);

    let varta_easyblades = varta.scan().await?;
    for varta_easyblade in &varta_easyblades {
        println!("{:?}", varta_easyblade);
        varta.read_device_error_history(varta_easyblade).await?;

        let cell_voltages = varta.read_cell_voltages(varta_easyblade).await?;
        for (index, v) in cell_voltages.iter().enumerate() {
            println!("cell {index:2}: {v:.3}");
        }
    }

    Ok(())
}
