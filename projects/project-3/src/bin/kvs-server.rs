use kvs::Result;
use structopt::StructOpt;
use slog::{info, slog_o};
use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder, Destination};

#[derive(StructOpt)]
#[structopt(name = "kvs-server",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(long = "addr", value_name = "IP-PORT", default_value = "127.0.0.1:4000")]
    addr: String,
    #[structopt(long = "engine", value_name = "ENGINE-NAME")]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);

    let logger = builder.build()?;

    let config = Config::from_args();

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(logger, "IP-PORT: {}, ENGINE: {}", &config.addr, config.engine.clone().unwrap_or(String::from("kvs")));

    let engine = if config.engine.is_some() {

    } else {

    };

    Ok(())
}
