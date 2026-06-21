use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::fs::{
    FSError, FSFile, FSHandle, FileOperations, Inode, InodeOperations, Meta, NodeType,
};

// 3. Dentry (The Name-to-Node Link): This is the Glue. A Dentry holds the name of the file (e.g., main.rs) and points to an Inode (the actual data).
pub struct Dentry {
    pub name: String,
    pub inode: Arc<dyn Inode>,
    pub children: RwLock<HashMap<String, Arc<Dentry>>>,
}

pub struct InMemoryFileInode {
    pub meta: Meta,
    pub handle: FSHandle,
}
pub struct InMemoryDirectoryInode {
    pub meta: Meta,
    pub handle: FSHandle,
}

#[async_trait]
impl Inode for InMemoryDirectoryInode {
    fn is_dir(&self) -> bool {
        true
    }

    fn meta(&self) -> &Meta {
        &self.meta
    }
    fn handle(&self) -> FSHandle {
        self.handle
    }
}

#[async_trait]
impl Inode for InMemoryFileInode {
    fn is_dir(&self) -> bool {
        true
    }

    fn meta(&self) -> &Meta {
        &self.meta
    }
    fn handle(&self) -> FSHandle {
        self.handle
    }
}

impl InMemoryDirectoryInode {
    pub fn new(handle: FSHandle, meta: Meta) -> Self {
        Self { meta, handle }
    }

    fn bump_size(&self) {
        // optional
    }
}
impl InMemoryFileInode {
    pub fn new(handle: FSHandle, meta: Meta) -> Self {
        Self { meta, handle }
    }

    pub fn handle(&self) -> FSHandle {
        self.handle
    }
}

pub struct RootInode {
    meta: Meta,
    pub handle: FSHandle,
}

impl RootInode {
    pub fn new(handle: FSHandle, meta: Meta) -> Self {
        Self { meta, handle }
    }
    // pub fn new(handle: FSHandle) -> Self {
    //     Self {
    //         meta: Meta::new(vec![], NodeType::Directory).with_handle(handle),
    //     }
    // }
}

impl Inode for RootInode {
    fn is_dir(&self) -> bool {
        todo!()
    }
    fn meta(&self) -> &Meta {
        todo!()
    }
    fn handle(&self) -> FSHandle {
        self.handle
    }
}

impl Dentry {
    pub fn new(name: &str, inode: Arc<dyn Inode>) -> Self {
        Self {
            name: name.to_string(),
            inode,
            children: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_root(root_inode: Arc<dyn Inode>) -> Self {
        Self {
            name: "/".to_string(),
            inode: root_inode,
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
    pub inode_id: String,
    pub children: Vec<DentryDTO>,
}
