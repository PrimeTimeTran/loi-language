use std::collections::HashMap;

use crate::backend::utter::{CssUtter, HtmlUtter, JsUtter, Utter};

pub struct UtterRegistry {
    utters: HashMap<String, Box<dyn Utter>>,
}

impl UtterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            utters: HashMap::new(),
        };

        registry
            .utters
            .insert("ui".to_string(), Box::new(HtmlUtter));
        registry
            .utters
            .insert("css".to_string(), Box::new(CssUtter));
        registry.utters.insert("js".to_string(), Box::new(JsUtter));

        registry
    }
    pub fn get_utter(&self, capability: &str) -> Option<&dyn Utter> {
        self.utters.get(capability).map(|u| u.as_ref())
    }
}
