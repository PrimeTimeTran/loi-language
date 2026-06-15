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
    let config = Config::default();
    let root = config
        .analysis_root
        .canonicalize()
        .unwrap_or_else(|_| config.analysis_root.clone());
    let mut all_files = collect_files(&config.analysis_root);

    all_files.sort_by(|a, b| {
        let a_rel = a.strip_prefix(&root).unwrap_or(a);
        let b_rel = b.strip_prefix(&root).unwrap_or(b);

        let a_depth = a_rel.components().count();
        let b_depth = b_rel.components().count();
        a_depth.cmp(&b_depth).then_with(|| a_rel.cmp(b_rel))
    });
    let mut output = String::new();
    for file in all_files {
        process_file(&file, &config, &mut output);
    }
    let final_output = format_output(&output, &config);

    fs::write(&config.output_name, final_output).expect("failed to write output");

    println!("Wrote {:?}", config.output_name);
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
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

fn process_file(path: &Path, config: &Config, output: &mut String) {
    let src = fs::read_to_string(path).unwrap_or_default();
    let ast = match syn::parse_file(&src) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut analyzer = MyAnalyzer {
        config: &config,
        items: &ast.items,
        rendered_output: Vec::new(),
        registry: SymbolRegistry::default(),
    };

    analyzer.visit_file(&ast);

    let root = config
        .analysis_root
        .canonicalize()
        .unwrap_or(config.analysis_root.clone());

    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let rel = abs.strip_prefix(&root).unwrap_or(&abs);

    let file_depth = rel.parent().map(|p| p.components().count()).unwrap_or(0) * 2;

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return,
    };

    let ast: syn::File = match syn::parse_file(&src) {
        Ok(f) => f,
        Err(_) => return,
    };

    let header = render_header(rel, file_depth, config);
    output.push_str(&header);

    let sym_indent = indent(file_depth);

    let mut items: Vec<&syn::Item> = ast.items.iter().collect();

    items.sort_by(|a, b| {
        a.to_token_stream()
            .to_string()
            .cmp(&b.to_token_stream().to_string())
    });

    for item in items {
        if let Some(rendered) = render_item(item, &config, sym_indent.clone(), &ast.items) {
            output.push_str(&rendered);
            output.push('\n');
        }
    }

    output.push_str("\n");
}

fn match_symbol(item: &syn::Item, matcher: &SymbolMatcher, indent: String) -> Option<String> {
    let kind = match item {
        syn::Item::Struct(_) => SymbolKind::Type(TypeKind::Struct),
        syn::Item::Enum(_) => SymbolKind::Type(TypeKind::Enum),
        syn::Item::Fn(_) => SymbolKind::Function(FunctionKind::Free),
        _ => return None,
    };

    // check kind match
    if !matcher.kinds.contains(&kind) {
        return None;
    }

    // structural filter (depth/parent)
    if let Some(structural) = &matcher.structural {
        if !passes_structural_filter(item, structural) {
            return None;
        }
    }

    // render
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
        Some(_) => {
            // requires AST parent tracking (later upgrade)
        }
    }

    true
}

fn vis_to_str(vis: &syn::Visibility) -> &'static str {
    if matches!(vis, syn::Visibility::Public(_)) {
        "pub "
    } else {
        ""
    }
}

fn format_path(path: &Path, root: &Path, mode: &PathMode) -> String {
    match mode {
        PathMode::FileName => path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown.rs")
            .to_string(),

        PathMode::Relative => path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace("\\", "/"),

        PathMode::ModulePath => path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace("\\", "/")
            .replace(".rs", "")
            .replace("/", "::"),
    }
}

fn tree_cmp(a: &Path, b: &Path) -> Ordering {
    let mut a_it = a.components();
    let mut b_it = b.components();

    loop {
        match (a_it.next(), b_it.next()) {
            (Some(a_c), Some(b_c)) => {
                let a_is_dir = matches!(a_c, Component::Normal(_));
                let b_is_dir = matches!(b_c, Component::Normal(_));

                // directories first at same level
                if a_is_dir != b_is_dir {
                    return if a_is_dir {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    };
                }

                let ord = a_c.as_os_str().cmp(b_c.as_os_str());
                if ord != Ordering::Equal {
                    return ord;
                }
            }

            (None, Some(_)) => return Ordering::Less, // shorter first (parent before child)
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
}

fn depth(path: &Path, root: &Path) -> usize {
    path.strip_prefix(root).unwrap_or(path).components().count()
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}
