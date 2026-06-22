use std::collections::HashMap;

use async_trait::async_trait;

use crate::fs::{
    error::FSError,
    meta::Meta,
    system::{FS, FSHandle},
    r#trait::Storage,
};

#[derive(Default)]
pub struct MemStorage {
    files: std::sync::RwLock<HashMap<FSHandle, Vec<u8>>>,
    meta: std::sync::RwLock<HashMap<FSHandle, Meta>>,
    // dirs: std::sync::RwLock<HashMap<FSHandle, Vec<FSHandle>>>,
}

impl MemStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Storage for MemStorage {
    async fn read(&self, h: &FSHandle) -> Result<Vec<u8>, FSError> {
        crate::vfs_log!("[MemStorage] read {:?}", h);
        let files = self.files.read().unwrap();
        Ok(files.get(h).cloned().unwrap_or_default())
    }

    async fn write(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        crate::vfs_log!("[MemStorage] write {:?}", h);
        let mut files = self.files.write().unwrap();
        files.insert(*h, data);
        Ok(())
    }

    async fn append(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        crate::vfs_log!("[MemStorage] append {:?}", h);
        let mut files = self.files.write().unwrap();
        files.entry(*h).or_default().extend(data);
        Ok(())
    }

    async fn meta(&self, h: &FSHandle) -> Result<Meta, FSError> {
        crate::vfs_log!("[MemStorage] meta {:?}", h);
        let meta = self.meta.read().unwrap();
        meta.get(h).cloned().ok_or(FSError::NotFound)
    }
}

pub type MemFS = FS<MemStorage>;
