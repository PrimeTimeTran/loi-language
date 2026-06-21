use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use wasm_bindgen::prelude::*;

use crate::fs::{Dentry, FSError, FSInput, InMemoryDirectoryInode, InMemoryFileInode, Inode, Meta};

#[derive(Deserialize)]
pub struct JsonNode {
    pub name: String,
    pub r#type: String,
    pub content: Option<String>,
    pub children: Option<Vec<JsonNode>>,
}

pub struct VFS {
    root: Arc<Dentry>,
}

// impl FileSystem for VFS {
//     fn resolve(&self, path: &str) -> Result<Arc<Dentry>, FSError> {
//         let parts = path.trim_matches('/').split('/').filter(|x| !x.is_empty());
//         let mut current = Arc::clone(&self.root);
//         for part in parts {
//             let next = {
//                 let children = current.children.read().map_err(|_| FSError::IoError)?;

//                 children.get(part).cloned()
//             };

//             current = next.ok_or(FSError::NotFound)?;
//         }
//         Ok(current)
//     }

//     fn parent(&self, path: &str) -> Result<(Arc<Dentry>, String), FSError> {
//         let mut parts: Vec<_> = path.trim_matches('/').split('/').collect();

//         let name = parts.pop().ok_or(FSError::InvalidPath)?.to_string();

//         let parent = format!("/{}", parts.join("/"));

//         Ok((self.resolve(&parent)?, name))
//     }

//     fn new() -> VFS {
//         VFS {
//             root: build_node(JsonNode {
//                 name: "/".into(),
//                 r#type: "directory".into(),
//                 content: None,
//                 children: Some(vec![]),
//             }),
//         }
//     }
//     fn from_json(json_str: &str) -> Result<VFS, JsValue> {
//         let input: FSInput = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
//         Ok(VFS {
//             root: build_fs_from_flat_json(input),
//         })
//     }
//     fn stat(&self, path: &str) -> Result<Meta, JsValue> {
//         let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;
//         Ok(node.inode.get_stat())
//     }

//     fn read(&self, path: &str) -> Result<Vec<u8>, JsValue> {
//         let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;
//         let size = node.inode.get_stat().size;
//         let mut buf = vec![0; size as usize];
//         node.inode
//             .file_ops()
//             .read(&mut buf, 0)
//             .map_err(|e| format!("{:?}", e))?;

//         Ok(buf)
//     }

//     fn mkdir(&self, path: &str) -> Result<(), JsValue> {
//         let (parent, name) = self.parent(&path).map_err(|e| format!("{:?}", e))?;

//         let mut children = parent.children.write().unwrap();

//         if children.contains_key(&name) {
//             return Err("Already exists".into());
//         }

//         children.insert(
//             name.clone(),
//             Arc::new(Dentry::new(&name, Arc::new(InMemoryDirectoryInode::new()))),
//         );

//         Ok(())
//     }

//     fn write(&self, path: &str, data: Vec<u8>) -> Result<(), JsValue> {
//         let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;

//         node.inode
//             .file_ops()
//             .write(&data, 0)
//             .map_err(|e| format!("{:?}", e))?;

//         Ok(())
//     }

//     fn exists(&self, path: &str) -> bool {
//         self.resolve(&path).is_ok()
//     }

//     fn readdir(&self, path: &str) -> Result<Vec<String>, JsValue> {
//         let node = self.resolve(&path).map_err(|e| format!("{:?}", e))?;

//         Ok(node.children.read().unwrap().keys().cloned().collect())
//     }
// }
