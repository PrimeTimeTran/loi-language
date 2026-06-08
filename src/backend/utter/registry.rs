use std::collections::HashMap;

use crate::backend::utter::handler::{CssHandler, Handler, HtmlHandler, JsHandler};
use crate::backend::utter::utter::{CssUtter, HtmlUtter, JsUtter, UIUtter};
use crate::{backend::utter::utter::Utter, registry::file_meta::FileMeta};

#[derive(Clone)]
pub struct UtterRegistry {
    pub utters: HashMap<String, Box<dyn Utter>>,
    pub handlers: HashMap<String, Box<dyn Handler>>,
}
impl PartialEq for UtterRegistry {
    fn eq(&self, other: &Self) -> bool {
        if self.utters.len() != other.utters.len() {
            return false;
        }

        self.utters.iter().all(|(k, v)| {
            other
                .utters
                .get(k)
                .map_or(false, |other_v| v.equals(other_v.as_ref()))
        })
    }
}

impl UtterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            utters: HashMap::new(),
            handlers: HashMap::new(),
        };

        registry.utters.insert("ui".to_string(), Box::new(UIUtter));
        registry
            .utters
            .insert("html".to_string(), Box::new(HtmlUtter));
        registry
            .utters
            .insert("css".to_string(), Box::new(CssUtter));

        registry.utters.insert("js".to_string(), Box::new(JsUtter));

        registry
            .handlers
            .insert("html".to_string(), Box::new(HtmlHandler));
        registry
            .handlers
            .insert("css".to_string(), Box::new(CssHandler));
        registry
            .handlers
            .insert("js".to_string(), Box::new(JsHandler));
        registry
    }
    // pub fn resolve(&self, file: &FileMeta) -> Option<&dyn Utter> {
    //     // 1. Primary: Extension (e.g., "html")
    //     // If the extension points to a handler, we use that handler to process the utter.
    //     // Note: If Handler and Utter are different types, we need to decide
    //     // how to pair them. Assuming for now we resolve to the Utter:

    //     // If you have a specific mapping, check that first:
    //     if let Some(handler) = self.handlers.get(&file.ext) {}

    //     // 2. Secondary: Utter (e.g., "ui")
    //     if let Some(c) = file.utter.as_ref() {
    //         if let Some(u) = self.utters.get(c) {
    //             return Some(u.as_ref());
    //         }
    //     }

    //     None
    // }
    pub fn get_utter(&self, capability: &str) -> Option<&dyn Utter> {
        self.utters.get(capability).map(|u| u.as_ref())
    }
    // pub fn resolve_from_filename(&self, name: &str) -> Option<&dyn Utter> {
    //     // Example: index@ui.html.loi
    //     // 1. Split by '.' to find the extension: "html"
    //     // 2. Look up "html" in your registry
    //     let ext = name.split('.').nth(1)?; // Returns "html"
    //     self.get_utter(ext)
    // }
}
