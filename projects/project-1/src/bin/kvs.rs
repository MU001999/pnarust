extern crate clap;
use std::process::exit;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("set")
                .arg(
                    Arg::with_name("key")
                        .value_name("KEY")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("value")
                        .value_name("VALUE")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("get").arg(
                Arg::with_name("key")
                    .value_name("KEY")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            SubCommand::with_name("rm").arg(
                Arg::with_name("key")
                    .value_name("KEY")
                    .required(true)
                    .index(1),
            ),
        )
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("set") {
        eprintln!("unimplemented");
        exit(1);
    } else if let Some(_matches) = matches.subcommand_matches("get") {
        eprintln!("unimplemented");
        exit(1);
    } else if let Some(_matches) = matches.subcommand_matches("rm") {
        eprintln!("unimplemented");
        exit(1);
    }

    exit(1);
}
