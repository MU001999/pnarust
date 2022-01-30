mod naive_thread_pool;
mod rayon_thread_pool;
mod shared_queue_thread_pool;

use crate::Result;
pub use naive_thread_pool::NaiveThreadPool;
pub use rayon_thread_pool::RayonThreadPool;
pub use shared_queue_thread_pool::SharedQueueThreadPool;

/// A simple trait of thread pools,
/// which only provides methods new and spawn.
pub trait ThreadPool: Sized {
    /// Creates a new thread pool capable of executing `threads` number of jobs concurrently.
    fn new(threads: usize) -> Result<Self>;

    /// Executes the function `job` on one thread in this pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
