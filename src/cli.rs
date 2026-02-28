use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "matrix-bridge-imessage")]
#[command(about = "A Matrix-iMessage bridge", long_about = None)]
pub struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "config/config.yaml")]
    pub config: String,
}

pub fn parse_args() -> Args {
    Args::parse()
}
