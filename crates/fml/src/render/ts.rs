use std::{collections::BTreeMap, path::Path};

use swc_core::ecma::ast::{
    Decl, Expr, Module, ModuleItem, ObjectPatProp, Param, Pat, Stmt, TsEntityName,
    TsKeywordTypeKind, TsType, TsTypeAnn,
};

use crate::{
    config::Config,
    parser::ParserContext,
    render::{FileRenderer, RenderedFile, get_path_metadata},
    ui::{render_blocks, render_header},
};

pub struct TypeScriptFileRenderer {
    pub config: Config,
}

impl FileRenderer for TypeScriptFileRenderer {
    fn render(&self, path: &Path, source: &str) -> RenderedFile {
        let ctx = ParserContext {
            cm: Default::default(),
        };

        let module = ctx.with_parser("input.ts", source, |parser| {
            parser.parse_module().unwrap_or_else(|_| Module {
                span: Default::default(),
                body: vec![],
                shebang: None,
            })
        });

        let (rel, depth, indent) = get_path_metadata(path, &self.config.analysis_root);
        let groups = group_items_ts(&self.config, &module, &indent);
        let header = render_header(&rel, depth, &self.config)
            .trim_end()
            .to_string();
        let body = render_blocks(&self.config, groups, &indent);
        let is_empty = body.trim().is_empty();

        RenderedFile {
            path: rel,
            header,
            body,
            is_empty,
        }
    }
}

pub fn group_items_ts(
    config: &Config,
    module: &Module,
    sym_indent: &str,
) -> BTreeMap<String, Vec<String>> {
    let mark = &config.format.comment_mark;
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for item in &module.body {
        if let ModuleItem::Stmt(Stmt::Decl(decl)) = item {
            match decl {
                Decl::Class(class) => {
                    groups
                        .entry("TYPES".to_string())
                        .or_default()
                        .push(format!("{}{}", sym_indent, class.ident.sym));
                }

                Decl::Fn(func) => {
                    let params: Vec<String> = func
                        .function
                        .params
                        .iter()
                        .map(|p| match &p.pat {
                            Pat::Ident(i) => {
                                let name = i.id.sym.to_string();
                                let type_str = i
                                    .type_ann
                                    .as_ref()
                                    .map(|ann| type_to_string(&ann.type_ann))
                                    .unwrap_or_else(|| "any".to_string());
                                format!("{}: {}", name, type_str)
                            }
                            _ => "arg: any".to_string(),
                        })
                        .collect();

                    let return_type = get_return_type(&func.function.return_type);

                    let signature = format!(
                        "function {}({}) {} {}",
                        func.ident.sym,
                        params.join(", "),
                        mark,
                        return_type
                    );

                    groups
                        .entry("FUNCTIONS".to_string())
                        .or_default()
                        .push(format!("{}{}", sym_indent, signature));
                }

                Decl::Var(var) => {
                    for d in &var.decls {
                        extract_pat(config, &d.name, &d.init, sym_indent, &mut groups);
                    }
                }

                _ => {}
            }
        }
    }

    for items in groups.values_mut() {
        items.sort();
    }

    groups
}

fn extract_pat(
    config: &Config,
    pat: &Pat,
    init: &Option<Box<Expr>>,
    sym_indent: &str,
    groups: &mut BTreeMap<String, Vec<String>>,
) {
    match pat {
        Pat::Ident(id) => {
            let name = id.id.sym.to_string();
            let mut signature = name.clone();

            if let Some(expr) = init {
                let params = match &**expr {
                    Expr::Arrow(arrow) => Some(extract_params_from_pat(&arrow.params)),
                    Expr::Fn(fn_expr) => Some(extract_params_from_params(&fn_expr.function.params)),
                    _ => None,
                };

                if let Some(p) = params {
                    let return_type = match &**expr {
                        Expr::Arrow(arrow) => get_return_type(&arrow.return_type),
                        Expr::Fn(fn_expr) => get_return_type(&fn_expr.function.return_type),
                        _ => "void".to_string(),
                    };

                    signature = format!("{}({}) // {}", name, p.join(", "), return_type);

                    groups
                        .entry("FUNCTIONS".to_string())
                        .or_default()
                        .push(format!("{}{}", sym_indent, signature));
                    return;
                }
            }

            groups
                .entry("VARIABLES".to_string())
                .or_default()
                .push(format!("{}{}", sym_indent, signature));
        }

        Pat::Array(arr) => {
            for p in arr.elems.iter().flatten() {
                extract_pat(config, p, &None, sym_indent, groups);
            }
        }

        Pat::Object(obj) => {
            for prop in &obj.props {
                if let ObjectPatProp::KeyValue(kv) = prop {
                    extract_pat(config, &kv.value, &None, sym_indent, groups);
                }
            }
        }

        _ => {}
    }
}
fn type_to_string(ts_type: &TsType) -> String {
    match ts_type {
        TsType::TsKeywordType(k) => match k.kind {
            TsKeywordTypeKind::TsStringKeyword => "string".to_string(),
            TsKeywordTypeKind::TsNumberKeyword => "number".to_string(),
            TsKeywordTypeKind::TsBooleanKeyword => "boolean".to_string(),
            _ => "any".to_string(),
        },
        TsType::TsArrayType(arr) => format!("{}[]", type_to_string(&arr.elem_type)),
        TsType::TsTypeRef(ref_type) => {
            // Get the name (e.g., "Record")
            match &ref_type.type_name {
                TsEntityName::Ident(id) => id.sym.to_string(),
                _ => "any".to_string(),
            }
        }
        _ => "any".to_string(),
    }
}

fn extract_params_from_pat(params: &[Pat]) -> Vec<String> {
    params
        .iter()
        .map(|p| match p {
            Pat::Ident(i) => i.id.sym.to_string(), // Add type extraction here if desired
            _ => "arg".to_string(),
        })
        .collect()
}

fn extract_params_from_params(params: &[Param]) -> Vec<String> {
    params
        .iter()
        .map(|p| match &p.pat {
            Pat::Ident(i) => i.id.sym.to_string(),
            _ => "arg".to_string(),
        })
        .collect()
}

fn get_return_type(rt: &Option<Box<TsTypeAnn>>) -> String {
    rt.as_ref()
        .map(|ann| type_to_string(&ann.type_ann)) // Use your existing helper
        .unwrap_or_else(|| "void".to_string())
}
