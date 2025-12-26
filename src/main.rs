use std::path::PathBuf;

use clap::Parser;

use crate::events::DeviceWrapper;
mod events;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    device: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let device = DeviceWrapper::try_open(args.device)?;
    device.listen();

    Ok(())
}
