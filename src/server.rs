use std::io::{BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

use log::{error, info, debug};
use serde_json::Deserializer;

use crate::common::{Request, SetResponse, RemoveResponse, GetResponse};
use crate::engines::KvsEngine;
use crate::{Result};

pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine>  KvsServer<E> {
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    pub fn run(mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        error!("Error on serving client: {}", e);
                    }
                },
                Err(e) => error!("Connection failed: {}", e),
            }
        }

        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
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
                    send_resp!(match self.engine.get(key) {
                        Ok(value) => GetResponse::Ok(value),
                        Err(e) => GetResponse::Err(e.to_string()),
                    })
                },
                Request::Set { key, value } => {
                    send_resp!(match self.engine.set(key, value) {
                        Ok(_) => SetResponse::Ok,
                        Err(e) => SetResponse::Err(e.to_string())
                    })
                },
                Request::Remove { key } => {
                    send_resp!(match self.engine.remove(key) {
                        Ok(_) => RemoveResponse::Ok,
                        Err(e) => RemoveResponse::Err(e.to_string())
                    })
                },
            }
        }
        Ok(())
    }
}