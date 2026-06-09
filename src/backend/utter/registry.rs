use std::collections::HashMap;

use crate::backend::bundle::target::{GenericHandler, RenderTarget};
use crate::backend::utter::handler::Handler;
use crate::backend::utter::utter::{
    CssUtter, HtmlUtter, IdentityUtter, JsUtter, LoiUtter, UIUtter,
};
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
            .insert("loi".to_string(), Box::new(LoiUtter));
        registry
            .utters
            .insert("identity".to_string(), Box::new(IdentityUtter));
        registry
            .utters
            .insert("html".to_string(), Box::new(HtmlUtter));
        registry
            .utters
            .insert("css".to_string(), Box::new(CssUtter));

        registry.utters.insert("js".to_string(), Box::new(JsUtter));

        registry.handlers.insert(
            "loi".to_string(),
            Box::new(GenericHandler {
                target: RenderTarget::Loi,
            }),
        );

        registry.handlers.insert(
            "html".to_string(),
            Box::new(GenericHandler {
                target: RenderTarget::Html,
            }),
        );

        registry.handlers.insert(
            "css".to_string(),
            Box::new(GenericHandler {
                target: RenderTarget::Css,
            }),
        );

        registry.handlers.insert(
            "js".to_string(),
            Box::new(GenericHandler {
                target: RenderTarget::Js,
            }),
        );
        registry
    }
    pub fn get_utter(&self, capability: &str) -> Option<&dyn Utter> {
        self.utters.get(capability).map(|u| u.as_ref())
    }
}
