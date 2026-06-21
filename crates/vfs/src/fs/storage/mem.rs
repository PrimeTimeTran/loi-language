use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::fs::{Dentry, FS, FSError, FSHandle, HandleAllocator, Meta, Storage};

#[derive(Default)]
pub struct MemStorage {
    nodes: HashMap<FSHandle, Vec<u8>>,
    meta: HashMap<FSHandle, Meta>,
    children: HashMap<FSHandle, Vec<String>>,
}
impl MemStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Storage for MemStorage {
    async fn read(&self, h: &FSHandle) -> Result<Vec<u8>, FSError> {
        todo!("read from disk path mapped by handle")
    }

    async fn write(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        todo!()
    }

    async fn append(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError> {
        todo!()
    }

    async fn meta(&self, h: &FSHandle) -> Result<Meta, FSError> {
        todo!()
    }

    async fn readdir(&self, h: &FSHandle) -> Result<Vec<String>, FSError> {
        todo!()
    }
}

pub type MemFS = FS<MemStorage>;
