mod naive_thread_pool;
mod rayon_thread_pool;
mod shared_queue_thread_pool;

use crate::Result;
pub use naive_thread_pool::NaiveThreadPool;
pub use rayon_thread_pool::RayonThreadPool;
pub use shared_queue_thread_pool::SharedQueueThreadPool;

pub trait ThreadPool: Sized {
    fn new(threads: u32) -> Result<Self>;
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
}
