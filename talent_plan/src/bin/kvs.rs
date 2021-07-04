// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
extern crate clap;

use std::process;

use clap::{App, Arg, SubCommand};

use kvs::kvs::error::CliErr;
use kvs::KvStore;

fn main() -> Result<(), CliErr> {
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
                    Arg::with_name("VAL")
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

    let mut kv_store = KvStore::new()?;
    match matches.subcommand() {
        ("set", Some(set_args)) => {
            let key = set_args.value_of("KEY").expect("Key is not Exists");
            let val = set_args.value_of("VAL").expect("Val is not Exists");
            if let Err(err) = kv_store.set(key.to_owned(), val.to_owned()) {
                println!("{}", err)
            }
        }
        ("get", Some(get_args)) => {
            let key = get_args.value_of("KEY").expect("Key is not Exists");
            match kv_store.get(key.to_owned()) {
                Ok(val) => println!("{}", val.unwrap_or(String::from("Key not found"))),
                Err(err) => println!("{}", err),
            }
        }
        ("rm", Some(rm_args)) => {
            let key = rm_args.value_of("KEY").expect("Key is not Exists");
            if let Err(err) = kv_store.remove(key.to_owned()) {
                println!("{}", err);
                process::exit(0x0100);
            }
        }
        _ => {
            eprintln!("unimplemented");
            process::exit(0x0100);
        }
    }

    // more program logic goes here...
    Ok(())
}
