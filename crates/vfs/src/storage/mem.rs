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
        web_sys::console::log_1(&format!("[MemStorage] read {:?}", h).into());
        let files = self.files.read().unwrap();
        Ok(files.get(h).cloned().unwrap_or_default())
    }

    async fn write(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        web_sys::console::log_1(&format!("[MemStorage] write {:?}", h).into());
        let mut files = self.files.write().unwrap();
        files.insert(*h, data);
        Ok(())
    }

    async fn append(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        web_sys::console::log_1(&format!("[MemStorage] append {:?}", h).into());
        let mut files = self.files.write().unwrap();
        files.entry(*h).or_default().extend(data);
        Ok(())
    }

    async fn meta(&self, h: &FSHandle) -> Result<Meta, FSError> {
        web_sys::console::log_1(&format!("[MemStorage] meta {:?}", h).into());
        let meta = self.meta.read().unwrap();
        meta.get(h).cloned().ok_or(FSError::NotFound)
    }
}

pub type MemFS = FS<MemStorage>;
