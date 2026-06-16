use quote::{ToTokens, quote};
use std::{
    cmp::Ordering,
    collections::HashSet,
    env, fs,
    path::{Component, Path, PathBuf},
};

use syn::{File, Item};
use walkdir::WalkDir;

use crate::{
    config::{Config, RenderPolicy},
    extract::{DepthConstraint, Matcher, ParentConstraint, StructuralFilter, SymbolMatcher},
    format::{
        DenseConfig, FieldFormat, HeaderFormat, LineStyle, OutputConfig, ParamFormat, PathMode,
    },
    language::{FunctionKind, SymbolKind, TypeKind, VariableKind::Field},
    mode::ViewMode,
};

pub fn render_item(
    item: &syn::Item,
    config: &Config,
    indent: String,
    items: &[syn::Item],
) -> Option<String> {
    match item {
        syn::Item::Fn(f) => Some(format!("{}fn {}()", indent, f.sig.ident)),
        syn::Item::Struct(s) => Some(render_struct(s, config, indent, items)),
        syn::Item::Enum(e) => Some(render_enum(e, config, indent)),
        _ => None,
    }
}

pub fn render_header(rel: &Path, file_depth: usize, config: &Config) -> String {
    match config.format.header {
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
pub fn render_struct(
    s: &syn::ItemStruct,
    config: &Config,
    indent: String,
    items: &[syn::Item],
) -> String {
    let policy = &config.render_policy;
    // Define the 2-space nest explicitly
    let nest = "  ";
    let inner_indent = format!("{}{}", indent, nest);

    let mut output = format!("{}struct {}\n", indent, s.ident);

    if policy.include_properties {
        let props = collect_fields(s, policy);
        if !props.is_empty() {
            output.push_str(&format!("{}PROPERTIES:\n", inner_indent));
            output.push_str(&format!("{}{}\n\n", inner_indent, props.join(", ")));
        }
    }

    if policy.include_functions {
        let methods: Vec<String> = items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Impl(i) = item {
                    if let syn::Type::Path(p) = &*i.self_ty {
                        if p.path.is_ident(&s.ident) {
                            return Some(render_impl_methods(i, config, inner_indent.clone()));
                        }
                    }
                }
                None
            })
            .flatten()
            .collect();

        if !methods.is_empty() {
            output.push_str(&format!("{}METHODS:\n", inner_indent));
            output.push_str(&format!("{}\n\n", methods.join("\n")));
        }
    }

    output
}
pub fn render_impl_methods(i: &syn::ItemImpl, config: &Config, indent: String) -> Vec<String> {
    i.items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(m) = item {
                // Use the shared signature formatter instead of manual string building
                // Note: pass empty indent or specific indent to format_signature
                Some(format_signature(&m.sig, config, &indent))
            } else {
                None
            }
        })
        .collect()
}
pub fn render_function(f: &syn::ItemFn, config: &Config, indent: String) -> String {
    // Simply pass the signature to the shared formatter
    format_signature(&f.sig, config, &indent)
}
pub fn format_signature(sig: &syn::Signature, config: &Config, indent: &str) -> String {
    let name = sig.ident.to_string();
    let policy = &config.render_policy;

    // Extract parameters using the logic you already have
    let params: Vec<String> = if policy.include_params {
        sig.inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Typed(pt) => {
                    let p_name = match &*pt.pat {
                        syn::Pat::Ident(i) => i.ident.to_string(),
                        _ => "_".to_string(),
                    };
                    if policy.include_nested_types {
                        format!("{}: {}", p_name, quote::ToTokens::to_token_stream(&pt.ty))
                    } else {
                        p_name
                    }
                }
                syn::FnArg::Receiver(_) => "self".to_string(),
            })
            .collect()
    } else {
        vec![]
    };

    // EXTRACT RETURN TYPE
    let ret = match &sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(quote::ToTokens::to_token_stream(ty).to_string()),
    };

    // Use your existing central formatter logic
    config.format_function_signature(&name, &params, ret, indent)
}

pub fn render_enum(e: &syn::ItemEnum, config: &Config, indent: String) -> String {
    let name = e.ident.to_string();
    let policy = &config.render_policy;
    let format = &config.format;

    // 1. If mode is System, just output the name
    if let ViewMode::System = policy.mode {
        return format!("{}enum {}", indent, name);
    }

    // 2. Map the variants
    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let variant_name = v.ident.to_string();

            // Use a sub-renderer for the payload
            let payloads = render_enum_payload(&v.fields, policy);

            if payloads.is_empty() {
                variant_name
            } else {
                // Apply formatting logic based on line_style
                match format.line_style {
                    LineStyle::Compact => format!("{}({})", variant_name, payloads.join(", ")),
                    _ => format!("{}(\n  {}\n)", variant_name, payloads.join(",\n  ")),
                }
            }
        })
        .collect();

    // 3. Final layout composition
    format!("{}enum {} {{ {} }}", indent, name, variants.join(", "))
}
pub fn render_enum_payload(fields: &syn::Fields, policy: &RenderPolicy) -> Vec<String> {
    if !policy.include_nested_types {
        return vec![];
    }

    match fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                format!("{}: {}", name, ty)
            })
            .collect(),

        syn::Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                format!("_{}: {}", i, ty)
            })
            .collect(),

        syn::Fields::Unit => vec![],
    }
}
pub fn render_impl_block(i: &syn::ItemImpl, config: &DenseConfig, indent: &str) -> String {
    let mut methods = Vec::new();
    for item in &i.items {
        if let syn::ImplItem::Fn(m) = item {
            // Use your existing logic to render a function
            methods.push(format!("{}- fn {}", indent, m.sig.ident));
        }
    }
    methods.join("\n")
}

pub fn format_output(output: &str, config: &Config) -> String {
    let mut result = output.to_string();

    result = result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    result.push('\n');
    result
}

fn get_params(f: &syn::ItemFn, policy: &RenderPolicy) -> Vec<String> {
    if !policy.include_params {
        return vec![];
    }

    f.sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(pat_type) => {
                let p_name = match &*pat_type.pat {
                    syn::Pat::Ident(i) => i.ident.to_string(),
                    _ => "_".to_string(),
                };
                if policy.include_nested_types {
                    let ty = quote::ToTokens::to_token_stream(&pat_type.ty).to_string();
                    format!("{}: {}", p_name, ty)
                } else {
                    p_name
                }
            }
            syn::FnArg::Receiver(_) => "self".to_string(),
        })
        .collect()
}

fn collect_fields(s: &syn::ItemStruct, policy: &RenderPolicy) -> Vec<String> {
    match &s.fields {
        syn::Fields::Named(f) => f
            .named
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                match policy.mode {
                    ViewMode::System => name,
                    ViewMode::SystemFlow => name,
                    ViewMode::SystemFlowDetailed => {
                        let ty = quote::ToTokens::to_token_stream(&f.ty).to_string();
                        format!("{}: {}", name, ty)
                    }
                    _ => name,
                }
            })
            .collect(),
        _ => vec![],
    }
}

pub fn render_header_only(path: &Path, root: &Path, config: &Config) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let rel = abs.strip_prefix(root).unwrap_or(&abs);
    let file_depth = rel.parent().map(|p| p.components().count()).unwrap_or(0);

    render_header(rel, file_depth, config)
}

pub fn render_indent(level: usize) -> String {
    "  ".repeat(level)
}
