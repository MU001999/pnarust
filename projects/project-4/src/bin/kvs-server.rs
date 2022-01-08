use std::net::SocketAddr;
use std::path::Path;
use std::process::exit;

use kvs::{KvStore, KvsServer, Result, SledKvsEngine};
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

#[derive(PartialEq, Eq)]
enum EngineKind {
    Kvs,
    Sled,
}

impl EngineKind {
    fn as_str(&self) -> &str {
        match self {
            EngineKind::Kvs => "kvs",
            EngineKind::Sled => "sled",
        }
    }
}

fn main() -> Result<()> {
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);
    let logger = builder.build()?;

    let Config { addr, engine } = Config::from_args();

    let addr: SocketAddr = addr.parse().expect("IP-PORT does not parse as an address");
    let engine = engine.map(|val| match val.as_str() {
        "kvs" => EngineKind::Kvs,
        "sled" => EngineKind::Sled,
        _ => {
            eprintln!("ENGINE-NAME is either 'kvs' or 'sled'");
            exit(1);
        }
    });

    let exist_engine = if Path::new("db.kvs").exists() {
        Some(EngineKind::Kvs)
    } else if Path::new("db.sled").exists() {
        Some(EngineKind::Sled)
    } else {
        None
    };

    let engine = match (engine, exist_engine) {
        (None, None) => EngineKind::Kvs,
        (Some(en1), Some(en2)) if en1 == en2 => en1,
        (Some(_), Some(_)) => {
            eprintln!("data was previously persisted with a different engine than selected");
            exit(1);
        }
        (en1, en2) => en1.or(en2).unwrap(),
    };

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(logger, "IP-PORT: {}, ENGINE: {}", addr, engine.as_str());

    match engine {
        EngineKind::Kvs => {
            let engine = KvStore::open("db.".to_owned() + engine.as_str())?;
            KvsServer::new(&logger, engine, addr)?.run()?;
        }
        EngineKind::Sled => {
            todo!()
            // let mut engine = SledKvsEngine::open("db.".to_owned() + engine.as_str())?;
            // KvsServer::new(&logger, engine, addr)?.run()?;
        }
    };

    Ok(())
}
