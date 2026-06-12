use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct TaskScheduler;

impl TaskScheduler {
    pub fn schedule<F: FnOnce() + Send + 'static>(&self, f: F) {
        std::thread::spawn(f);
    }
}

#[derive(Default)]
pub struct JobQueue {
    pub jobs: Arc<Mutex<Vec<String>>>,
}

#[derive(Default)]
pub struct PrioritySystem {
    pub priority_map: HashMap<String, u8>,
}

#[derive(Default)]
pub struct PluginSystem;
