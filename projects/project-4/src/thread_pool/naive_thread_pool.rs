use super::ThreadPool;

pub struct NaiveThreadPool {

}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> crate::Result<Self> {
        todo!()
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        todo!()
    }
}
