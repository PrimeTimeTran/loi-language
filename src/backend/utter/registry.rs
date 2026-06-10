use std::collections::HashMap;

use crate::backend::bundle::target::{GenericHandler, RenderTarget};
use crate::backend::utter::{
    handler::Handler,
    utter::{Utter, UtterFlags, get_language_definitions},
};
use crate::registry::file_meta::FileMeta;

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

        let definitions = get_language_definitions();

        for utter in definitions {
            registry
                .utters
                .insert(utter.name().to_string(), Box::new(utter));
        }

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
        registry.handlers.insert(
            "ts".to_string(),
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
