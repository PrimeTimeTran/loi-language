use dyn_clone::DynClone;

use crate::{
    backend::{symbol_registry::SymbolRegistry, utter::utter::Utter},
    middle::ir::IR,
    registry::file_meta::FileMeta,
};

pub trait Handler: DynClone {
    // Added symbols: &SymbolRegistry to the signature
    fn handle(
        &self,
        file: &FileMeta,
        utter: &dyn Utter,
        symbols: &SymbolRegistry,
    ) -> Result<IR, String>;
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
}
