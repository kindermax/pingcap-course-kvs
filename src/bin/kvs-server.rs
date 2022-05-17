use clap::{Parser, ArgEnum};

use kvs::server::KvsServer;
use log::{info, warn, error, LevelFilter};

use std::{fs, fmt};
use std::net::SocketAddr;
use std::env::current_dir;
use std::str::FromStr;
use std::process::exit;

use kvs::{Result, KvsError};
use kvs::engines::{KvStore, KvsEngine};


const DEFAULT_ENGINE: Engine = Engine::kvs;


#[derive(Parser)]
#[clap(name = "kvs-server")]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(
        name = "ADDRESS_FORMAT",
        default_value = "127.0.0.1:4000",
        help = "Sets the server address",
    )]
    addr: SocketAddr,
    #[clap(
        name = "ENGINE_NAME",
        help = "Sets the server engine",
    )]
    #[clap(arg_enum)]
    engine: Option<Engine>
}


#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, ArgEnum, Debug)]
enum Engine {
    sled,
    kvs,
}

impl FromStr for Engine {
    type Err = KvsError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "sled" => Ok(Engine::sled),
            "kvs" => Ok(Engine::kvs),
            _ => Err(KvsError::StringError(format!("unknown engine: {}", s))),
        }
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Engine::sled => write!(f, "sled"),
            Engine::kvs => write!(f, "kvs"),
        }
    }
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut cli = Cli::parse();

    let res = current_engine().and_then(move |current_engine| {
        if cli.engine.is_none() {
            cli.engine = current_engine;
        }
        if current_engine.is_some() && current_engine != cli.engine {
            error!("Wrong engine!");
            exit(1);
        }
        run(cli)
    });

    if let Err(e) = res {
        error!("{}", e);
        exit(1);
    }
}


fn run(cli: Cli) -> Result<()> {
    let engine = cli.engine.unwrap_or(DEFAULT_ENGINE);
    let addr = cli.addr;

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {:?}", engine);
    info!("Listening on {:?}", addr);

    let workdir = current_dir()?;
    fs::write(workdir.join("engine"), format!("{engine}"))?;

    match engine {
        Engine::kvs => {
            run_with_engine(
                KvStore::open(workdir)?,
                addr
            )
        },
        Engine::sled => {
            unimplemented!()
        },
    }
}

fn run_with_engine<E: KvsEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr)
}


fn current_engine() -> Result<Option<Engine>> {
    let engine = current_dir()?.join("engine");
    if !engine.exists() {
        return Ok(None);
    }

    match fs::read_to_string(engine)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!("The content of engine file is invalid: {}", e);
            Ok(None)
        }
    }
}