// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
extern crate clap;

use std::process;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("My kvs program")
        .version(env!("CARGO_PKG_VERSION"))
        .author("ping <ping@gmail.com>")
        .about("Does awesome things")
        .subcommands(vec![
            SubCommand::with_name("get")
                .about("get a value")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .help("get the value bind by this key")
                .arg(
                    Arg::with_name("KEY")
                        .help("the key to get the value")
                        .required(true)
                        .index(1),
                ),
            SubCommand::with_name("set")
                .about("set a value")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .help("bind this value to the key")
                .args(&[
                    Arg::with_name("KEY")
                        .help("the key")
                        .required(true)
                        .index(1),
                    Arg::with_name("VALUE")
                        .help("the value")
                        .required(true)
                        .index(2),
                ]),
            SubCommand::with_name("rm")
                .about("get a value")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .help("remove a value by this key")
                .arg(
                    Arg::with_name("KEY")
                        .help("the key to remove the value")
                        .required(true)
                        .index(1),
                ),
        ])
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("get") {
        eprintln!("unimplemented");
        process::exit(0x0100);
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        eprintln!("unimplemented");
        process::exit(0x0100);
    }

    match matches.subcommand() {
        ("get", _) | ("set", _) | ("rm", _) => {
            eprintln!("unimplemented");
            process::exit(0x0100);
        }
        _ => unreachable!(),
    }

    // more program logic goes here...
}
