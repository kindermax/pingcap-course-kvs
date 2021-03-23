use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::io::{Write, SeekFrom};
use std::io::BufReader;

use crate::{Result, KvsError};

type Index = HashMap<String, u64>;

#[derive(Default, Debug)]
pub struct KvStore {
    path: PathBuf,
    index: Index,
    offset: u64,
}


#[derive(Serialize, Deserialize, Debug)]
enum Cmd {
    Set(String, String),
    Rm(String),
}

// TODOs:
// where is the system performing buffering and where do you need buffering?
// What is the impact of buffering on subsequent reads?
// When should you open and close file handles? For each command? For the lifetime of the KvStore?


impl KvStore {
    pub fn open(path: &Path) -> Result<KvStore> {
        let mut store = KvStore {
            path: path.to_path_buf().join(PathBuf::from("wal.json")),
            index: HashMap::new(),
            offset: 0,
        };

        store.restore_index()?;

        Ok(store)
    }


    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let offset = self.write_cmd(Cmd::Set(key.clone(), value.clone()))?;
        self.index.insert(key, self.offset);
        self.offset += offset;
        // TODO better error handling
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(offset) => {
                let offset = offset.to_owned();
                self.read_log_entry(offset)
            },
            None => Ok(None),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound)
        }

        self.write_cmd(Cmd::Rm(key.clone()))?;

        self.index.remove(&key);
        Ok(())
    }

    // TODO opens file at every call
    fn write_cmd(&self, cmd: Cmd) -> Result<u64> {
        let mut log = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.path)?;

        let data = serde_json::to_string(&cmd)?;
        log.write_all(format!("{}\n", data).as_bytes())?;

        Ok(log.seek(SeekFrom::End(0))?)
        // Ok(log.stream_len()?)
    }

    fn restore_index(&mut self) -> Result<()> {
        if !Path::new(&self.path).exists() {
            File::create(&self.path)?;
        }
        let log = File::open(&self.path)?;

        let reader = BufReader::new(log);

        for line in reader.lines() {
            let raw_cmd = line?;
            let cmd: Cmd = serde_json::from_str(&raw_cmd)?;
            match cmd {
                Cmd::Set(key, _) => {
                    self.index.insert(key, self.offset);
                },
                Cmd::Rm(key) => {
                    self.index.remove(&key);
                },
            }
            // +1 for \n
            self.offset += raw_cmd.len() as u64 + 1;
        }

        Ok(())
    }

    fn read_log_entry(&mut self, offset: u64) -> Result<Option<String>> {
        if !Path::new(&self.path).exists() {
            return Err(KvsError::KeyNotFound);
        }
        let mut log = File::open(&self.path)?;
        log.seek(SeekFrom::Start(offset))?;

        let reader = BufReader::new(log);

        if let Some(line) = reader.lines().next() {
            let cmd: Cmd = serde_json::from_str(&line?)?;
            match cmd {
                Cmd::Set(_, value) => return Ok(Some(value)),
                _ => panic!("must be Set cmd at offset")
            }
        }

        Ok(None)
    }
}
