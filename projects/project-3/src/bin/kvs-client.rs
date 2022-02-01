use kvs::{Command, KvsClient, Response, Result};
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

// `Config` is the type that represents the command-line arguments
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
    // gets the corresponding command from the Config
    fn into_command(self) -> Command {
        match self {
            Config::Set { key, value, .. } => Command::Set { key, value },
            Config::Get { key, .. } => Command::Get { key },
            Config::Rm { key, .. } => Command::Rm { key },
        }
    }

    // gets the server address from the Config
    fn addr(&self) -> &str {
        match self {
            Config::Set {
                key: _,
                value: _,
                addr,
            } => addr.as_str(),
            Config::Get { key: _, addr } => addr.as_str(),
            Config::Rm { key: _, addr } => addr.as_str(),
        }
    }
}

fn main() -> Result<()> {
    // parses the command-line arguments
    let config = Config::from_args();
    let addr: SocketAddr = config
        .addr()
        .parse()
        .expect("IP-PORT does not parse as an address");

    // creates a kvs client with input address
    let mut client = KvsClient::connect(addr)?;
    // sends the command to the kvs serevr
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
