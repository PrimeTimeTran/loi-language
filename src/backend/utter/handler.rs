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
        let body = match ir {
            IR::Raw(s) => s,
            IR::Structured { body, .. } => {
                // temporary debug rendering of structured IR
                &format!("{:?}", body)
            }
        };

        Ok(format!("<loi>{}</loi>", body))
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
        let body = match ir {
            IR::Raw(s) => s,
            IR::Structured { body, .. } => {
                // temporary debug rendering of structured IR
                &format!("{:?}", body)
            }
        };

        Ok(format!("<html>{}</html>", body))
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
        let body = match ir {
            IR::Raw(s) => s,
            IR::Structured { body, .. } => &format!("{:?}", body),
        };

        Ok(format!("/* css ir */\n{}", body))
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
        let body = match ir {
            IR::Raw(s) => s,
            IR::Structured { body, .. } => &format!("{:?}", body),
        };

        Ok(format!("// js ir\n{}", body))
    }
}
