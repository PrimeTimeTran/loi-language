use dyn_clone::DynClone;

use crate::{
    backend::{symbol_registry::SymbolRegistry, utter::utter::Utter},
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

#[derive(Clone)]
pub struct LoiHandler;

impl Handler for LoiHandler {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String> {
        utter.to_ir(file, symbols)
    }

    fn emit(&self, ir: &IR) -> Result<String, String> {
        // LOI is NOT a web format
        // this is ONLY for debugging fallback if accidentally routed

        match ir {
            IR::Raw(s) => Ok(s.clone()),
            IR::Structured { body, .. } => Ok(format!("{:#?}", body)),
        }
    }
}
#[derive(Clone)]
pub struct HtmlHandler;

impl Handler for HtmlHandler {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String> {
        utter.to_ir(file, symbols)
    }

    fn emit(&self, ir: &IR) -> Result<String, String> {
        Ok(match ir {
            IR::Raw(s) => s.clone(),
            IR::Structured { body, .. } => format!("{:?}", body),
        })
    }
}

#[derive(Clone)]
pub struct CssHandler;

impl Handler for CssHandler {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String> {
        utter.to_ir(file, symbols)
    }

    fn emit(&self, ir: &IR) -> Result<String, String> {
        Ok(match ir {
            IR::Raw(s) => s.clone(),
            IR::Structured { body, .. } => format!("{:?}", body),
        })
    }
}

#[derive(Clone)]
pub struct JsHandler;

impl Handler for JsHandler {
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String> {
        utter.to_ir(file, symbols)
    }

    fn emit(&self, ir: &IR) -> Result<String, String> {
        Ok(match ir {
            IR::Raw(s) => s.clone(),
            IR::Structured { body, .. } => format!("{:?}", body),
        })
    }
}
