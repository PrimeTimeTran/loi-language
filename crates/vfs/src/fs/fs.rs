use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::fs::{
    Dentry, FSConfig, FSError, FileSystem, Meta, VFS,
    inode::{InMemoryDirectoryInode, InMemoryFileInode},
    meta::NodeType,
    r#trait::Inode,
    vfs::JsonNode,
};

pub struct HostFS {
    pub root: PathBuf,
}
pub struct MemFS {
    pub root: PathBuf,
}

// 1. FSPath (The Address): A conceptual string/vector (/src/main.rs). It tells you where something should be.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FSPath(Vec<String>);

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
}

// 2. FSHandle (The Identifier): The key used to "grab" the data. It might be a memory pointer or a file descriptor.
#[derive(Clone, Debug)]
pub enum FSHandle {
    Uninitialized,
    Mem(String),
    Host(PathBuf),
}

impl Default for FSHandle {
    fn default() -> Self {
        Self::Uninitialized
    }
}

#[derive(Deserialize, Serialize)]
pub enum FSHandleDTO {
    Mem(String),
    Host(PathBuf),
}

pub struct FSFile {
    pub handle: FSHandle,
    pub content: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct FSFileDTO {
    pub handle_id: String,
    pub content: String,
}

pub struct FSInput {
    pub files: HashMap<String, Meta>,
}

// pub struct FsLoader;

// impl FsLoader {
//     pub async fn from_input(input: FSInput) -> VFS {
//         // 1. Create root Dentry
//         // 2. Iterate through input.files
//         // 3. Populate tree structure
//         // 4. Return the initialized MemoryFS struct
//     }
// }

pub fn build_node(node: JsonNode) -> Arc<Dentry> {
    let inode: Arc<dyn Inode> = if node.r#type == "directory" {
        Arc::new(InMemoryDirectoryInode::new())
    } else {
        Arc::new(InMemoryFileInode::new(node.content.unwrap_or_default()))
    };

    let dentry = Arc::new(Dentry::new(&node.name, inode));

    if let Some(children) = node.children {
        for child in children {
            let child_dentry = build_node(child);
            dentry
                .children
                .write()
                .unwrap()
                .insert(child_dentry.name.clone(), child_dentry);
        }
    }

    dentry
}

pub fn build_fs_from_flat_json(input: FSInput) -> Arc<Dentry> {
    let root = Arc::new(Dentry::new("/", Arc::new(InMemoryDirectoryInode::new())));

    for (path, entry) in input.files {
        let parts: Vec<String> = path
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect();
        let mut current = Arc::clone(&root);

        for (i, part) in parts.iter().enumerate() {
            let mut children = current.children.write().unwrap();

            let node = children.entry(part.clone()).or_insert_with(|| {
                // Determine type: Is this the leaf of the path provided in the JSON?
                // If it is the end of the parts array, it's the File.
                // Everything before it MUST be a directory.
                let node_type = if i == parts.len() - 1 {
                    NodeType::File
                } else {
                    NodeType::Directory
                };

                let meta = Meta::new(parts[..=i].to_vec(), node_type.clone());

                let inode: Arc<dyn Inode> = match node_type {
                    NodeType::Directory => Arc::new(InMemoryDirectoryInode::new(meta)),
                    NodeType::File => Arc::new(InMemoryFileInode::new(entry.content.clone(), meta)),
                };

                Arc::new(Dentry::new(part, inode))
            });

            current = Arc::clone(node);
        }
    }
    root
}

