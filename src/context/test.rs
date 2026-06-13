use crate::compiler::{
    cache::MemoryCache,
    diagnostic::{DiagnosticStore, Logger},
};

#[derive(Default, Debug)]
pub struct TestContext {
    pub logger: Logger,
    pub cache: MemoryCache,
    pub diagnostics: DiagnosticStore,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            logger: Logger::test(),
            diagnostics: DiagnosticStore::default(),
            cache: MemoryCache::new(),
        }
    }
}
