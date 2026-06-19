use std::path::PathBuf;

use crate::compiler::{
    cache::MemoryCache,
    diagnostic::{DiagnosticStore, Logger},
};

use crate::compiler::{config::Config, env::Env, state::CompileState};
use thiserror::Error;

#[derive(Debug)]
pub struct FS {
    pub root: PathBuf,
    pub cwd: PathBuf,
    pub cache_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl FS {
    pub fn source_path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    pub fn cache_path(&self) -> PathBuf {
        self.root.join(".cache")
    }

    pub fn build_path(&self) -> PathBuf {
        self.root.join("dist")
    }
}
