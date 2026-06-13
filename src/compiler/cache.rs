use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct CompilationCache {
    pub cache: HashMap<String, Vec<u8>>,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct PersistentCache {
    pub disk_path: Option<PathBuf>,
}

impl PersistentCache {
    pub fn new() -> Self {
        Self {
            disk_path: Some(PathBuf::new()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MemoryCache {
    pub map: HashMap<String, String>,
}
impl MemoryCache {
    pub fn new() -> Self {
        let map = HashMap::new();
        Self { map }
    }
}

#[derive(Default)]
pub struct CachePolicy;

impl CachePolicy {
    pub fn should_invalidate(&self, _key: &str) -> bool {
        true
    }
}

#[derive(Default)]
pub struct NetworkCache;
