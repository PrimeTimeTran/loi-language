use dyn_clone::DynClone;

use crate::{
    backend::{symbol::registry::SymbolRegistry, utter::utter::Utter},
    middle::ir::IR,
    registry::file_meta::FileMeta,
};

pub trait Handler: DynClone {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String>;

    fn emit(&self, ir: &IR) -> Result<String, String>;
}

dyn_clone::clone_trait_object!(Handler);

#[derive(Clone, Copy)]
pub enum RenderTarget {
    Html,
    Css,
    Js,
    Ts,
    Json,
    Md,
    Loi,
}

#[derive(Clone)]
pub struct GenericHandler {
    pub target: RenderTarget,
}

impl GenericHandler {
    fn render_ir(&self, ir: &IR) -> String {
        let mut out = String::new();

        // -------------------------------------------------
        // RAW LAYER (pass-through / foreign language blocks)
        // -------------------------------------------------
        if !ir.raw.is_empty() {
            out.push_str("=== RAW ===\n");
            out.push_str(&ir.raw);
            out.push_str("\n\n");
        }

        // -------------------------------------------------
        // METADATA
        // -------------------------------------------------
        if !ir.metadata.is_empty() {
            out.push_str("=== METADATA ===\n");
            for (k, v) in &ir.metadata {
                out.push_str(&format!("{}: {}\n", k, v));
            }
            out.push('\n');
        }

        // -------------------------------------------------
        // SYMBOLS
        // -------------------------------------------------
        if !ir.symbols.is_empty() {
            out.push_str("=== SYMBOLS ===\n");
            for (name, _sym) in &ir.symbols {
                out.push_str(&format!("{}\n", name));
            }
            out.push('\n');
        }

        // -------------------------------------------------
        // IR NODES
        // -------------------------------------------------
        out.push_str("=== NODES ===\n");
        for node in &ir.nodes {
            out.push_str(&format!("{:?}\n", node));
        }

        out
    }
}
impl Handler for GenericHandler {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String> {
        utter.to_ir(file, symbols)
    }

    fn emit(&self, ir: &IR) -> Result<String, String> {
        let out = self.render_ir(ir);

        match self.target {
            RenderTarget::Html => Ok(format!("<html>{}</html>", out)),
            RenderTarget::Css => Ok(format!("/* css */\n{}", out)),
            RenderTarget::Js => Ok(format!("// js\n{}", out)),
            RenderTarget::Ts => Ok(format!("// js\n{}", out)),
            RenderTarget::Json => Ok(format!("// js\n{}", out)),
            RenderTarget::Md => Ok(format!("// js\n{}", out)),
            RenderTarget::Loi => Ok(out),
        }
    }
}
