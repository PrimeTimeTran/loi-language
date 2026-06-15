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
        syn::Item::Fn(f) => Some(render_function(f, config, indent)),
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
pub fn render_function(f: &syn::ItemFn, config: &Config, indent: String) -> String {
    let name = f.sig.ident.to_string();
    let policy = &config.render_policy;
    let format = &config.format;
    let layout = &config.layout;

    // 1. Gatekeeper: Quick check for Summary mode
    if let ViewMode::Summary = policy.mode {
        return format!("{}{}", indent, name);
    }

    // 2. Data Collection: Gather params only if allowed
    let params: Vec<String> = if policy.include_functions && policy.include_params {
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
    } else {
        vec![]
    };

    // 3. Layout Engine: Use the triad logic
    let body = match layout.line_style {
        LineStyle::Compact => format!("{}({})", name, params.join(", ")),
        LineStyle::ExpandedParams => {
            if params.is_empty() {
                format!("{}()", name)
            } else {
                format!("{}(\n{}{}\n{})", name, indent, params.join(",\n"), indent)
            }
        }
        LineStyle::Block => format!("{}\n{}{}", name, indent, params.join("\n")),
    };

    format!("{}{}", indent, body)
}
pub fn render_struct(
    s: &syn::ItemStruct,
    config: &Config,
    indent: String,
    items: &[syn::Item],
) -> String {
    let mut output = format!("{}struct {}", indent, s.ident);
    let policy = &config.render_policy;

    if policy.include_nested_types {
        match &s.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let ty = quote::ToTokens::to_token_stream(&field.ty);
                    output.push_str(&format!("\n{}  - {}: {}", indent, name, ty));
                }
            }
            syn::Fields::Unnamed(fields) => {
                for (idx, field) in fields.unnamed.iter().enumerate() {
                    let ty = quote::ToTokens::to_token_stream(&field.ty);
                    output.push_str(&format!("\n{}  - {}: {}", indent, idx, ty));
                }
            }
            syn::Fields::Unit => {}
        }
    }

    // 2. Render Associated Methods
    // We only traverse items if policy allows
    if policy.include_functions {
        let methods: Vec<String> = items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Impl(i) = item {
                    if let syn::Type::Path(p) = &*i.self_ty {
                        if p.path.is_ident(&s.ident) {
                            return Some(render_impl_methods(i, config, indent.clone()));
                        }
                    }
                }
                None
            })
            .flatten()
            .collect();

        if !methods.is_empty() {
            output.push('\n');
            output.push_str(&methods.join("\n"));
        }
    }

    output
}
pub fn render_impl_methods(i: &syn::ItemImpl, config: &Config, indent: String) -> Vec<String> {
    i.items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(m) = item {
                let ret = match &m.sig.output {
                    syn::ReturnType::Default => "()".to_string(),
                    syn::ReturnType::Type(_, ty) => {
                        quote::ToTokens::to_token_stream(ty).to_string()
                    }
                };
                // Put return type on a new line with extra indentation
                Some(format!(
                    "{} fn {}(...)\n{}    -> {}",
                    indent, m.sig.ident, indent, ret
                ))
            } else {
                None
            }
        })
        .collect()
}
pub fn render_enum(e: &syn::ItemEnum, config: &Config, indent: String) -> String {
    let name = e.ident.to_string();
    let policy = &config.render_policy;
    let format = &config.format;

    // 1. If mode is Summary, just output the name
    if let ViewMode::Summary = policy.mode {
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
