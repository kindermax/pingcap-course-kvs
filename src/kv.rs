use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;

use crate::{Result, KvsError};

#[derive(Default, Debug)]
pub struct KvStore {
    path: PathBuf,
    store: HashMap<String, String>,
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
        Ok(KvStore {
            path: path.to_path_buf().join(PathBuf::from("1.json")),
            store: HashMap::new(),
        })
    }


    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Cmd::Set(key.clone(), value.clone());

        self.write_cmd(cmd)?;

        self.store.insert(key, value);
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.read_log()?;
        Ok(self.store.get(&key).cloned())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.store.contains_key(&key) {
            // TODO maybe return error if no key ?
            return Err(KvsError::KeyNotFound)
        }

        let cmd = Cmd::Rm(key.clone());

        self.write_cmd(cmd)?;

        self.store.remove(&key);
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
        let content = std::fs::read_to_string(&self.path)?;

        for line in content.lines() {
            let cmd: Cmd = serde_json::from_str(line)?;
            match cmd {
                Cmd::Set(key, value) => {
                    // TODO better error handling
                    self.set(key, value)?
                },
                Cmd::Rm(key) => {
                    self.remove(key)?
                },
            }
        }

        Ok(())
    }
}
