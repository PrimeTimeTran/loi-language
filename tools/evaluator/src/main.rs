use quote::ToTokens;
use quote::quote;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use syn::{File, Item};
use walkdir::WalkDir;

#[derive(Clone)]
enum PathMode {
    FileName,
    Relative,
    ModulePath,
}

enum HeaderMode {
    Flat,
    DepthHash,
}

#[derive(Clone)]
enum ExtractMode {
    SymbolsOnly,
    FullBody,
}

struct ExtractConfig {
    mode: ExtractMode,

    include_structs: bool,
    include_enums: bool,
    include_file_header: bool,

    input_dir: PathBuf,
    output_file: PathBuf,
    path_mode: PathMode,
    header_mode: HeaderMode,
    wrap_in_codeblock: bool,
    codeblock_lang: Option<String>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            mode: ExtractMode::SymbolsOnly,

            include_structs: true,
            include_enums: true,
            include_file_header: true,

            input_dir: PathBuf::from("./src"),
            output_file: PathBuf::from("./dump.md"),
            path_mode: PathMode::Relative,
            header_mode: HeaderMode::DepthHash,

            wrap_in_codeblock: true,
            codeblock_lang: Some("rust".to_string()),
        }
    }
}

fn main() {
    let config = ExtractConfig::default();
    let root = config
        .input_dir
        .canonicalize()
        .unwrap_or(config.input_dir.clone());
    let mut output = String::new();
    let mut all_files = collect_files(&config.input_dir);
    all_files.sort_by(|a, b| {
        let a_rel = a.strip_prefix(&root).unwrap_or(a);
        let b_rel = b.strip_prefix(&root).unwrap_or(b);

        let a_depth = a_rel.components().count();
        let b_depth = b_rel.components().count();

        a_depth.cmp(&b_depth).then_with(|| a_rel.cmp(b_rel))
    });
    for file in all_files {
        process_file(&file, &config, &mut output, root.clone());
    }

    let final_output = format_output(&output, &config);
    fs::write(&config.output_file, final_output).expect("failed to write output");
    // fs::write(&config.output_file, output.trim_end()).expect("failed to write output");
    println!("Wrote {:?}", config.output_file);
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

fn process_file(path: &Path, config: &ExtractConfig, output: &mut String, root: PathBuf) {
    let root = config
        .input_dir
        .canonicalize()
        .unwrap_or(config.input_dir.clone());

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

    let symbols = extract_symbols(&ast, config);

    if symbols.is_empty() {
        return;
    }

    let header = render_header(rel, file_depth, config);
    output.push_str(&header);

    if config.wrap_in_codeblock {
        let lang = config.codeblock_lang.as_deref().unwrap_or("");

        output.push_str(&format!("```{}\n", lang));
    }

    let sym_indent = indent(file_depth);

    let mut items: Vec<&syn::Item> = ast.items.iter().collect();
    items.sort_by(|a, b| {
        a.to_token_stream()
            .to_string()
            .cmp(&b.to_token_stream().to_string())
    });

    for item in items {
        let line = match item {
            syn::Item::Struct(s) if config.include_structs => {
                format!(
                    "{}{}struct {} {{}}",
                    sym_indent,
                    vis_to_str(&s.vis),
                    s.ident
                )
            }

            syn::Item::Enum(e) if config.include_enums => {
                format!("{}{}enum {} {{}}", sym_indent, vis_to_str(&e.vis), e.ident)
            }

            _ => continue,
        };

        output.push_str(&line);
        output.push('\n');
    }

    if config.wrap_in_codeblock {
        output.push_str("```\n\n");
    }
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

use std::cmp::Ordering;

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

fn render_header(rel: &Path, file_depth: usize, config: &ExtractConfig) -> String {
    println!("file_depth {}", file_depth);
    match config.header_mode {
        HeaderMode::Flat => {
            format!("# {}\n\n", rel.to_string_lossy())
        }

        HeaderMode::DepthHash => {
            let hashes = "#".repeat(if file_depth == 0 { 1 } else { file_depth });
            format!("{} {}\n\n", hashes, rel.to_string_lossy())
        }
    }
}

fn extract_symbols(ast: &syn::File, config: &ExtractConfig) -> Vec<String> {
    let mut symbols = vec![];

    match config.mode {
        ExtractMode::SymbolsOnly => {
            for item in &ast.items {
                match item {
                    syn::Item::Struct(s) if config.include_structs => {
                        symbols.push(format!("{}struct {} {{}}", vis_to_str(&s.vis), s.ident));
                    }

                    syn::Item::Enum(e) if config.include_enums => {
                        symbols.push(format!("{}enum {} {{}}", vis_to_str(&e.vis), e.ident));
                    }

                    _ => {}
                }
            }
        }

        ExtractMode::FullBody => {
            for item in &ast.items {
                match item {
                    syn::Item::Struct(s) if config.include_structs => {
                        symbols.push(render_struct(s, config));
                    }

                    syn::Item::Enum(e) if config.include_enums => {
                        symbols.push(render_enum(e, config));
                    }

                    _ => {}
                }
            }
        }
    }

    symbols
}

fn render_struct(s: &syn::ItemStruct, _config: &ExtractConfig) -> String {
    let body = match &s.fields {
        syn::Fields::Named(fields) => {
            let mut parts = vec![];

            for f in &fields.named {
                let name = f.ident.as_ref().unwrap().to_string();
                let ty = f.ty.to_token_stream().to_string();
                parts.push(format!("{}: {}", name, ty));
            }

            format!("{{ {} }}", parts.join(", "))
        }

        syn::Fields::Unnamed(fields) => {
            let mut parts = vec![];

            for f in &fields.unnamed {
                let ty = f.ty.to_token_stream().to_string();
                parts.push(ty);
            }

            format!("({})", parts.join(", "))
        }

        syn::Fields::Unit => "{}".to_string(),
    };

    format!("{}struct {} {}", vis_to_str(&s.vis), s.ident, body)
}

fn render_enum(e: &syn::ItemEnum, _config: &ExtractConfig) -> String {
    let mut variants = vec![];

    for v in &e.variants {
        let variant = match &v.fields {
            syn::Fields::Unit => v.ident.to_string(),
            syn::Fields::Named(_) => format!("{} {{ .. }}", v.ident),
            syn::Fields::Unnamed(_) => format!("{}(..)", v.ident),
        };

        variants.push(variant);
    }

    format!(
        "{}enum {} {{ {} }}",
        vis_to_str(&e.vis),
        e.ident,
        variants.join(", ")
    )
}

fn format_output(output: &str, config: &ExtractConfig) -> String {
    let mut result = output.to_string();

    result = result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    result.push('\n');
    result
}
