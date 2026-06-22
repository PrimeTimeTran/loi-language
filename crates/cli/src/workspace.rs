use std::path::PathBuf;

pub fn find() -> Option<PathBuf> {
    std::env::current_dir().ok()
}
