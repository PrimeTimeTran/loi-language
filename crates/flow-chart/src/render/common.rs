use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    config::Config,
    ui::{render_indent, render_sym_item},
};

pub fn get_path_metadata(path: &Path, root: &Path) -> (PathBuf, usize, String) {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let rel = abs
        .strip_prefix(root)
        .or_else(|_| abs.strip_prefix(root.canonicalize().unwrap_or_else(|_| root.to_path_buf())))
        .unwrap_or(&abs)
        .to_path_buf();
    let depth = rel.components().count().saturating_sub(1);

    let indent = render_indent(depth);
    (rel, depth, indent)
}
pub fn group_items(
    ast: &syn::File,
    config: Config,
    sym_indent: &str,
) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for item in &ast.items {
        if let Some((label, rendered)) = render_sym_item(config.clone(), item, ast, sym_indent) {
            groups.entry(label).or_default().push(rendered);
        }
    }

    for items in groups.values_mut() {
        items.sort();
    }

    groups
}
