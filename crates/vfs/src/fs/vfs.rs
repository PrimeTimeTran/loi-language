use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::fs::{Dentry, FSError, FSInput, InMemoryDirectoryInode, InMemoryFileInode, Inode, Meta};

#[derive(Deserialize)]
pub struct JsonNode {
    pub name: String,
    pub r#type: String,
    pub content: Option<String>,
    pub children: Option<Vec<JsonNode>>,
}

// pub struct VFS {
//     root: Arc<Dentry>,
// }
