use rayon::ThreadPoolBuilder;

use crate::Result;
use super::ThreadPool;

pub struct RayonThreadPool {
    pool: rayon::ThreadPool,

}

impl ThreadPool for RayonThreadPool {
    fn new(size: u32) -> Result<Self> {
        assert!(size > 0);
        Ok(RayonThreadPool {
            pool: ThreadPoolBuilder::new()
                .num_threads(size as usize)
                .build()
                .unwrap(),
        })
    }

    fn spawn<F>(&self, func: F)
        where F: FnOnce() + Send + 'static
    {
        self.pool.install(|| func());
    }
}