use std::process::exit;
use kvs::{Command, Response, KvsClient, Result};
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "kvs-client",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub enum Config {
    Set {
        key: String,
        value: String,
        #[structopt(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: String,
    },
    Get {
        key: String,
        #[structopt(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: String,
    },
    Rm {
        key: String,
        #[structopt(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: String,
    },
}

impl Config {
    fn to_command(self) -> Command {
        match self {
            Config::Set { key, value, .. } => Command::Set { key, value },
            Config::Get { key, .. } => Command::Get {key},
            Config::Rm { key, .. } => Command::Rm{key},
        }
    }

    fn addr(&self) -> String {
        match self {
            Config::Set { key: _, value: _, addr } => addr.clone(),
            Config::Get { key: _, addr } => addr.clone(),
            Config::Rm { key: _, addr } => addr.clone(),
        }
    }
}

fn main() -> Result<()> {
    let config = Config::from_args();
    let addr: SocketAddr = config.addr().parse().expect("IP-PORT does not parse as an address");

    let mut client = KvsClient::connect(addr)?;
    match client.send(config.to_command())? {
        Response::Fail(msg) => {
            eprintln!("{}", msg);
            exit(1);
        }
        Response::SuccessGet(value) => {
            match value {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        _ => ()
    }

    Ok(())
}
