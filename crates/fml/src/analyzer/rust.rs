use crate::{
    analyzer::{
        AnalysisError,
        r#trait::{Analyzer, AnalyzerOptions},
    },
    ir::{FunctionKind, Symbol, SymbolKind, TypeKind, Visibility},
};
use quote::ToTokens;
use syn::visit::{self, Visit};

pub struct RustAnalyzer;

impl Analyzer for RustAnalyzer {
    fn analyze(
        &self,
        source: &str,
        options: &AnalyzerOptions,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        let ast = syn::parse_file(source).map_err(|e| AnalysisError::Parse(e.to_string()))?;

        let mut visitor = RustVisitor {
            options,
            symbols: Vec::new(),
        };

        visitor.visit_file(&ast);

        Ok(visitor.symbols)
    }
}

struct RustVisitor<'a> {
    options: &'a AnalyzerOptions,
    symbols: Vec<Symbol>,
}

impl<'ast> Visit<'ast> for RustVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.add_type(item.ident.to_string(), TypeKind::Struct, &item.vis);

        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.add_type(item.ident.to_string(), TypeKind::Enum, &item.vis);

        visit::visit_item_enum(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let visibility = visibility(&item.vis);

        if self.options.include_private || matches!(visibility, Visibility::Public) {
            // Extract Rust parameters
            let params = Some(
                item.sig
                    .inputs
                    .iter()
                    .map(|arg| match arg {
                        syn::FnArg::Typed(pat) => (
                            pat.pat.to_token_stream().to_string(),
                            pat.ty.to_token_stream().to_string(),
                        ),
                        syn::FnArg::Receiver(rec) => {
                            ("self".to_string(), rec.to_token_stream().to_string())
                        }
                    })
                    .collect(),
            );

            let return_type = Some(item.sig.output.to_token_stream().to_string());

            self.symbols.push(Symbol {
                name: item.sig.ident.to_string(),
                kind: SymbolKind::Function(FunctionKind::Free),
                visibility,
                params,
                return_type,
            });
        }
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        // 1. Extract and format parameters using ToTokens
        let params = Some(
            item.sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Typed(pat) => (
                        pat.pat.to_token_stream().to_string(),
                        pat.ty.to_token_stream().to_string(),
                    ),
                    syn::FnArg::Receiver(rec) => {
                        ("self".to_string(), rec.to_token_stream().to_string())
                    }
                })
                .collect(),
        );

        let return_type = Some(item.sig.output.to_token_stream().to_string());

        self.symbols.push(Symbol {
            name: item.sig.ident.to_string(),
            kind: SymbolKind::Function(FunctionKind::Method),
            visibility: visibility(&item.vis),
            params,
            return_type,
        });

        visit::visit_impl_item_fn(self, item);
    }
}

impl RustVisitor<'_> {
    fn add_type(&mut self, name: String, kind: TypeKind, vis: &syn::Visibility) {
        let visibility = visibility(vis);

        if self.options.include_private || matches!(visibility, Visibility::Public) {
            self.symbols.push(Symbol {
                name,
                kind: SymbolKind::Type(kind),
                visibility,
                params: None,
                return_type: None,
            });
        }
    }
}

fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,

        _ => Visibility::Private,
    }
}
