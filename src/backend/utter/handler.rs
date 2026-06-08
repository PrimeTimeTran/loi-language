use dyn_clone::DynClone;

use crate::{
    backend::utter::utter::{self, Utter},
    middle::ir::IR,
    registry::file_meta::FileMetadata,
};

pub trait Handler: DynClone {
    fn handle(&self, file: &FileMetadata, utter: &dyn Utter) -> Result<IR, String>;
}

dyn_clone::clone_trait_object!(Handler);

#[derive(Clone)]
pub struct HtmlHandler;
impl Handler for HtmlHandler {
    fn handle(&self, file: &FileMetadata, utter: &dyn Utter) -> Result<IR, String> {
        utter.to_ir(file)
    }
}
#[derive(Clone)]
pub struct CssHandler;
impl Handler for CssHandler {
    fn handle(&self, file: &FileMetadata, utter: &dyn Utter) -> Result<IR, String> {
        utter.to_ir(file)
    }
}
#[derive(Clone)]
pub struct JsHandler;
impl Handler for JsHandler {
    fn handle(&self, file: &FileMetadata, utter: &dyn Utter) -> Result<IR, String> {
        utter.to_ir(file)
    }
}
