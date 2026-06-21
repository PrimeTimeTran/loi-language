use quote::ToTokens;
use std::{collections::BTreeMap, path::Path};

use syn::{
    Fields, File, FnArg, ImplItem, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, Pat, ReturnType,
    Signature, Type,
};

use crate::{
    config::{Config, RenderPolicy, format_type},
    format::{HeaderFormat, LineStyle},
    mode::ViewMode,
};

pub const INDENT_STEP: &str = "    ";

pub fn render_header(rel: &Path, file_depth: usize, config: &Config) -> String {
    let path_str = rel.to_string_lossy();
    let display_path = path_str.strip_prefix("src/").unwrap_or(&path_str);

    match config.format.header {
        HeaderFormat::None => String::new(),

        HeaderFormat::Flat => {
            format!("# {}\n\n\n", display_path)
        }

        HeaderFormat::DepthHash => {
            let depth = file_depth.saturating_add(1);
            let hashes = "#".repeat(depth);

            format!("{} {}\n\n\n", hashes, display_path)
        }
    }
}
pub fn render_sym_item<'a>(
    config: Config,
    item: &'a Item,
    ast: &'a File,
    sym_indent: &str,
) -> Option<(String, String)> {
    match item {
        Item::Struct(s) => Some((
            "STRUCTS".to_string(),
            render_struct(s, &config, sym_indent.to_string(), &ast.items)
                .trim_end()
                .to_string(),
        )),

        Item::Fn(f) => Some((
            "FUNCTIONS".to_string(),
            render_signature(RenderSig::Function(f), &config, sym_indent),
        )),
        Item::Enum(e) => Some((
            "ENUMS".to_string(),
            render_enum(e, &config, sym_indent.to_string())
                .trim_end()
                .to_string(),
        )),

        _ => None,
    }
}
pub fn render_blocks(
    config: &Config,
    groups: BTreeMap<String, Vec<String>>,
    sym_indent: &str,
) -> String {
    let mut output = String::new();
    let mark = &config.format.comment_mark;

    for (category, items) in groups {
        output.push_str(&format!(
            "{}{} {}:\n",
            sym_indent,
            mark,
            category.to_uppercase()
        ));

        for item in items {
            output.push_str(&format!("{}\n", item));
        }

        output.push('\n');
    }

    output.trim_end().to_string()
}
pub fn render_struct(s: &ItemStruct, config: &Config, indent: String, items: &[Item]) -> String {
    let policy = &config.render_policy;
    let mark = &config.format.comment_mark;
    let mut output = format!("{}struct {}\n", indent, s.ident);
    let content_indent = format!("{}{}", indent, INDENT_STEP);
    if policy.include_properties {
        let props = extract_fields(s, policy)
            .into_iter()
            .map(|p| format_type(&p))
            .collect::<Vec<_>>();

        if !props.is_empty() {
            output.push_str(&format!("{}{} PROPERTIES:\n", content_indent, mark));

            output.push_str(&format!("{}{}\n\n", content_indent, props.join(", ")));
        }
    }
    if policy.include_functions {
        let mut methods = Vec::new();

        for item in items {
            if let Item::Impl(i) = item
                && let Type::Path(p) = &*i.self_ty
                && p.path.is_ident(&s.ident)
            {
                for impl_item in &i.items {
                    if let ImplItem::Fn(m) = impl_item {
                        methods.push(render_signature(
                            RenderSig::Method(&m.sig, &indent),
                            config,
                            &indent,
                        ))
                    }
                }
            }
        }

        if !methods.is_empty() {
            output.push_str(&format!("{}{} METHODS:\n", content_indent, mark));
            output.push_str(&methods.join("\n"));
            output.push_str("\n\n");
        }
    }

    output
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
pub fn render_indent(level: usize) -> String {
    INDENT_STEP.repeat(level)
}
pub fn render_signature(kind: RenderSig, config: &Config, scope: &str) -> String {
    match kind {
        RenderSig::Function(f) => config.format_signature(
            &f.sig.ident.to_string(),
            &extract_params(&f.sig, config),
            extract_ret(&f.sig),
            scope,
        ),

        RenderSig::Method(sig, struct_scope) => {
            let scope = config.method_scope(struct_scope);

            config.format_signature(
                &sig.ident.to_string(),
                &extract_params(sig, config),
                extract_ret(sig),
                &scope,
            )
        }
    }
}
pub fn render_output(output: &str, _config: &Config) -> String {
    let mut result = output.to_string();

    result = result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    result.push('\n');
    result
}

pub fn extract_fields(s: &ItemStruct, policy: &RenderPolicy) -> Vec<String> {
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
pub fn extract_params(sig: &Signature, config: &Config) -> Vec<String> {
    let policy = &config.render_policy;
    if !policy.include_params {
        return vec![];
    }

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
}
pub fn extract_ret(sig: &Signature) -> Option<String> {
    match &sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ToTokens::to_token_stream(ty).to_string()),
    }
}

pub enum RenderSig<'a> {
    Function(&'a ItemFn),
    Method(&'a Signature, &'a str),
}
