use std::net::SocketAddr;
use std::process::exit;

use kvs::{KvStore, KvsServer, Result};
use slog::info;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::Build;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "kvs-server",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(
        long = "addr",
        value_name = "IP-PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,
    #[structopt(long = "engine", value_name = "ENGINE-NAME")]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);

    let logger = builder.build()?;

    let Config { addr, engine } = Config::from_args();
    let addr: SocketAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("IP-PORT does not parse as an address");
            exit(1);
        }
    };

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(
        logger,
        "IP-PORT: {}, ENGINE: {}",
        &addr,
        engine.clone().unwrap_or_else(|| String::from("kvs"))
    );

    let engine = engine.unwrap_or_else(|| String::from("kvs"));
    let mut engine = if engine == "kvs" {
        KvStore::open(".")?
    } else if engine == "sled" {
        todo!()
    } else {
        panic!("ENGINE-NAME is invalid")
    };

    let mut server = KvsServer::new(&logger, &mut engine, addr)?;
    server.run()?;

    Ok(())
}
