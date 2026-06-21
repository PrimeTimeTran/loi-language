use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::fs::{
    Dentry, Engine, FSConfig, FSError, InMemoryDirectoryInode, InMemoryFileInode, Inode, JsonNode,
    Meta, NodeType, RootInode, Storage, disk::DiskFS, mem::MemFS,
};

#[derive(Clone, Default)]
pub struct HandleAllocator {
    counter: Arc<AtomicU64>,
}

impl HandleAllocator {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn new_handle(&self) -> FSHandle {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        FSHandle(id)
    }
}

// 1. FSPath (The Address): A conceptual string/vector (/src/main.rs). It tells you where something should be.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FSPath(Vec<String>);

// 2. FSHandle (The Identifier): The key used to "grab" the data. It might be a memory pointer or a file descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct FSHandle(pub u64);

#[derive(Deserialize, Serialize)]
pub enum FSHandleDTO {
    Mem(String),
    Host(PathBuf),
}

pub struct FSFile {
    pub handle: FSHandle,
    pub content: Vec<u8>,
}

#[derive(Deserialize)]
pub struct FSNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub children: Vec<FSNode>,
    pub content: Option<String>,
    // pub r#type: NodeType,
}

#[derive(Deserialize, Serialize)]
pub struct FSFileDTO {
    pub handle_id: String,
    pub content: String,
}

pub struct FSInput {
    pub files: Vec<FileEntry>,
}

#[derive(Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub r#type: NodeType,
}

impl FSInput {
    pub fn empty() -> Self {
        Self { files: vec![] }
    }
    pub fn from_files(paths: Vec<String>) -> Self {
        let files = paths
            .into_iter()
            .map(|path| FileEntry {
                path,
                r#type: NodeType::File,
            })
            .collect();

        Self { files }
    }
    pub fn from_node(root: FSNode) -> Self {
        let mut files = Vec::new();
        Self::walk_node(&root, "".to_string(), &mut files);
        Self { files }
    }

    fn walk_node(node: &FSNode, prefix: String, out: &mut Vec<FileEntry>) {
        let current_path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{}/{}", prefix, node.name)
        };

        match node.node_type {
            NodeType::File => {
                out.push(FileEntry {
                    path: current_path,
                    r#type: NodeType::File,
                });
            }

            NodeType::Directory => {
                for child in &node.children {
                    Self::walk_node(child, current_path.clone(), out);
                }
            }
        }
    }
}

impl FSPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self(segments)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn to_string(&self) -> String {
        self.0.join("/")
    }
    pub fn join(&self, other: &FSPath) -> Self {
        let mut new_segments = self.0.clone();
        new_segments.extend(other.0.clone());
        Self::new(new_segments)
    }

    pub fn from_string(path: &str) -> Self {
        let segments = path
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect();

        Self::new(segments)
    }
}

pub struct FS<S: Storage> {
    pub core: Engine<S>,
}

impl<S: Storage> FS<S> {
    pub async fn readdir(&self, path: &str) -> Result<Vec<String>, FSError> {
        self.core.readdir(path).await
    }

    pub async fn write(&self, path: &str, data: Vec<u8>) -> Result<(), FSError> {
        self.core.write(path, data).await
    }
    pub fn new(
        storage: S,
        allocator: HandleAllocator,
        root_handle: FSHandle,
        root_meta: Meta,
    ) -> Self {
        let root_handle: FSHandle = allocator.new_handle();
        let root_inode = Arc::new(RootInode::new(root_handle, root_meta));
        let root = Arc::new(Dentry::new_root(root_inode));

        let core = Engine {
            root,
            storage,
            allocator,
        };

        Self { core }
    }

    pub async fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
        self.core.walk(path)
    }
}
