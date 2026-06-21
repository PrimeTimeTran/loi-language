use std::sync::Arc;

use async_trait::async_trait;

use crate::fs::{Meta, error::FSError, system::FSHandle};

// Storage = “how bytes + metadata are persisted”
#[async_trait]
pub trait Storage: Send + Sync {
    async fn read(&self, h: &FSHandle) -> Result<Vec<u8>, FSError>;
    async fn write(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn append(&self, h: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    async fn meta(&self, h: &FSHandle) -> Result<Meta, FSError>;
    // async fn readdir(&self, h: &FSHandle) -> Result<Vec<String>, FSError>;
}

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn walk(&self, path: &str) -> Result<FSHandle, FSError>;
}
/// The central "Inode" trait acts as the common interface for
/// all filesystem nodes (files, directories, symlinks, etc.).
///
/// It enforces the "Bridge Pattern": it separates the Meta
/// operations (InodeOperations) from the I/O operations (InodeOperations).
#[async_trait]
pub trait Inode: Send + Sync {
    fn is_dir(&self) -> bool;
    fn handle(&self) -> FSHandle;
    fn meta(&self) -> &Meta;
}

pub trait InodeOperations: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, FSError>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, FSError>;
}

pub trait FileOperations: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, FSError>;
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, FSError>;
    fn fsync(&self) -> Result<(), FSError>;
    fn release(&self) -> Result<(), FSError>;
}

pub trait Backend: Send + Sync {
    fn read(&self, handle: &FSHandle) -> Result<Vec<u8>, FSError>;
    fn write(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
    fn append(&self, handle: &FSHandle, data: Vec<u8>) -> Result<(), FSError>;
}
