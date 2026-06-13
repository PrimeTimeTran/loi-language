use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct TaskScheduler;

impl TaskScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskScheduler {
    pub fn schedule<F: FnOnce() + Send + 'static>(&self, f: F) {
        std::thread::spawn(f);
    }
}

#[derive(Default)]
pub struct JobQueue {
    pub jobs: Arc<Mutex<Vec<String>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        let jobs = Arc::new(Mutex::new(Vec::<String>::new()));

        Self { jobs }
    }
}

#[derive(Default)]
pub struct PrioritySystem {
    pub priority_map: HashMap<String, u8>,
}

impl PrioritySystem {
    pub fn new() -> Self {
        Self {
            priority_map: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct PluginSystem;
