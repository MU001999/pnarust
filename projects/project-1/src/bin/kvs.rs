extern crate clap;
use std::process::exit;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("kvs")
        .version("0.1.0")
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

    if let Some(matches) = matches.subcommand_matches("set") {
        eprintln!("unimplemented");
        exit(1);
    } else if let Some(matches) = matches.subcommand_matches("get") {
        eprintln!("unimplemented");
        exit(1);
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        eprintln!("unimplemented");
        exit(1);
    }

    exit(1);
}
