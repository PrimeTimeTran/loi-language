use crate::compiler::{cache::MemoryCache, diagnostic::{DiagnosticStore, Logger}};

pub struct TestContext {
    pub logger: Logger,
    pub diagnostics: DiagnosticStore,
    pub cache: MemoryCache,
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
