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
        Ok(format!("<html>{:?}</html>", ir.body))
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
        Ok(format!("/* css ir */\n{:?}", ir.body))
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
        Ok(format!("// js ir\n{:?}", ir.body))
    }
}
