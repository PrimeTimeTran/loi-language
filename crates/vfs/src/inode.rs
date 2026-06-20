use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{fs::Metadata, vfs::VfsError};

pub trait InodeOperations: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>, VfsError>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, VfsError>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>, VfsError>;
}

pub trait FileOperations: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, VfsError>;
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, VfsError>;
    fn fsync(&self) -> Result<(), VfsError>;
    fn release(&self) -> Result<(), VfsError>;
}

/// The central "Inode" trait acts as the common interface for
/// all filesystem nodes (files, directories, symlinks, etc.).
///
/// It enforces the "Bridge Pattern": it separates the metadata
/// operations (InodeOperations) from the I/O operations (FileOperations).
pub trait Inode: Send + Sync {
    fn inode_ops(&self) -> &dyn InodeOperations;
    fn file_ops(&self) -> &dyn FileOperations;
    fn is_dir(&self) -> bool;
    fn get_stat(&self) -> Metadata;
}

// --- 2. The Inode Implementation ---
/// The specialized Inode for File types
impl InMemoryFileInode {
    pub fn new(content: String) -> Self {
        let bytes = content.into_bytes();

        Self {
            meta: Metadata::default(),
            data: RwLock::new(bytes.clone()),
        }
    }
}
pub struct InMemoryFileInode {
    pub data: RwLock<Vec<u8>>,
    pub meta: Metadata,
}

impl Inode for InMemoryFileInode {
    fn get_stat(&self) -> Metadata {
        self.meta.clone()
    }
    fn inode_ops(&self) -> &dyn InodeOperations {
        self
    }
    fn file_ops(&self) -> &dyn FileOperations {
        self
    }
    fn is_dir(&self) -> bool {
        false
    }
}

/// The specialized Inode for Directory types
pub struct InMemoryDirectoryInode {
    pub meta: Metadata,
}

impl InMemoryDirectoryInode {
    pub fn new() -> Self {
        Self {
            meta: Metadata::default(),
        }
    }
    fn bump_size(&self) {
        // optional: count children
    }
}

impl Inode for InMemoryDirectoryInode {
    fn inode_ops(&self) -> &dyn InodeOperations {
        self
    }
    fn file_ops(&self) -> &dyn FileOperations {
        self
    }
    fn is_dir(&self) -> bool {
        true
    }
    fn get_stat(&self) -> Metadata {
        self.meta.clone()
    }
}

impl InodeOperations for InMemoryFileInode {
    fn lookup(&self, _name: &str) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::NotFound)
    }

    fn create(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::IoError)
    }

    fn mkdir(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::IoError)
    }
}
impl InodeOperations for InMemoryDirectoryInode {
    fn lookup(&self, _name: &str) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::NotFound)
    }

    fn create(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::IoError)
    }

    fn mkdir(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, VfsError> {
        Err(VfsError::IoError)
    }
}

impl FileOperations for InMemoryFileInode {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, VfsError> {
        let data = self.data.read().map_err(|_| VfsError::IoError)?;

        let start = offset as usize;
        if start >= data.len() {
            return Ok(0);
        }

        let end = (start + buf.len()).min(data.len());
        let len = end - start;

        buf[..len].copy_from_slice(&data[start..end]);

        Ok(len)
    }

    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, VfsError> {
        let mut data = self.data.write().map_err(|_| VfsError::IoError)?;

        let start = offset as usize;

        if start + buf.len() > data.len() {
            data.resize(start + buf.len(), 0);
        }

        data[start..start + buf.len()].copy_from_slice(buf);

        Ok(buf.len())
    }

    fn fsync(&self) -> Result<(), VfsError> {
        Ok(())
    }
    fn release(&self) -> Result<(), VfsError> {
        Ok(())
    }
}
impl FileOperations for InMemoryDirectoryInode {
    fn read(&self, _: &mut [u8], _: u64) -> Result<usize, VfsError> {
        Err(VfsError::IoError)
    }

    fn write(&self, _: &[u8], _: u64) -> Result<usize, VfsError> {
        Err(VfsError::PermissionDenied)
    }

    fn fsync(&self) -> Result<(), VfsError> {
        Ok(())
    }

    fn release(&self) -> Result<(), VfsError> {
        Ok(())
    }
}

pub struct Dentry {
    pub name: String,
    pub inode: Arc<dyn Inode>,
    pub children: RwLock<HashMap<String, Arc<Dentry>>>,
}

impl Dentry {
    pub fn new(name: &str, inode: Arc<dyn Inode>) -> Self {
        Self {
            name: name.to_string(),
            inode,
            children: RwLock::new(HashMap::new()),
        }
    }

    pub fn lookup(&self, name: &str) -> Result<Arc<Dentry>, VfsError> {
        self.children
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or(VfsError::NotFound)
    }
}
