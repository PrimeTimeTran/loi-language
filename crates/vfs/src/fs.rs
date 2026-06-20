use std::collections::HashMap;

use serde::Deserialize;
use wasm_bindgen::prelude::wasm_bindgen;

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
