use std::path::Path;

use anyhow::Error;

pub trait FileSystemProvider: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<String, Error>;
    fn root_path(&self) -> &Path;
}
