use clap::Parser;
use opencuboids_common::DEFAULT_PORT;

/// Cli for the opencuboids server
#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// The port of the server to run on
    #[clap(short, long, value_parser, default_value_t = DEFAULT_PORT)]
    port: u16,
}

fn main() {
    opencuboids_common::log_setup();

    let args = Args::parse();
    let address = format!("0.0.0.0:{}", args.port).parse().unwrap();
    opencuboids_server::start(address);
}
