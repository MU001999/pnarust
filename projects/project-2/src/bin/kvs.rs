use std::process::exit;

use structopt::StructOpt;
use kvs::{KvStore, Result, Command};

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

fn main() -> Result<()> {
    let config = Config::from_args();

    if let Some(cmd) = config.cmd {
        let mut kvstore = KvStore::open(".")?;

        match cmd {
            Command::Set { key, value } => {
                kvstore.set(key, value)?;
                return Ok(());
            },
            Command::Get { key} => {
                if let Some(value) = kvstore.get(key)? {
                    println!("{}", value);
                } else {
                    println!("Key not found");
                }
                return Ok(());
            },
            Command::Rm { key } => {
                if kvstore.get(key.clone())?.is_some() {
                    kvstore.remove(key)?;
                    return Ok(());
                } else {
                    println!("Key not found");
                    exit(1);
                }
            },
        }
    }

    panic!()
}
