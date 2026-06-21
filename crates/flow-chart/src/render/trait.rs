use std::path::Path;

use crate::render::RenderedFile;

pub trait FileRenderer {
    fn render(&self, path: &Path, source: &str) -> RenderedFile;
}
