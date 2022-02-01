use kvs::{kvs_engine::*, thread_pool::*, KvsServer, Result};

use clap::{ArgEnum, Parser};
use slog::info;
use sloggers::{
    terminal::{Destination, TerminalLoggerBuilder},
    Build,
};
use std::{net::SocketAddr, path::Path, process::exit};

// `Config` is the type that represents the command-line arguments
#[derive(Parser)]
#[clap(name = "kvs-server",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[clap(
        long = "addr",
        value_name = "IP-PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,
    #[clap(arg_enum, long = "engine", value_name = "ENGINE-NAME")]
    engine: Option<EngineKind>,
}

// `EngineKind` is for the argument <ENGINE-NAME>
#[derive(ArgEnum, Clone, PartialEq, Eq)]
enum EngineKind {
    Kvs,
    Sled,
}

impl EngineKind {
    // translates the EngineKind to the corresponding str
    fn as_str(&self) -> &str {
        match self {
            EngineKind::Kvs => "kvs",
            EngineKind::Sled => "sled",
        }
    }
}

fn main() -> Result<()> {
    // builds the logger which logs to stderr
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);
    let logger = builder.build()?;

    // parses the command-line arguments and checks the engine
    let Config { addr, engine } = Config::parse();
    let engine = check_engine(engine);

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(logger, "IP-PORT: {}, ENGINE: {}", addr, engine.as_str());

    // creates the thread_pool, engine and server and then runs the server
    let thread_pool = SharedQueueThreadPool::new(num_cpus::get()).unwrap();
    match engine {
        EngineKind::Kvs => {
            let engine = KvStore::open("db.".to_owned() + engine.as_str())?;
            KvsServer::new(logger, addr, engine, thread_pool)?.run(None)?;
        }
        EngineKind::Sled => {
            let engine = SledKvsEngine::open("db.".to_owned() + engine.as_str())?;
            KvsServer::new(logger, addr, engine, thread_pool)?.run(None)?;
        }
    };

    Ok(())
}

// checks the input engine with selected engine if there has been a selected engine
fn check_engine(engine: Option<EngineKind>) -> EngineKind {
    // gets the existed engine
    let exist_engine = if Path::new("db.kvs").exists() {
        Some(EngineKind::Kvs)
    } else if Path::new("db.sled").exists() {
        Some(EngineKind::Sled)
    } else {
        None
    };

    // checks and returns the final valid engine
    match (engine, exist_engine) {
        (None, None) => EngineKind::Kvs,
        (Some(en1), Some(en2)) if en1 == en2 => en1,
        // prints to stderr and exits if the input engine is different from the selected engine
        (Some(_), Some(_)) => {
            eprintln!("data was previously persisted with a different engine than selected");
            exit(1);
        }
        (en1, en2) => en1.or(en2).unwrap(),
    }
}
