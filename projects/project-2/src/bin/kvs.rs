use std::process::exit;
use structopt::StructOpt;
use kvs::Result;

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
        _key: String,
        #[structopt(value_name = "VALUE")]
        _value: String,
    },
    Get {
        #[structopt(value_name = "KEY")]
        _key: String,
    },
    Rm {
        #[structopt(value_name = "KEY")]
        _key: String,
    },
}

fn main() -> Result<()> {
    let config = Config::from_args();

    if let Some(cmd) = config.sub_cmd {
        match cmd {
            SubCommand::Set { .. } => {
                eprintln!("unimplemented");
                exit(1);
            },
            SubCommand::Get { .. } => {
                eprintln!("unimplemented");
                exit(1);
            },
            SubCommand::Rm { .. } => {
                eprintln!("unimplemented");
                exit(1);
            },
        }
    }

    panic!()
}
