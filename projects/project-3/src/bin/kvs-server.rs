use kvs::Result;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "kvs-server",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Config {
    #[structopt(long = "addr", value_name = "IP-PORT", default_value = "127.0.0.1:4000")]
    addr: String,
    #[structopt(long = "engine", value_name = "ENGINE-NAME")]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let config = Config::from_args();

    let engine = if config.engine.is_some() {

    } else {

    };

    Ok(())
}
