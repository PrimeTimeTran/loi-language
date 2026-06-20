// Evaluator
//     |
//     +-- FileScanner
//     |
//     +-- AnalyzerRegistry
//     |       |
//     |       +-- RustAnalyzer
//     |
//     +-- AnalysisResult
//     |
//     +-- OutputFormatter
//             |
//             +-- MarkdownFormatter
//             +-- CsvFormatter
//             +-- JsonFormatter
//             +-- ExcelFormatter

use std::{
    cmp::Ordering::{Greater, Less},
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use syn::{File, parse_file, parse_str};
use walkdir::WalkDir;

use crate::{
    config::Config,
    ui::{
        format_output, render_blocks, render_header, render_header_only, render_indent,
        render_sym_item,
    },
};

pub struct Evaluator {
    config: Config,
    scanner: FileScanner,
    renderer: Box<dyn FileRenderer>,
    writer: Box<dyn OutputWriter>,
}

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

pub trait OutputWriter {
    fn write_file(&self, files: Vec<RenderedFile>, config: &Config) -> String;
}

pub struct RenderedFile {
    pub path: PathBuf,
    pub header: String,
    pub body: String,
    pub is_empty: bool,
}

pub struct MarkdownWriter;

impl OutputWriter for MarkdownWriter {
    fn write_file(&self, files: Vec<RenderedFile>, config: &Config) -> String {
        let mut populated = vec![];
        let mut empty = vec![];

        for f in files {
            if f.is_empty {
                empty.push(format!("  {}", f.path.display()));
            } else {
                populated.push(format!("--- {}\n{}", f.header, f.body));
            }
        }

        let mut out = populated.join("\n\n");

        if !empty.is_empty() {
            out.push_str("\n\n--- # EMPTY FILES\n");
            out.push_str(&empty.join("\n"));
        }

        format_output(&out, config)
    }
}

pub trait FileRenderer {
    fn render(&self, path: &Path, source: &str) -> RenderedFile;
}

pub struct RustFileRenderer {
    config: Config,
}

impl FileRenderer for RustFileRenderer {
    fn render(&self, path: &Path, source: &str) -> RenderedFile {
        let ast = syn::parse_file(source).unwrap_or_else(|_| syn::parse_str("").unwrap());

        let (rel, depth, indent) = get_path_metadata(path, &self.config.analysis_root);

        let groups = group_items(&ast, self.config.clone(), &indent);

        let header = render_header(&rel, depth, &self.config)
            .trim_end()
            .to_string();

        let body = render_blocks(groups, &indent);

        let is_empty = body.trim().is_empty();

        RenderedFile {
            path: rel,
            header,
            body,
            is_empty,
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        let config = Config::default();

        let root = config
            .analysis_root
            .canonicalize()
            .unwrap_or_else(|_| config.analysis_root.clone());

        Self {
            config: config.clone(),
            scanner: FileScanner::new(root),
            renderer: Box::new(RustFileRenderer {
                config: config.clone(),
            }),
            writer: Box::new(MarkdownWriter),
        }
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate_fs(&mut self) {
        let files = self.scanner.scan();

        let mut rendered = vec![];

        for file in files {
            let src = fs::read_to_string(&file).unwrap_or_default();
            rendered.push(self.renderer.render(&file, &src));
        }

        let output = self.writer.write_file(rendered, &self.config);

        fs::write(&self.config.output_name, output).unwrap();

        println!("Wrote {:?}", self.config.output_name);
    }
}

fn get_path_metadata(path: &Path, root: &Path) -> (PathBuf, usize, String) {
    let abs = path;

    let rel = abs.strip_prefix(root).unwrap_or(abs).to_path_buf();

    let depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);

    let indent = render_indent(depth);

    (rel, depth, indent)
}

fn group_items(
    ast: &syn::File,
    config: Config,
    sym_indent: &str,
) -> BTreeMap<&'static str, Vec<String>> {
    let mut groups: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

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

// impl Default for Evaluator {
//     fn default() -> Self {
//         let (config, root) = Self::configure_defaults();

//         Self {
//             config,
//             root,
//             output: String::new(),
//         }
//     }
// }

// impl Evaluator {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     fn configure_defaults() -> (Config, PathBuf) {
//         let mut config = Config::default();
//         config.apply_cli_args();
//         let root = config
//             .analysis_root
//             .canonicalize()
//             .unwrap_or_else(|_| config.analysis_root.clone());
//         (config, root)
//     }

//     pub fn evaluate_fs(&mut self) {
//         let all_files = self.clone().build_fs();
//         let mut populated_files = Vec::new();
//         let mut empty_files = Vec::new();

//         for file in &all_files {
//             let file_content = self.render_single_file(file);
//             let header = render_header_only(file, &self.root, &self.config);

//             if file_content.trim() == header.trim() {
//                 empty_files.push(format!("  {}", file.to_string_lossy()));
//             } else {
//                 let formatted_content = format!("--- {}", file_content.trim());
//                 populated_files.push(formatted_content);
//             }
//         }

//         let mut final_output = populated_files.join("\n\n");

//         if !empty_files.is_empty() {
//             final_output.push_str("\n\n--- # EMPTY FILES\n");
//             final_output.push_str(&empty_files.join("\n"));
//         }

//         let formatted = format_output(&final_output, &self.config);
//         fs::write(&self.config.output_name, formatted).unwrap();
//         println!("Wrote {:?}", &self.config.output_name);
//     }

//     fn render_single_file(&self, path: &Path) -> String {
//         let src = fs::read_to_string(path).unwrap_or_default();
//         let ast = parse_file(&src)
//             .ok()
//             .unwrap_or_else(|| parse_str("").unwrap());
//         // let symbols = analyzer.analyze(&src, &options)?;
//         let (rel, file_depth, sym_indent) = self.get_path_metadata(path);
//         let groups = self.group_items(&ast, &sym_indent);
//         let header = render_header(&rel, file_depth, &self.config)
//             .trim_end()
//             .to_string();
//         let body = render_blocks(groups, &sym_indent);

//         format!("{}\n{}", header, body)
//     }

//     fn get_path_metadata(&self, path: &Path) -> (PathBuf, usize, String) {
//         let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
//         let rel = abs.strip_prefix(&self.root).unwrap_or(&abs).to_path_buf();
//         let depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);
//         (rel, depth, render_indent(depth))
//     }
//     fn group_items<'a>(
//         &self,
//         ast: &'a File,
//         sym_indent: &str,
//     ) -> BTreeMap<&'static str, Vec<String>> {
//         let mut groups: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

//         for item in &ast.items {
//             if let Some((label, rendered)) =
//                 render_sym_item(self.config.clone(), item, ast, sym_indent)
//             {
//                 groups.entry(label).or_default().push(rendered);
//             }
//         }

//         for items in groups.values_mut() {
//             items.sort();
//         }
//         groups
//     }
//     fn build_fs(&self) -> Vec<PathBuf> {
//         let root = &self.root;
//         let mut all_files = self.collect_files(root);

//         all_files.sort_by(|a, b| {
//             let a_rel = a.strip_prefix(root).unwrap_or(a);
//             let b_rel = b.strip_prefix(root).unwrap_or(b);

//             let a_components: Vec<_> = a_rel.components().collect();
//             let b_components: Vec<_> = b_rel.components().collect();

//             for (a_comp, b_comp) in a_components.iter().zip(b_components.iter()) {
//                 let a_is_last = a_comp == a_components.last().unwrap();
//                 let b_is_last = b_comp == b_components.last().unwrap();

//                 if a_comp != b_comp {
//                     if a_is_last != b_is_last {
//                         return if a_is_last { Greater } else { Less };
//                     }
//                     return a_comp.cmp(b_comp);
//                 }
//             }
//             a_components.len().cmp(&b_components.len())
//         });

//         all_files
//     }
//     fn collect_files(&self, root: &Path) -> Vec<PathBuf> {
//         let mut files = vec![];
//         for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
//             let path = entry.path();
//             if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
//                 files.push(path.to_path_buf());
//             }
//         }
//         files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
//         files
//     }
// }
