use structopt::StructOpt;
use kvs::{KvStore, Result};

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(subcommand)]
    sub_cmd: Option<SubCommand>,
}

#[derive(StructOpt)]
enum SubCommand {
    Set {
        #[structopt(value_name = "KEY")]
        key: String,
        #[structopt(value_name = "VALUE")]
        value: String,
    },
    Get {
        #[structopt(value_name = "KEY")]
        key: String,
    },
    Rm {
        #[structopt(value_name = "KEY")]
        key: String,
    },
}

fn main() -> Result<()> {
    let config = Config::from_args();

    let mut kvstore = KvStore::open("kvs.data")?;

    if let Some(cmd) = config.sub_cmd {
        match cmd {
            SubCommand::Set { key, value } => {
                kvstore.set(key, value)?;
                return Ok(());
            },
            SubCommand::Get { key} => {
                let result = kvstore.get(key)?;
                if let Some(value) = result {
                    println!("{}", value);
                } else {
                    println!("Key not found");
                }
                return Ok(());
            },
            SubCommand::Rm { key } => {
                kvstore.remove(key)?;
                return Ok(());
            },
        }
    }

    panic!()
}
