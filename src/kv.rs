use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::io::Write;
use std::io::BufReader;

use crate::{Result, KvsError};

type Index = HashMap<String, String>;

#[derive(Default, Debug)]
pub struct KvStore {
    path: PathBuf,
    index: Index,
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
        };

        store.read_log()?;

        Ok(store)
    }


    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.write_cmd(Cmd::Set(key.clone(), value.clone()))?;
        self.index.insert(key, value);
        // TODO better error handling
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self.index.get(&key).cloned())
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
    fn write_cmd(&self, cmd: Cmd) -> Result<()> {
        let mut log = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.path)?;

        let data = serde_json::to_string(&cmd)?;
        log.write_all(format!("{}\n", data).as_bytes())?;

        Ok(())
    }

    fn read_log(&mut self) -> Result<()> {
        if !Path::new(&self.path).exists() {
            File::create(&self.path)?;
        }
        let log = File::open(&self.path)?;

        let reader = BufReader::new(log);


        // TODO Maybe serde will deserialize a record directly from an
        // I/O stream and stop reading when it's done,
        // leaving the file cursor in the correct place to read subsequent records
        for line in reader.lines() {
            let cmd: Cmd = serde_json::from_str(&line?)?;
            match cmd {
                Cmd::Set(key, value) => {
                    self.index.insert(key, value);
                },
                Cmd::Rm(key) => {
                    self.index.remove(&key);
                },
            }
        }

        Ok(())
    }
}
