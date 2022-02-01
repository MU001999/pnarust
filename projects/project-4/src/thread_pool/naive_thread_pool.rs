use super::ThreadPool;
use crate::Result;

/// A naive thread pool that just creates a new thread for each spawned job.
pub struct NaiveThreadPool {
    // ...
}

impl ThreadPool for NaiveThreadPool {
    /// Creates a naive thread pool that creates a new thread for each spawned job.
    fn new(_threads: usize) -> Result<Self> {
        Ok(NaiveThreadPool {})
    }

    /// Executes a job by std::thread::spawn.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(job);
    }
}
