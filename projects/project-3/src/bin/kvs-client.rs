use kvs::{Command, KvsClient, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "kvs-client",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(
        long = "addr",
        value_name = "IP-PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,
}

fn main() -> Result<()> {
    let Config { cmd, addr } = Config::from_args();

    let mut client = KvsClient::connect(addr)?;
    let res = client.send(cmd)?;

    println!("{}", res);

    Ok(())
}
