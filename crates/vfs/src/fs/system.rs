use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::fs::{Dentry, Engine, FSError, Meta, NodeType, RootInode, Storage};

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

#[derive(Deserialize, Debug)]
pub struct OwnedNode {
    pub name: String,

    #[serde(rename = "type")]
    pub node_type: NodeType,

    #[serde(default)]
    pub children: Vec<OwnedNode>,

    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct FSFileDTO {
    pub handle_id: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct FSInput {
    pub files: Vec<FileEntry>,
}

#[derive(Deserialize, Clone)]
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

    pub fn walk_node_owned(node: &OwnedNode, prefix: String, out: &mut Vec<FileEntry>) {
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
                    Self::walk_node_owned(child, current_path.clone(), out);
                }
            }
        }
    }

    // fn walk_node(node: &OwnedNode, prefix: String, out: &mut Vec<FileEntry>) {
    //     let current_path = if prefix.is_empty() {
    //         node.name.clone()
    //     } else {
    //         format!("{}/{}", prefix, node.name)
    //     };

    //     match node.node_type {
    //         NodeType::File => {
    //             out.push(FileEntry {
    //                 path: current_path,
    //                 r#type: NodeType::File,
    //             });
    //         }

    //         NodeType::Directory => {
    //             for child in &node.children {
    //                 Self::walk_node(child, current_path.clone(), out);
    //             }
    //         }
    //     }
    // }
    pub fn from_node(root: OwnedNode) -> Self {
        let owned = root.into_owned();
        let mut files = Vec::new();
        Self::walk_owned(&owned, String::new(), &mut files);
        Self { files }
    }

    fn walk_owned(node: &OwnedNode, prefix: String, out: &mut Vec<FileEntry>) {
        let path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{}/{}", prefix, node.name)
        };

        match node.node_type {
            NodeType::File => {
                out.push(FileEntry {
                    path,
                    r#type: NodeType::File,
                });
            }
            NodeType::Directory => {
                for child in &node.children {
                    Self::walk_owned(child, path.clone(), out);
                }
            }
        }
    }
}

impl OwnedNode {
    pub fn into_owned(self) -> OwnedNode {
        OwnedNode {
            name: self.name,
            node_type: self.node_type,
            content: self.content,
            children: self.children.into_iter().map(|c| c.into_owned()).collect(),
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
    // fn find_dentry(&self, node: &Arc<Dentry>, target: &FSHandle) -> Option<Arc<Dentry>> {
    //     if node.inode.handle() == *target {
    //         return Some(node.clone());
    //     }

    //     let children = node.children.read().unwrap();

    //     for child in children.values() {
    //         if let Some(found) = self.find_dentry(child, target) {
    //             return Some(found);
    //         }
    //     }

    //     None
    // }
    pub async fn readdir(&self, path: &str) -> Result<Vec<String>, FSError> {
        web_sys::console::log_1(
            &format!(
                "[ROOT CHILDREN from readaddir] = {:?}",
                self.core
                    .root
                    .children
                    .read()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            )
            .into(),
        );
        self.core.readdir(path).await
    }

    pub async fn write(&self, path: &str, data: Vec<u8>) -> Result<(), FSError> {
        self.core.write(path, data).await
    }
    pub fn new(
        storage: S,
        allocator: HandleAllocator,
        _root_handle: FSHandle,
        root_meta: Meta,
    ) -> Self {
        let root_handle: FSHandle = allocator.new_handle();
        let root_inode = Arc::new(RootInode::new(root_handle, root_meta));
        let root = Dentry::new_root(root_inode);
        let mut index = HashMap::new();
        index.insert(root_handle, root.clone());

        let core = Engine {
            lock: Mutex::new(()),
            cwd: std::sync::RwLock::new(root_handle),
            root,
            storage,
            allocator,
            index: RwLock::new(index),
        };

        web_sys::console::log_1(
            &format!(
                "[ROOT CHILDREN from new] = {:?}",
                core.root
                    .children
                    .read()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            )
            .into(),
        );

        Self { core }
    }
    pub fn pwd(&self) -> Result<String, FSError> {
        self.core.pwd()
    }
    pub fn cd(&self, path: &str) -> Result<(), FSError> {
        self.core.cd(path)
    }

    pub async fn walk(&self, path: &str) -> Result<FSHandle, FSError> {
        self.core.walk(path)
    }
}
