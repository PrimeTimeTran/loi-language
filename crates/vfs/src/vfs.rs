use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use wasm_bindgen::prelude::*;

use crate::{
    fs::{FsInput, Metadata},
    inode::{Dentry, InMemoryDirectoryInode, InMemoryFileInode, Inode},
};

#[derive(Deserialize)]
pub struct JsonNode {
    pub name: String,
    pub r#type: String,
    pub content: Option<String>,
    pub children: Option<Vec<JsonNode>>,
}

#[derive(Debug)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    IoError,
    AlreadyExists,
    InvalidPath,
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
