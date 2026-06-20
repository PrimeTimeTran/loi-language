use std::path::PathBuf;

use serde::Deserialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::fs::fs::{FSHandle, FSPath};

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
            handle: FSHandle::Uninitialized,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

impl Meta {
    pub fn new(path_segments: Vec<String>, node_type: NodeType) -> Self {
        let path_rel = FSPath::new(path_segments.clone());
        let handle = FSHandle::Mem(path_segments.join("/"));
        Self {
            handle,
            node_type,
            path_rel: path_rel.clone(),
            path_abs: path_rel,
            ..Default::default()
        }
    }

    pub fn is_dir(&self) -> bool {
        self.node_type == NodeType::Directory
    }
}
