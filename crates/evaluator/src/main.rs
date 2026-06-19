use std::{
    cmp::Ordering::{Greater, Less},
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
use syn::{File, Item, parse_file, parse_str};
use walkdir::WalkDir;

use evaluator::{
    config::Config,
    ui::{
        format_output, render_blocks, render_header, render_header_only, render_indent,
        render_sym_item,
    },
};

fn main() {
    let mut evaluator = Evaluator::new();
    evaluator.evaluate_fs();
}

#[derive(Debug, Clone)]
struct Evaluator {
    config: Config,
    root: PathBuf,
    output: String,
}

impl Evaluator {
    fn configure_defaults() -> (Config, PathBuf) {
        let config = Config::default();
        let root = config
            .analysis_root
            .canonicalize()
            .unwrap_or_else(|_| config.analysis_root.clone());
        (config, root)
    }
    fn new() -> Self {
        let (config, root) = Self::configure_defaults();
        let output = String::new();
        Self {
            config,
            root,
            output,
        }
    }
    fn evaluate_fs(&mut self) {
        let all_files = self.clone().build_fs();
        let mut populated_files = Vec::new();
        let mut empty_files = Vec::new();

        for file in &all_files {
            let file_content = self.render_single_file(file);
            let header = render_header_only(file, &self.root, &self.config);

            if file_content.trim() == header.trim() {
                empty_files.push(format!("  {}", file.to_string_lossy()));
            } else {
                let formatted_content = format!("--- {}", file_content.trim());
                populated_files.push(formatted_content);
            }
        }

        let mut final_output = populated_files.join("\n\n");

        if !empty_files.is_empty() {
            final_output.push_str("\n\n--- # EMPTY FILES\n");
            final_output.push_str(&empty_files.join("\n"));
        }

        let formatted = format_output(&final_output, &self.config);
        fs::write(&self.config.output_name, formatted).unwrap();
        println!("Wrote {:?}", &self.config.output_name);
    }

    fn render_single_file(&self, path: &Path) -> String {
        let src = fs::read_to_string(path).unwrap_or_default();
        let ast = parse_file(&src)
            .ok()
            .unwrap_or_else(|| parse_str("").unwrap());
        let (rel, file_depth, sym_indent) = self.get_path_metadata(path);
        let groups = self.group_items(&ast, &sym_indent);
        let header = render_header(&rel, file_depth, &self.config)
            .trim_end()
            .to_string();
        let body = render_blocks(groups, &sym_indent);

        format!("{}\n{}", header, body)
    }

    fn get_path_metadata(&self, path: &Path) -> (PathBuf, usize, String) {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let rel = abs.strip_prefix(&self.root).unwrap_or(&abs).to_path_buf();
        let depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);
        (rel, depth, render_indent(depth))
    }
    fn group_items<'a>(
        &self,
        ast: &'a File,
        sym_indent: &str,
    ) -> BTreeMap<&'static str, Vec<String>> {
        let mut groups: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

        for item in &ast.items {
            if let Some((label, rendered)) =
                render_sym_item(self.config.clone(), item, ast, sym_indent)
            {
                groups.entry(label).or_default().push(rendered);
            }
        }

        for items in groups.values_mut() {
            items.sort();
        }
        groups
    }
    fn build_fs(&self) -> Vec<PathBuf> {
        let root = &self.root;
        let mut all_files = self.collect_files(root);

        all_files.sort_by(|a, b| {
            let a_rel = a.strip_prefix(root).unwrap_or(a);
            let b_rel = b.strip_prefix(root).unwrap_or(b);

            let a_components: Vec<_> = a_rel.components().collect();
            let b_components: Vec<_> = b_rel.components().collect();

            for (a_comp, b_comp) in a_components.iter().zip(b_components.iter()) {
                let a_is_last = a_comp == a_components.last().unwrap();
                let b_is_last = b_comp == b_components.last().unwrap();

                if a_comp != b_comp {
                    if a_is_last != b_is_last {
                        return if a_is_last { Greater } else { Less };
                    }
                    return a_comp.cmp(b_comp);
                }
            }
            a_components.len().cmp(&b_components.len())
        });

        all_files
    }
    fn collect_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut files = vec![];
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
                files.push(path.to_path_buf());
            }
        }
        files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        files
    }
}
