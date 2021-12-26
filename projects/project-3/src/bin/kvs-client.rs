use std::process::exit;
use kvs::{Command, KvsEngine, KvStore, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "kvs-client",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(subcommand)]
    cmd: Option<Command>,
    #[structopt(long = "addr", value_name = "IP-PORT", default_value = "127.0.0.1:4000")]
    addr: String,
}

fn main() -> Result<()> {
    let config = Config::from_args();

    if let Some(cmd) = config.cmd {
        let mut kvstore = KvStore::open(".")?;

        match cmd {
            Command::Set { key, value } => {
                kvstore.set(key, value)?;
                return Ok(());
            }
            Command::Get { key } => {
                if let Some(value) = kvstore.get(key)? {
                    println!("{}", value);
                } else {
                    eprintln!("Key not found");
                }
                return Ok(());
            }
            Command::Rm { key } => {
                if let Err(err) = kvstore.remove(key) {
                    eprintln!("{}", err);
                    exit(1);
                }
                return Ok(());
            }
        }
    }

    panic!()
}
