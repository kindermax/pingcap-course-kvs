use clap::{App, AppSettings, Arg, SubCommand};

use std::process::exit;
use kvs::KvStore;
use std::path::Path;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();

    // TODO handle open error properly
    // 1. some method ?
    // 2. match
    // 3. return type in main
    let mut store = KvStore::open(Path::new("./")).expect("Open wal.json must succeed");

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            let key = _matches.value_of("KEY").unwrap();
            let value = _matches.value_of("VALUE").unwrap();
            match store.set(key.to_owned(), value.to_owned()) {
                Ok(()) => exit(0),
                Err(err) => eprintln!("failed to set key {} with value {}: {:?}", key, value, err)
            }
            exit(1);
        }
        ("get", Some(_matches)) => {
            let key = _matches.value_of("KEY").unwrap();
            match store.get(key.to_owned()) {
                Ok(value) => {
                    if let Some(value) = value {
                        println!("{}", value);
                        exit(0);
                    }
                },
                Err(err) => {
                    // TODO how to print KvsError
                    eprintln!("failed to get key {}: {:?}", key, err);
                }
            };
            println!("Key not found");
            exit(0);
        }
        ("rm", Some(_matches)) => {
            let key = _matches.value_of("KEY").unwrap();
            match store.remove(key.to_owned()) {
                Ok(()) => {
                    exit(0);
                },
                Err(err) => eprintln!("failed to remove key {}: {:?}", key, err)
            };
            println!("Key not found");
            exit(1);
        }
        _ => unreachable!(),
    }
}
