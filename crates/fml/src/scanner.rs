use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileScanner {
    root: PathBuf,
}

impl FileScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan(&self) -> Vec<PathBuf> {
        let mut files = vec![];

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() {
                files.push(path.to_path_buf());
            }
        }

        files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        files
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
