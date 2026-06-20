use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
pub struct JsonNode {
    pub name: String,
    pub r#type: String,
    pub content: Option<String>,
    pub children: Option<Vec<JsonNode>>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    pub size: u64,
    pub mode: u32,
    pub is_dir: bool,
    pub ext: String,
    pub path_abs: String,
    pub path_rel: String,
    pub language: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            size: 0,
            mode: 0o644,
            is_dir: false,
            path_abs: String::new(),
            path_rel: String::new(),
            ext: String::new(),
            language: String::new(),
        }
    }
}
#[derive(Deserialize)]
pub struct FileEntry {
    pub path_abs: String,
    pub path_rel: String,
    pub name: String,
    pub ext: String,
    pub content: String,
    pub language: String,
}

#[derive(Deserialize)]
pub struct FsInput {
    pub files: HashMap<String, FileEntry>,
}

// --- 1. Core Error & Utility Types ---
#[derive(Debug)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    IoError,
    AlreadyExists,
    InvalidPath,
}

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
    data: RwLock<Vec<u8>>,
    meta: Metadata,
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
    meta: Metadata,
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

#[wasm_bindgen]
pub struct Vfs {
    root: Arc<Dentry>,
}
impl Vfs {
    pub fn resolve(&self, path: &str) -> Result<Arc<Dentry>, VfsError> {
        let parts = path.trim_matches('/').split('/').filter(|x| !x.is_empty());

        let mut current = Arc::clone(&self.root);

        for part in parts {
            let next = {
                let children = current.children.read().map_err(|_| VfsError::IoError)?;

                children.get(part).cloned()
            };

            current = next.ok_or(VfsError::NotFound)?;
        }

        Ok(current)
    }

    pub fn parent(&self, path: &str) -> Result<(Arc<Dentry>, String), VfsError> {
        let mut parts: Vec<_> = path.trim_matches('/').split('/').collect();

        let name = parts.pop().ok_or(VfsError::InvalidPath)?.to_string();

        let parent = format!("/{}", parts.join("/"));

        Ok((self.resolve(&parent)?, name))
    }
}

#[wasm_bindgen]
impl Vfs {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Vfs {
        Vfs {
            root: build_node(JsonNode {
                name: "/".into(),
                r#type: "directory".into(),
                content: None,
                children: Some(vec![]),
            }),
        }
    }
    pub fn from_json(json_str: &str) -> Result<Vfs, JsValue> {
        let input: FsInput = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        Ok(Vfs {
            root: build_fs_from_flat_json(input),
        })
    }
    pub fn stat(&self, path: String) -> Result<Metadata, JsValue> {
        let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;
        Ok(node.inode.get_stat())
    }

    pub fn read(&self, path: String) -> Result<Vec<u8>, JsValue> {
        let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;
        let size = node.inode.get_stat().size;
        let mut buf = vec![0; size as usize];
        node.inode
            .file_ops()
            .read(&mut buf, 0)
            .map_err(|e| format!("{:?}", e))?;

        Ok(buf)
    }

    pub fn mkdir(&self, path: String) -> Result<(), JsValue> {
        let (parent, name) = self.parent(&path).map_err(|e| format!("{:?}", e))?;

        let mut children = parent.children.write().unwrap();

        if children.contains_key(&name) {
            return Err("Already exists".into());
        }

        children.insert(
            name.clone(),
            Arc::new(Dentry::new(&name, Arc::new(InMemoryDirectoryInode::new()))),
        );

        Ok(())
    }

    pub fn write(&self, path: String, data: Vec<u8>) -> Result<(), JsValue> {
        let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;

        node.inode
            .file_ops()
            .write(&data, 0)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn exists(&self, path: String) -> bool {
        self.resolve(&path).is_ok()
    }

    pub fn readdir(&self, path: String) -> Result<Vec<String>, JsValue> {
        let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;

        Ok(node.children.read().unwrap().keys().cloned().collect())
    }
}

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

pub fn build_fs_from_flat_json(input: FsInput) -> Arc<Dentry> {
    let root = Arc::new(Dentry::new("/", Arc::new(InMemoryDirectoryInode::new())));
    for (path, entry) in input.files {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut current = Arc::clone(&root);

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                let meta = Metadata {
                    is_dir: false,
                    size: entry.content.len() as u64,
                    mode: 0o644,
                    path_abs: entry.path_abs.clone(),
                    path_rel: entry.path_rel.clone(),
                    ext: entry.ext.clone(),
                    language: entry.language.clone(),
                };
                let inode = Arc::new(InMemoryFileInode {
                    data: RwLock::new(entry.content.clone().into_bytes()),
                    meta,
                });
                current
                    .children
                    .write()
                    .unwrap()
                    .insert(part.to_string(), Arc::new(Dentry::new(part, inode)));
            } else {
                let mut children = current.children.write().unwrap();
                if !children.contains_key(*part) {
                    let mut dir_meta = Metadata::default();
                    dir_meta.is_dir = true;
                    children.insert(
                        part.to_string(),
                        Arc::new(Dentry::new(
                            part,
                            Arc::new(InMemoryDirectoryInode { meta: dir_meta }),
                        )),
                    );
                }

                let next_node = Arc::clone(children.get(*part).unwrap());

                drop(children);

                current = next_node;
            }
        }
    }
    root
}
