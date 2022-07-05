use std::panic::{AssertUnwindSafe, self};
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

use crate::Result;
use super::ThreadPool;

pub struct SharedQueueThreadPool {
    sender: Sender<Message>,
    workers: Vec<Worker>,
}

struct Worker {
    handle: Option<JoinHandle<()>>,
}


struct Job {
    func: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl Job {
    fn new(func: Box<dyn FnOnce() + Send + 'static>) -> Job {
        Job { func: Some(func) }
    }

    fn run(&mut self) {
        if let Some(func) = self.func.take() {
            func();
        }
    }
}

impl Worker {
    fn new(receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv().unwrap();
            match msg {
                Message::Job(func) => {
                    let mut job = Job::new(func);
                    let _ = panic::catch_unwind(AssertUnwindSafe(move || job.run()));
                },
                Message::Shutdown => break
            }
        });
        Worker { handle: Some(handle) }
    }
}

enum Message {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Shutdown
}


impl ThreadPool for SharedQueueThreadPool {
    fn new(size: u32) -> Result<Self> {
        assert!(size > 0);
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size as usize);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }
        
        log::info!("Started {} workers", workers.len());
        let pool = SharedQueueThreadPool{
            sender,
            workers,
        };
        Ok(pool)
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        self.send(job);
    }
}

impl SharedQueueThreadPool {
    fn send<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        self.sender.send(Message::Job(Box::new(job))).unwrap();
    }
}


impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Shutdown).expect("send shutdown message");
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
