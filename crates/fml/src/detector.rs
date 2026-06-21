use std::path::Path;

use crate::ir::Language;

pub struct LanguageDetector;

impl LanguageDetector {
    pub fn detect(path: &Path) -> Language {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("js") | Some("jsx") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            _ => Language::Unknown,
        }
    }
}
