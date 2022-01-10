use super::ThreadPool;
use crate::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    panic,
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Run(Job),
    Exit,
}

pub struct SharedQueueThreadPool {
    handles: Vec<JoinHandle<()>>,
    channel: (Sender<Message>, Receiver<Message>),
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: usize) -> Result<Self> {
        let mut handles = Vec::with_capacity(threads);
        let channel: (Sender<Message>, Receiver<Message>) = unbounded();
        for _ in 0..threads {
            let recv = channel.1.clone();
            handles.push(thread::spawn(move || {
                loop {
                    // reuse the panicked thread, because recreating threads requires communication to tell the main thread and maybe slower than reuse
                    let exit_now = panic::catch_unwind(|| {
                        if let Message::Run(job) = recv.recv().unwrap() {
                            job();
                            false
                        } else {
                            true
                        }
                    });

                    if exit_now.is_err() {
                        continue;
                    } else if exit_now.unwrap() {
                        break;
                    }
                }
            }));
        }
        Ok(SharedQueueThreadPool { handles, channel })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.channel.0.send(Message::Run(Box::new(job))).unwrap();
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.handles.len() {
            self.channel.0.send(Message::Exit).unwrap();
        }

        while !self.handles.is_empty() {
            let handle = self.handles.pop().unwrap();
            handle.join().unwrap();
        }
    }
}
