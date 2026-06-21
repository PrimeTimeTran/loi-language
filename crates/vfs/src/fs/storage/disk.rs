use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::fs::{Dentry, FS, FSConfig, FSError, FSHandle, HandleAllocator, Meta, Storage};

#[derive(Default)]
pub struct DiskStorage {
    path_root: PathBuf,
}

impl DiskStorage {
    pub fn new() -> Self {
        Self {
            path_root: PathBuf::new(),
        }
    }
    pub fn with_root(path: impl Into<PathBuf>) -> Self {
        Self {
            path_root: path.into(),
        }
    }

    fn walk(&self, path: &str) -> PathBuf {
        let clean = path.trim_matches('/');

        self.path_root.join(clean)
    }
}

#[async_trait]
impl Storage for DiskStorage {
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

    // async fn readdir(&self, h: &FSHandle) -> Result<Vec<String>, FSError> {
    //     todo!()
    // }
}

pub type DiskFS = FS<DiskStorage>;
