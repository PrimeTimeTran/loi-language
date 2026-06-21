use std::path::PathBuf;

pub struct RenderedFile {
    pub path: PathBuf,
    pub header: String,
    pub body: String,
    pub is_empty: bool,
}
