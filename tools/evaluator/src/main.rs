use evaluator::types::{
    DenseConfig, DepthConstraint, ExtractConfig, ExtractMode, FunctionKind, HeaderFormat,
    HeaderMode, Matcher, OutputConfig, ParamFormat, ParentConstraint, PathMode, StructuralFilter,
    SymbolKind, SymbolMatcher, TypeKind,
};
use quote::{ToTokens, quote};
use std::{
    collections::HashSet,
    env, fs,
    path::{Component, Path, PathBuf},
};

use std::cmp::Ordering;

use syn::{File, Item};
use walkdir::WalkDir;

fn main() {
    let config = ExtractConfig::default();

    // canonical root once (used for stable relative paths)
    let root = config
        .input_dir
        .canonicalize()
        .unwrap_or_else(|_| config.input_dir.clone());

    // collect + sort phase (pure traversal layer)
    let mut all_files = collect_files(&config.input_dir);

    all_files.sort_by(|a, b| {
        let a_rel = a.strip_prefix(&root).unwrap_or(a);
        let b_rel = b.strip_prefix(&root).unwrap_or(b);

        let a_depth = a_rel.components().count();
        let b_depth = b_rel.components().count();

        a_depth.cmp(&b_depth).then_with(|| a_rel.cmp(b_rel))
    });

    // extraction buffer (pure output accumulator)
    let mut output = String::new();

    // execution phase (rule-based extraction engine)
    for file in all_files {
        process_file(&file, &config, &mut output);
    }

    // final formatting layer (OutputConfig responsibility)
    let final_output = format_output(&output, &config.output);

    fs::write(&config.output_file, final_output).expect("failed to write output");

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

// wrap_in_codeblock
// codeblock_lang
// include_structs
// include_enums
// wrap_in_codeblock
fn process_file(path: &Path, config: &ExtractConfig, output: &mut String) {
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

    // ---- header (now purely output-driven) ----
    let header = render_header(rel, file_depth, config);
    output.push_str(&header);

    let sym_indent = indent(file_depth);

    // ---- collect items ----
    let mut items: Vec<&syn::Item> = ast.items.iter().collect();

    items.sort_by(|a, b| {
        a.to_token_stream()
            .to_string()
            .cmp(&b.to_token_stream().to_string())
    });
    // ---- rule-based rendering ----
    for item in items {
        if let Some(rendered) = render_item(item, &config.output.dense, sym_indent.clone()) {
            output.push_str(&rendered);
            output.push('\n');
        }
    }

    output.push_str("\n");
}

// pub fn render_item(item: &syn::Item, config: &ExtractConfig, indent: String) -> Option<String> {
//     match item {
//         syn::Item::Fn(f) => {
//             // 👇 CALL HERE
//             Some(render_fn_dense(f, &config.output.dense, indent))
//         }

//         syn::Item::Struct(s) => {
//             let fields: Vec<String> = match &s.fields {
//                 syn::Fields::Named(named) => named
//                     .named
//                     .iter()
//                     .map(|f| {
//                         let name = f.ident.as_ref().unwrap().to_string();
//                         let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();

//                         match config.output.dense.structs.fields {
//                             ParamFormat::NameOnly => name,
//                             ParamFormat::NameList => name,
//                             ParamFormat::NameType => format!("{}:{}", name, ty),
//                         }
//                     })
//                     .collect(),

//                 syn::Fields::Unnamed(_) => vec![],
//                 syn::Fields::Unit => vec![],
//             };

//             Some(format!(
//                 "{}struct {} {{ {} }}",
//                 indent,
//                 s.ident,
//                 fields.join(", ")
//             ))
//         }

//         syn::Item::Enum(e) => Some(format!("{}enum {} {{}}", indent, e.ident)),

//         _ => None,
//     }
// }

fn render_item(item: &syn::Item, config: &DenseConfig, indent: String) -> Option<String> {
    match item {
        syn::Item::Fn(f) => Some(render_function(f, config, indent)),
        syn::Item::Struct(s) => Some(render_struct(s, config, indent)),
        syn::Item::Enum(e) => Some(render_enum(e, config, indent)),
        _ => None,
    }
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
    // simplified version for now

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

fn render_header(rel: &Path, file_depth: usize, config: &ExtractConfig) -> String {
    match config.output.header {
        HeaderFormat::None => String::new(),

        HeaderFormat::Flat => {
            format!("# {}\n\n", rel.to_string_lossy())
        }

        HeaderFormat::DepthHash => {
            let depth = file_depth.max(1);
            let hashes = "#".repeat(depth);

            format!("{} {}\n\n", hashes, rel.to_string_lossy())
        }
    }
}

fn format_output(output: &str, config: &OutputConfig) -> String {
    let mut result = output.to_string();

    result = result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    result.push('\n');
    result
}

// fn render_fn_dense(f: &syn::ItemFn, config: &DenseConfig, indent: String) -> String {
//     let name = f.sig.ident.to_string();
//     if matches!(config.params, ParamFormat::NameOnly) {
//         return format!("{}{}", indent, name);
//     }
//     let params: Vec<String> = f
//         .sig
//         .inputs
//         .iter()
//         .map(|input| match input {
//             syn::FnArg::Typed(pat_type) => {
//                 let param_name = match &*pat_type.pat {
//                     syn::Pat::Ident(i) => i.ident.to_string(),
//                     _ => "_".to_string(),
//                 };

//                 match config.params {
//                     ParamFormat::NameOnly => "".to_string(),

//                     ParamFormat::NameList => param_name,

//                     ParamFormat::NameType => {
//                         let ty = quote::ToTokens::to_token_stream(&pat_type.ty).to_string();
//                         format!("{}:{}", param_name, ty)
//                     }
//                 }
//             }

//             syn::FnArg::Receiver(_) => match config.params {
//                 ParamFormat::NameOnly => "self".to_string(),
//                 ParamFormat::NameList => "self".to_string(),
//                 ParamFormat::NameType => "self:Self".to_string(),
//             },
//         })
//         .collect();

//     let body = match config.params {
//         ParamFormat::NameOnly => {
//             format!("{}", name)
//         }

//         ParamFormat::NameList => {
//             format!("{}({})", name, params.join(", "))
//         }

//         ParamFormat::NameType => {
//             format!("{}({})", name, params.join(", "))
//         }
//     };

//     format!("{}{}", indent, body)
// }

fn render_function(f: &syn::ItemFn, config: &DenseConfig, indent: String) -> String {
    let name = f.sig.ident.to_string();

    let body = match config.functions.params {
        ParamFormat::NameOnly => return format!("{}{}", indent, name),

        _ => {
            let params: Vec<String> = f
                .sig
                .inputs
                .iter()
                .map(|input| match input {
                    syn::FnArg::Typed(pat_type) => {
                        let param_name = match &*pat_type.pat {
                            syn::Pat::Ident(i) => i.ident.to_string(),
                            _ => "_".to_string(),
                        };

                        match config.functions.params {
                            ParamFormat::NameList => param_name,

                            ParamFormat::NameType => {
                                let ty = quote::ToTokens::to_token_stream(&pat_type.ty).to_string();
                                format!("{}:{}", param_name, ty)
                            }

                            _ => unreachable!(),
                        }
                    }

                    syn::FnArg::Receiver(_) => "self".to_string(),
                })
                .collect();

            format!("{}({})", name, params.join(", "))
        }
    };

    format!("{}{}", indent, body)
}

fn render_struct(s: &syn::ItemStruct, config: &DenseConfig, indent: String) -> String {
    let name = s.ident.to_string();

    let fields: Vec<String> = match &s.fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let field_name = f
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or("_".to_string());

                let ty = match config.structs.fields {
                    ParamFormat::NameOnly => field_name.clone(),

                    ParamFormat::NameList => field_name,

                    ParamFormat::NameType => {
                        let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                        format!("{}:{}", field_name, ty)
                    }
                };

                ty
            })
            .collect(),

        syn::Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let field_name = format!("_{}", i);

                match config.structs.fields {
                    ParamFormat::NameOnly => field_name.clone(),

                    ParamFormat::NameList => field_name,

                    ParamFormat::NameType => {
                        let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                        format!("{}:{}", field_name, ty)
                    }
                }
            })
            .collect(),

        syn::Fields::Unit => vec![],
    };

    if fields.is_empty() {
        return format!("{}struct {}", indent, name);
    }

    format!("{}struct {}({})", indent, name, fields.join(", "))
}

fn render_enum(e: &syn::ItemEnum, config: &DenseConfig, indent: String) -> String {
    let name = e.ident.to_string();

    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let variant_name = v.ident.to_string();

            let payloads: Vec<String> = match &v.fields {
                syn::Fields::Named(named) => named
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f
                            .ident
                            .as_ref()
                            .map(|i| i.to_string())
                            .unwrap_or("_".to_string());

                        match config.enums.variants {
                            ParamFormat::NameOnly => field_name.clone(),

                            ParamFormat::NameList => field_name,

                            ParamFormat::NameType => {
                                let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                                format!("{}:{}", field_name, ty)
                            }
                        }
                    })
                    .collect(),

                syn::Fields::Unnamed(unnamed) => unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let field_name = format!("_{}", i);

                        match config.enums.variants {
                            ParamFormat::NameOnly => field_name.clone(),

                            ParamFormat::NameList => field_name,

                            ParamFormat::NameType => {
                                let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                                format!("{}:{}", field_name, ty)
                            }
                        }
                    })
                    .collect(),

                syn::Fields::Unit => vec![],
            };

            if payloads.is_empty() {
                variant_name
            } else {
                format!("{}({})", variant_name, payloads.join(", "))
            }
        })
        .collect();

    if variants.is_empty() {
        return format!("{}enum {}", indent, name);
    }

    format!("{}enum {} {{ {} }}", indent, name, variants.join(", "))
}
