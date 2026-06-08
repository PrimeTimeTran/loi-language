use crate::middle::ir::IR;
use crate::registry::file_meta::FileMetadata;
use std::collections::HashMap;

use dyn_clone::DynClone;

pub trait Utter: DynClone {
    fn name(&self) -> &str;
    fn get_flags(&self) -> HashMap<String, bool>;
    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String>;
}

dyn_clone::clone_trait_object!(Utter);

#[derive(Clone)]
pub struct UIUtter;

impl Utter for UIUtter {
    fn name(&self) -> &str {
        "html_ui"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("browser_dom".to_string(), true);
        flags.insert("allow_network".to_string(), true);
        flags.insert("fs_access".to_string(), false);
        flags.insert("db_access".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        println!("Compiling UI module: {}", metadata.name);
        Ok(IR::new())
    }
}

#[derive(Clone)]
pub struct HtmlUtter;

impl Utter for HtmlUtter {
    fn name(&self) -> &str {
        "html_ui"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("browser_dom".to_string(), true);
        flags.insert("allow_network".to_string(), true);
        flags.insert("fs_access".to_string(), false);
        flags.insert("db_access".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        println!("Compiling UI module: {}", metadata.name);
        Ok(IR::new())
    }
}

#[derive(Clone)]
pub struct CssUtter;

impl Utter for CssUtter {
    fn name(&self) -> &str {
        "style_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("scoped_styles".to_string(), true);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        Ok(IR::new())
    }
}

#[derive(Clone)]
pub struct JsUtter;

impl Utter for JsUtter {
    fn name(&self) -> &str {
        "style_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("scoped_styles".to_string(), true);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        Ok(IR::new())
    }
}
