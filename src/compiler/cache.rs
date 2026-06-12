use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct CompilationCache {
    pub cache: HashMap<String, Vec<u8>>,
}

#[derive(Default)]
pub struct PersistentCache {
    pub disk_path: Option<PathBuf>,
}

#[derive(Default)]
pub struct MemoryCache {
    pub map: HashMap<String, String>,
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
