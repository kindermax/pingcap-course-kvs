use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use log::{error, info, debug};
use serde_json::Deserializer;

use crate::common::{Request, SetResponse, RemoveResponse, GetResponse};
use crate::engines::KvsEngine;
use crate::thread_pool::ThreadPool;
use crate::{Result};

pub struct KvsServer<E: KvsEngine, T: ThreadPool> {
    engine: E,
    thread_pool: T,
}

impl<E: KvsEngine, T: ThreadPool>  KvsServer<E, T> {
    pub fn new(engine: E, thread_pool: T) -> Self {
        KvsServer { engine, thread_pool }
    }

    pub fn run<A: ToSocketAddrs>(self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;


        for stream in listener.incoming() {
            let engine = self.engine.clone();
            self.thread_pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = serve(engine, stream) {
                        error!("Error on serving client: {}", e);
                    }
                },
                Err(e) => error!("Connection failed: {}", e),
            });
        }

        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> Result<()> {
    let peer_addr = tcp.peer_addr()?;
    info!("Accepted connection from {}", peer_addr);

    let buf_reader = BufReader::new(&tcp);
    let mut writer = BufWriter::new(&tcp);
    let req_reader = Deserializer::from_reader(buf_reader).into_iter::<Request>();

    macro_rules! send_resp {
        ($resp:expr) => {
            {
                let resp = $resp;
                serde_json::to_writer(&mut writer, &resp)?;
                writer.flush()?;
                debug!("Resonse sent to {}: {:?}", peer_addr, resp);
            }
        };
    }

    for req in req_reader {
        let req = req?;
        debug!("Receive request from {}: {:?}", peer_addr, req);
        match req {
            Request::Get { key } => {
                send_resp!(match engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(e) => GetResponse::Err(e.to_string()),
                })
            },
            Request::Set { key, value } => {
                send_resp!(match engine.set(key, value) {
                    Ok(_) => SetResponse::Ok,
                    Err(e) => SetResponse::Err(e.to_string())
                })
            },
            Request::Remove { key } => {
                send_resp!(match engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok,
                    Err(e) => RemoveResponse::Err(e.to_string())
                })
            },
        }
    }
    Ok(())
}