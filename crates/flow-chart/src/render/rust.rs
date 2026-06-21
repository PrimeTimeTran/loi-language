use std::path::Path;

use crate::{
    config::Config,
    render::{file::RenderedFile, get_path_metadata, group_items, r#trait::FileRenderer},
    ui::{render_blocks, render_header},
};

#[derive(Clone)]
pub struct RustFileRenderer {
    pub config: Config,
}

impl FileRenderer for RustFileRenderer {
    fn render(&self, path: &Path, source: &str) -> RenderedFile {
        let ast = syn::parse_file(source).unwrap_or_else(|_| syn::parse_str("").unwrap());

        let (rel, depth, indent) = get_path_metadata(path, &self.config.analysis_root);

        let groups = group_items(&ast, self.config.clone(), &indent);

        let header = render_header(&rel, depth, &self.config)
            .trim_end()
            .to_string();

        let body = render_blocks(&self.config, groups, &indent);

        let is_empty = body.trim().is_empty();

        RenderedFile {
            path: rel,
            header,
            body,
            is_empty,
        }
    }
}
