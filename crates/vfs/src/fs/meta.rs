use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::fs::{FSHandle, FSPath};

#[derive(Clone, Debug)]
pub struct Meta {
    pub handle: FSHandle,
    pub size: u64,
    pub mode: u32,
    pub ext: String,
    pub language: String,
    pub path_abs: FSPath,
    pub path_rel: FSPath,
    pub node_type: NodeType,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            size: 0,
            mode: 0o644,
            ext: String::new(),
            language: String::new(),
            node_type: NodeType::File,
            path_abs: FSPath::empty(),
            path_rel: FSPath::empty(),
            handle: FSHandle::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    File,
    Directory,
}

impl Meta {
    pub fn is_dir(&self) -> bool {
        self.node_type == NodeType::Directory
    }

    pub fn new(path_segments: Vec<String>, node_type: NodeType) -> Self {
        let path_rel = FSPath::new(path_segments.clone());

        Self {
            node_type,
            path_rel: path_rel.clone(),
            path_abs: path_rel,
            size: 0,
            mode: 0o644,
            ext: String::new(),
            language: String::new(),
            handle: FSHandle::default(),
        }
    }

    pub fn with_handle(mut self, handle: FSHandle) -> Self {
        self.handle = handle;
        self
    }
}
