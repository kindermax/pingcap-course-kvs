use serde::Deserialize;
use serde_json::de::{IoRead, Deserializer};

use std::{
    net::{TcpStream, ToSocketAddrs},
    io::{BufReader, BufWriter, Write}
};

use crate::{Result,KvsError, common::RemoveResponse};
use crate::common::{Request, GetResponse, SetResponse};

pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let reader = TcpStream::connect(addr)?;
        // TODO what try_clone does ?
        let writer = reader.try_clone()?;
        Ok(KvsClient { 
            reader: Deserializer::from_reader(
                BufReader::new(reader)
            ),
            writer: BufWriter::new(writer)
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(err) => Err(KvsError::StringError(err))
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set { key, value })?;
        self.writer.flush()?;
        let resp = SetResponse::deserialize(&mut self.reader)?;
        match resp {
            SetResponse::Ok => Ok(()),
            SetResponse::Err(err) => Err(KvsError::StringError(err))
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Remove { key })?;
        self.writer.flush()?;
        let resp = RemoveResponse::deserialize(&mut self.reader)?;
        match resp {
            RemoveResponse::Ok => Ok(()),
            RemoveResponse::Err(err) => Err(KvsError::StringError(err))
        }
    }
}