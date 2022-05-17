use sled;

use super::KvsEngine;
use crate::{Result, KvsError};


pub struct SledKvsEngine {
    db: sled::Db,
}


impl SledKvsEngine {
    pub fn new(db: sled::Db) -> Self {
        SledKvsEngine { db }
    }
}


impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let val = self.db.get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?;

        Ok(val)
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        let _ = self.db.insert(key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let _ = self.db.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        self.db.flush()?;
        Ok(())
    }
}