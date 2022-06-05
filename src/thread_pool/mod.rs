use crate::Result;

pub trait ThreadPool {
    fn new(size: u32) -> Result<Self>
        where Self: Sized;
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
}

mod naive;
mod shared_queue;
mod rayon;

pub use self::naive::NaiveThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;
pub use self::rayon::RayonThreadPool;