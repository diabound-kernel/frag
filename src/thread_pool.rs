use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Task {
    New(Job),
    Terminate,
}

struct Worker {
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let task = receiver.lock().unwrap().recv().unwrap();

            match task {
                Task::New(job) => {
                    job();
                }

                Task::Terminate => {
                    break;
                }
            }
        });

        Self {
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Task>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let job = Box::new(f);

        self.sender.send(Task::New(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Task::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            // unwrap and take ownership of JoinHandler and set optional to None
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
