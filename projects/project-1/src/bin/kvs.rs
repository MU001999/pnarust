use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
enum Config {
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

fn main() {
    let config = Config::from_args();

    match config {
        Config::Set { .. } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Config::Get { .. } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Config::Rm { .. } => {
            eprintln!("unimplemented");
            exit(1);
        }
    };
}
