use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::fs::{
    FSError, Meta,
    fs::FSFile,
    r#trait::{FSOperations, Inode, InodeOperations},
};

pub struct InMemoryFileInode {
    pub data: RwLock<Vec<u8>>,
    pub meta: Meta,
}

// Example: Converting the "Input" into the "Storage Object"
// impl From<FSFile> for InMemoryFileInode {
//     fn from(file: FSFile) -> Self {
//         let meta = Meta {
//             size: file.content.len() as u64,
//             path_abs: file.path_abs,
//             // ... map other fields
//         };
//         Self {
//             data: RwLock::new(file.content.into_bytes()),
//             meta,
//         }
//     }
// }

impl InMemoryFileInode {
    pub fn new(content: String) -> Self {
        let bytes = content.into_bytes();

        Self {
            meta: Meta::default(),
            data: RwLock::new(bytes.clone()),
        }
    }
}

impl Inode for InMemoryFileInode {
    fn get_meta(&self) -> Meta {
        self.meta.clone()
    }
    fn inode_ops(&self) -> &dyn InodeOperations {
        self
    }
    fn file_ops(&self) -> &dyn FSOperations {
        self
    }
    fn is_dir(&self) -> bool {
        false
    }
}

pub struct InMemoryDirectoryInode {
    pub meta: Meta,
}

impl InMemoryDirectoryInode {
    pub fn new() -> Self {
        Self {
            meta: Meta::default(),
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
    fn file_ops(&self) -> &dyn FSOperations {
        self
    }
    fn is_dir(&self) -> bool {
        true
    }
    fn get_meta(&self) -> Meta {
        self.meta.clone()
    }
}

impl InodeOperations for InMemoryFileInode {
    fn lookup(&self, _name: &str) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::NotFound)
    }

    fn create(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::IoError)
    }

    fn mkdir(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::IoError)
    }
}
impl InodeOperations for InMemoryDirectoryInode {
    fn lookup(&self, _name: &str) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::NotFound)
    }

    fn create(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::IoError)
    }

    fn mkdir(&self, _: &str, _: u32) -> Result<Arc<dyn Inode>, FSError> {
        Err(FSError::IoError)
    }
}

impl FSOperations for InMemoryFileInode {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, FSError> {
        let data = self.data.read().map_err(|_| FSError::IoError)?;

        let start = offset as usize;
        if start >= data.len() {
            return Ok(0);
        }

        let end = (start + buf.len()).min(data.len());
        let len = end - start;

        buf[..len].copy_from_slice(&data[start..end]);

        Ok(len)
    }

    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, FSError> {
        let mut data = self.data.write().map_err(|_| FSError::IoError)?;

        let start = offset as usize;

        if start + buf.len() > data.len() {
            data.resize(start + buf.len(), 0);
        }

        data[start..start + buf.len()].copy_from_slice(buf);

        Ok(buf.len())
    }

    fn fsync(&self) -> Result<(), FSError> {
        Ok(())
    }
    fn release(&self) -> Result<(), FSError> {
        Ok(())
    }
}
impl FSOperations for InMemoryDirectoryInode {
    fn read(&self, _: &mut [u8], _: u64) -> Result<usize, FSError> {
        Err(FSError::IoError)
    }

    fn write(&self, _: &[u8], _: u64) -> Result<usize, FSError> {
        Err(FSError::PermissionDenied)
    }

    fn fsync(&self) -> Result<(), FSError> {
        Ok(())
    }

    fn release(&self) -> Result<(), FSError> {
        Ok(())
    }
}

// 3. Dentry (The Name-to-Node Link): This is the Glue. A Dentry holds the name of the file (e.g., main.rs) and points to an Inode (the actual data).
#[derive(Debug)]
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

    pub fn lookup(&self, name: &str) -> Result<Arc<Dentry>, FSError> {
        self.children
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or(FSError::NotFound)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DentryDTO {
    pub name: String,
    pub inode_id: String,         // Or whatever unique ID you use
    pub children: Vec<DentryDTO>, // Recursive DTO
}
