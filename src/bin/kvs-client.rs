use clap::{Parser, Subcommand, Args};

use std::net::SocketAddr;

use kvs::{Result, KvsClient};


#[derive(Parser)]
#[clap(name = "kvs-client")]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the string value of a given string key
    Get(Get),
    /// Set the value of a string key to a string
    Set(Set),
    /// Remove a given string key.
    Rm(Rm)
}


#[derive(Args)]
struct Get {
    #[clap(help = "A string key")]
    key: String,
    #[clap(
        name = "ADDRESS_FORMAT",
        default_value = "127.0.0.1:4000",
        help = "Sets the server address",
    )]
    addr: SocketAddr
}

#[derive(Args)]
struct Set {
    #[clap(help = "A string key")]
    key: String,
    #[clap(help = "The string value of the key")]
    value: String,
    #[clap(
        name = "ADDRESS_FORMAT",
        default_value = "127.0.0.1:4000",
        help = "Sets the server address",
    )]
    addr: SocketAddr
}

#[derive(Args)]
struct Rm {
    #[clap(help = "A string key")]
    key: String,
    #[clap(
        name = "ADDRESS_FORMAT",
        default_value = "127.0.0.1:4000",
        help = "Sets the server address",
    )]
    addr: SocketAddr
}


fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}


fn run(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Get(Get{ key, addr }) => {
            let mut client = KvsClient::connect(addr)?;
            if let Some(value) = client.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        },
        Commands::Set(Set{ key, value, addr }) => {
            let mut client = KvsClient::connect(addr)?;
            client.set(key.to_string(), value.to_string())?
        },
        Commands::Rm(Rm{ key, addr }) => {
            let mut client = KvsClient::connect(addr)?;
            client.remove(key.to_string())?;
        },
    }
    Ok(())
}