use clap::Parser;
use kvs::{Command, KvsClient, Response, Result};
use std::net::SocketAddr;
use std::process::exit;

#[derive(Parser)]
#[clap(name = "kvs-client",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub enum Config {
    Set {
        key: String,
        value: String,
        #[clap(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: SocketAddr,
    },
    Get {
        key: String,
        #[clap(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: SocketAddr,
    },
    Rm {
        key: String,
        #[clap(
            long = "addr",
            value_name = "IP-PORT",
            default_value = "127.0.0.1:4000"
        )]
        addr: SocketAddr,
    },
}

impl Config {
    fn into_command(self) -> Command {
        match self {
            Config::Set { key, value, .. } => Command::Set { key, value },
            Config::Get { key, .. } => Command::Get { key },
            Config::Rm { key, .. } => Command::Rm { key },
        }
    }

    fn addr(&self) -> &SocketAddr {
        match self {
            Config::Set {
                key: _,
                value: _,
                addr,
            } => addr,
            Config::Get { key: _, addr } => addr,
            Config::Rm { key: _, addr } => addr,
        }
    }
}

fn main() -> Result<()> {
    let config = Config::parse();

    let mut client = KvsClient::connect(*config.addr())?;
    match client.send(config.into_command())? {
        Response::Fail(msg) => {
            eprintln!("{}", msg);
            exit(1);
        }
        Response::SuccessGet(value) => match value {
            Some(value) => println!("{}", value),
            None => println!("Key not found"),
        },
        _ => (),
    }

    Ok(())
}
