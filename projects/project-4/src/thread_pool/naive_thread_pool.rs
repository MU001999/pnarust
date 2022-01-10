use super::ThreadPool;
use crate::Result;

pub struct NaiveThreadPool {
    // ...
}

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: usize) -> Result<Self> {
        Ok(NaiveThreadPool {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(job);
    }
}
