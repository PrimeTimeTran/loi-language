use quote::ToTokens;
use std::{collections::BTreeMap, path::Path};

use syn::{
    Fields, File, FnArg, ImplItem, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, Pat, ReturnType,
    Signature, Type,
};

use crate::{
    config::{Config, RenderPolicy},
    format::{HeaderFormat, LineStyle},
    mode::ViewMode,
};

pub fn render_sym_item<'a>(
    config: Config,
    item: &'a Item,
    ast: &'a File,
    sym_indent: &str,
) -> Option<(&'static str, String)> {
    match item {
        Item::Struct(s) => Some((
            "STRUCTS",
            render_struct(s, &config, sym_indent.to_string(), &ast.items)
                .trim_end()
                .to_string(),
        )),
        Item::Enum(e) => Some((
            "ENUMS",
            render_enum(e, &config, sym_indent.to_string())
                .trim_end()
                .to_string(),
        )),
        Item::Fn(f) => Some((
            "FUNCTIONS",
            render_function(f, &config, sym_indent.to_string())
                .trim_end()
                .to_string(),
        )),
        _ => None,
    }
}
pub fn render_blocks(groups: BTreeMap<&'static str, Vec<String>>, sym_indent: &str) -> String {
    groups
        .into_iter()
        .map(|(label, items)| format!("{}{}:\n{}", sym_indent, label, items.join("\n")))
        .collect::<Vec<_>>()
        .join("\n\n")
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
pub fn render_struct(s: &ItemStruct, config: &Config, indent: String, items: &[Item]) -> String {
    let policy = &config.render_policy;
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
                if let Item::Impl(i) = item
                    && let Type::Path(p) = &*i.self_ty
                    && p.path.is_ident(&s.ident)
                {
                    return Some(render_impl_methods(i, config, inner_indent.clone()));
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
pub fn render_impl_methods(i: &ItemImpl, config: &Config, indent: String) -> Vec<String> {
    i.items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(m) = item {
                // Use the shared signature formatter instead of manual string building
                // Note: pass empty indent or specific indent to format_signature
                Some(format_signature(&m.sig, config, &indent))
            } else {
                None
            }
        })
        .collect()
}
pub fn render_function(f: &ItemFn, config: &Config, indent: String) -> String {
    // Simply pass the signature to the shared formatter
    format_signature(&f.sig, config, &indent)
}
pub fn format_signature(sig: &Signature, config: &Config, indent: &str) -> String {
    let name = sig.ident.to_string();
    let policy = &config.render_policy;

    // Extract parameters using the logic you already have
    let params: Vec<String> = if policy.include_params {
        sig.inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Typed(pt) => {
                    let p_name = match &*pt.pat {
                        Pat::Ident(i) => i.ident.to_string(),
                        _ => "_".to_string(),
                    };
                    if policy.include_nested_types {
                        format!("{}: {}", p_name, ToTokens::to_token_stream(&pt.ty))
                    } else {
                        p_name
                    }
                }
                FnArg::Receiver(_) => "self".to_string(),
            })
            .collect()
    } else {
        vec![]
    };

    // EXTRACT RETURN TYPE
    let ret = match &sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ToTokens::to_token_stream(ty).to_string()),
    };

    // Use your existing central formatter logic
    config.format_function_signature(&name, &params, ret, indent)
}
pub fn render_enum(e: &ItemEnum, config: &Config, indent: String) -> String {
    let name = e.ident.to_string();
    let policy = &config.render_policy;
    let format = &config.format;

    if let ViewMode::System = policy.mode {
        return format!("{}enum {}", indent, name);
    }

    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let variant_name = v.ident.to_string();
            let payloads = render_enum_payload(&v.fields, policy);
            if payloads.is_empty() {
                variant_name
            } else {
                match format.line_style {
                    LineStyle::Compact => format!("{}({})", variant_name, payloads.join(", ")),
                    _ => format!("{}(\n  {}\n)", variant_name, payloads.join(",\n  ")),
                }
            }
        })
        .collect();
    format!("{}enum {} {{ {} }}", indent, name, variants.join(", "))
}
pub fn render_enum_payload(fields: &Fields, policy: &RenderPolicy) -> Vec<String> {
    if !policy.include_nested_types {
        return vec![];
    }

    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                let ty = ToTokens::to_token_stream(&f.ty).to_string();
                format!("{}: {}", name, ty)
            })
            .collect(),

        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let ty = ToTokens::to_token_stream(&f.ty).to_string();
                format!("_{}: {}", i, ty)
            })
            .collect(),

        Fields::Unit => vec![],
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
pub fn get_params(f: &ItemFn, policy: &RenderPolicy) -> Vec<String> {
    if !policy.include_params {
        return vec![];
    }

    f.sig
        .inputs
        .iter()
        .map(|input| match input {
            FnArg::Typed(pat_type) => {
                let p_name = match &*pat_type.pat {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => "_".to_string(),
                };
                if policy.include_nested_types {
                    let ty = ToTokens::to_token_stream(&pat_type.ty).to_string();
                    format!("{}: {}", p_name, ty)
                } else {
                    p_name
                }
            }
            FnArg::Receiver(_) => "self".to_string(),
        })
        .collect()
}
pub fn collect_fields(s: &ItemStruct, policy: &RenderPolicy) -> Vec<String> {
    match &s.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                match policy.mode {
                    ViewMode::System => name,
                    ViewMode::SystemFlow => name,
                    ViewMode::SystemFlowDetailed => {
                        let ty = ToTokens::to_token_stream(&f.ty).to_string();
                        format!("{}: {}", name, ty)
                    }
                    _ => name,
                }
            })
            .collect(),
        _ => vec![],
    }
}
