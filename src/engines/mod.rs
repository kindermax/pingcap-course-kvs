use crate::Result;

pub trait KvsEngine: Clone + Send + 'static {
    fn get(&self, key: String) -> Result<Option<String>>;
    fn set(&self, key: String, value: String) -> Result<()>;
    fn remove(&self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
