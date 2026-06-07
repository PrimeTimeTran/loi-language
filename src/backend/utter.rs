use crate::middle::ir::IR;
use crate::registry::file_meta::FileMetadata;
use std::collections::HashMap;

pub trait Utter {
    fn name(&self) -> &str;

    // Returns a map of flags this utter enables or disables
    fn get_flags(&self) -> HashMap<String, bool>;

    // The Compiler Hook: Transform raw file to IR
    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String>;
}

pub struct HtmlUtter;

impl Utter for HtmlUtter {
    fn name(&self) -> &str {
        "html_ui"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        // Enable browser-specific behaviors
        flags.insert("browser_dom".to_string(), true);
        flags.insert("allow_network".to_string(), true);
        // Disable backend-specific behaviors
        flags.insert("fs_access".to_string(), false);
        flags.insert("db_access".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        // Logic to lower specific UI elements to IR
        println!("Compiling UI module: {}", metadata.name);

        let mut ir = IR::new();
        // Here you would use metadata.capability or other fields
        // to customize the IR generation process.

        Ok(ir)
    }
}

pub struct CssUtter;

impl Utter for CssUtter {
    fn name(&self) -> &str {
        "style_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("scoped_styles".to_string(), true);
        flags.insert("minify".to_string(), true);
        flags.insert("browser_dom".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        // Logic to lower CSS selectors and rules to your CSS-specific IR
        Ok(IR::new())
    }
}

pub struct JsUtter;

impl Utter for JsUtter {
    fn name(&self) -> &str {
        "logic_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("async_enabled".to_string(), true);
        flags.insert("strict_mode".to_string(), true);
        flags.insert("allow_side_effects".to_string(), true);
        flags
    }

    fn to_ir(&self, metadata: &FileMetadata) -> Result<IR, String> {
        // Here you would hook into your JS parser
        Ok(IR::new())
    }
}
