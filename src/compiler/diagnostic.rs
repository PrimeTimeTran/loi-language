#[derive(Default)]
pub struct Logger;

impl Logger {
    pub fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }
}

#[derive(Default)]
pub struct TraceSystem;

#[derive(Default)]
pub struct Profiler;

#[derive(Default)]
pub struct Inspector;

#[derive(Default)]
pub struct CompilerEventBus;
