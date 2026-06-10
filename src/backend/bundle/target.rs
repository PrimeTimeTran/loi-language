use crate::{
    backend::{
        symbol::registry::SymbolRegistry,
        utter::{handler::Handler, utter::Utter},
    },
    middle::ir::IR,
    registry::file_meta::FileMeta,
};

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
        match ir {
            IR::Raw(s) => s.clone(),
            IR::Structured { body, .. } => format!("{:?}", body),
        }
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
