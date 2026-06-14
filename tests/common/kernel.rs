use loi::init::init;
use loi::kernel::Kernel;
use std::sync::{Arc, RwLock};

pub struct KernelTestHarness {
    pub kernel: Kernel,
}

impl KernelTestHarness {
    pub fn new() -> Self {
        Self { kernel: init() }
    }

    pub fn peek_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&loi::compiler::state::CompileState) -> T,
    {
        let state = self
            .kernel
            .engine
            .state
            .read()
            .expect("State lock poisoned");
        f(&state)
    }

    pub fn peek_config<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&loi::compiler::config::CompileConfig) -> T,
    {
        let config = self
            .kernel
            .engine
            .config
            .read()
            .expect("Config lock poisoned");
        f(&config)
    }
}
