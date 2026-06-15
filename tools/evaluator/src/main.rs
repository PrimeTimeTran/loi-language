use quote::{ToTokens, quote};
use std::{
    cmp::Ordering,
    collections::HashSet,
    env, fs,
    path::{Component, Path, PathBuf},
};
use syn::{
    File, Item,
    visit::{self, Visit},
};
use walkdir::WalkDir;

use evaluator::{
    analyzer::MyAnalyzer,
    config::{Config, FormatConfig},
    extract::{DepthConstraint, Matcher, ParentConstraint, StructuralFilter, SymbolMatcher},
    format::{
        DenseConfig, ExtractMode, HeaderFormat, HeaderMode, OutputConfig, ParamFormat, PathMode,
    },
    language::{FunctionKind, SymbolKind, SymbolRegistry, TypeKind},
    ui::{format_output, render_enum, render_function, render_header, render_item, render_struct},
};

fn main() {
    let mut evaluator = Evaluator::new();
    evaluator.process_fs();
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

    fn process_fs(&mut self) {
        let all_files = self.clone().collect_and_sort_from_root();
        let mut populated_files = Vec::new();
        let mut empty_files = Vec::new();

        for file in &all_files {
            let file_content = self.render_single_file(file);
            let header = render_header_only(file, &self.root, &self.config);

            if file_content.trim() == header.trim() {
                empty_files.push(file.to_string_lossy().to_string());
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
    // fn process_file(&mut self, path: &Path) {
    //     let src = fs::read_to_string(path).unwrap_or_default();
    //     let ast = match syn::parse_file(&src) {
    //         Ok(f) => f,
    //         Err(_) => return,
    //     };

    //     let mut analyzer = MyAnalyzer {
    //         config: &self.config,
    //         items: &ast.items,
    //         rendered_output: Vec::new(),
    //         registry: SymbolRegistry::default(),
    //     };

    //     analyzer.visit_file(&ast);

    //     let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    //     let rel = abs.strip_prefix(&self.root).unwrap_or(&abs);
    //     let file_depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);

    //     let src = match fs::read_to_string(path) {
    //         Ok(s) => s,
    //         Err(_) => return,
    //     };

    //     let ast: syn::File = match syn::parse_file(&src) {
    //         Ok(f) => f,
    //         Err(_) => return,
    //     };

    //     let header = render_header(rel, file_depth, &self.config);
    //     self.output.push_str(&header);

    //     let sym_indent = indent(file_depth);

    //     let mut items: Vec<&syn::Item> = ast.items.iter().collect();

    //     let mut structs = Vec::new();
    //     let mut functions = Vec::new();

    //     for item in &items {
    //         match item {
    //             syn::Item::Struct(s) => structs.push(s),
    //             syn::Item::Fn(f) => functions.push(f),
    //             _ => {}
    //         }
    //     }

    //     items.sort_by(|a, b| {
    //         a.to_token_stream()
    //             .to_string()
    //             .cmp(&b.to_token_stream().to_string())
    //     });

    //     if !structs.is_empty() {
    //         self.output.push_str(&format!("{}STRUCTS:\n", sym_indent));
    //         for s in structs {
    //             self.output.push_str(&render_struct(
    //                 s,
    //                 &self.config,
    //                 sym_indent.clone(),
    //                 &ast.items,
    //             ));
    //         }
    //         self.output.push('\n');
    //     }

    //     if !functions.is_empty() {
    //         self.output.push_str(&format!("{}FUNCTIONS:\n", sym_indent));
    //         for f in functions {
    //             self.output
    //                 .push_str(&render_function(f, &self.config, sym_indent.clone()));
    //             self.output.push('\n');
    //         }
    //     }
    // }

    fn render_single_file(&self, path: &Path) -> String {
        let src = fs::read_to_string(path).unwrap_or_default();
        let ast = match syn::parse_file(&src) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };

        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let rel = abs.strip_prefix(&self.root).unwrap_or(&abs);
        let file_depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);
        let sym_indent = indent(file_depth);

        let header = render_header(rel, file_depth, &self.config)
            .trim_end()
            .to_string();

        let mut blocks = Vec::new();

        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let mut enums = Vec::new();

        for item in &ast.items {
            match item {
                syn::Item::Struct(s) => structs.push(s),
                syn::Item::Fn(f) => functions.push(f),
                syn::Item::Enum(e) => enums.push(e),
                _ => {}
            }
        }

        let mut add_block = |label: &str, items: Vec<String>| {
            if !items.is_empty() {
                let mut block = format!("{}{}:", sym_indent, label);
                for item_str in items {
                    block.push('\n');
                    block.push_str(&item_str);
                }
                blocks.push(block);
            }
        };

        structs.sort_by(|a, b| a.ident.to_string().cmp(&b.ident.to_string()));
        functions.sort_by(|a, b| a.sig.ident.to_string().cmp(&b.sig.ident.to_string()));
        enums.sort_by(|a, b| a.ident.to_string().cmp(&b.ident.to_string()));

        add_block(
            "STRUCTS",
            structs
                .iter()
                .map(|s| {
                    render_struct(s, &self.config, sym_indent.clone(), &ast.items)
                        .trim_end()
                        .to_string()
                })
                .collect(),
        );
        add_block(
            "ENUMS",
            enums
                .iter()
                .map(|e| {
                    render_enum(e, &self.config, sym_indent.clone())
                        .trim_end()
                        .to_string()
                })
                .collect(),
        );
        add_block(
            "FUNCTIONS",
            functions
                .iter()
                .map(|f| {
                    render_function(f, &self.config, sym_indent.clone())
                        .trim_end()
                        .to_string()
                })
                .collect(),
        );

        format!("{}\n{}", header, blocks.join("\n\n"))
    }

    fn collect_and_sort_from_root(self) -> Vec<PathBuf> {
        let root = &self.root.clone();
        let mut all_files = self.collect_files(&root.clone());

        all_files.sort_by(|a, b| {
            let a_rel = a.strip_prefix(root).unwrap_or(a);
            let b_rel = b.strip_prefix(root).unwrap_or(b);

            let a_depth = a_rel.components().count();
            let b_depth = b_rel.components().count();
            a_depth.cmp(&b_depth).then_with(|| a_rel.cmp(b_rel))
        });
        all_files
    }
    fn collect_files(self, root: &Path) -> Vec<PathBuf> {
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

fn match_symbol(item: &syn::Item, matcher: &SymbolMatcher, indent: String) -> Option<String> {
    let kind = match item {
        syn::Item::Struct(_) => SymbolKind::Type(TypeKind::Struct),
        syn::Item::Enum(_) => SymbolKind::Type(TypeKind::Enum),
        syn::Item::Fn(_) => SymbolKind::Function(FunctionKind::Free),
        _ => return None,
    };

    if !matcher.kinds.contains(&kind) {
        return None;
    }

    if let Some(structural) = &matcher.structural {
        if !passes_structural_filter(item, structural) {
            return None;
        }
    }

    let line = match item {
        syn::Item::Struct(s) => {
            format!("{}struct {} {{}}", indent, s.ident)
        }
        syn::Item::Enum(e) => {
            format!("{}enum {} {{}}", indent, e.ident)
        }
        syn::Item::Fn(f) => {
            format!("{}fn {}() {{}}", indent, f.sig.ident)
        }
        _ => return None,
    };

    Some(line)
}

fn passes_structural_filter(_item: &syn::Item, filter: &StructuralFilter) -> bool {
    match &filter.depth {
        DepthConstraint::Any => {}
        DepthConstraint::Exact(_) => {
            // you will need AST context depth tracking here
        }
        DepthConstraint::Range { .. } => {}
    }

    match &filter.parent {
        Some(ParentConstraint::Any) | None => {}
        Some(_) => {}
    }

    true
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn render_header_only(path: &Path, root: &Path, config: &Config) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let rel = abs.strip_prefix(root).unwrap_or(&abs);
    let file_depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);

    render_header(rel, file_depth, config)
}
