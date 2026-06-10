use regex::Regex;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    backend::{
        bundle::artifact::{Artifact, ArtifactKind, CompiledArtifact},
        symbol::registry::SymbolRegistry,
        utter::{handler::Handler, registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

#[derive(Clone)]
pub struct BundleManifest {
    // Environment (Runtime)
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,

    // Transformation Rules (Profile)
    pub strip_namespace: bool,
    pub strip_tag: bool,
    pub strip_utter: bool,
    pub strip_variant: bool,
    pub strip_version: bool,
    pub minify: bool,
    pub remove_comments: bool,
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self {
            dir_root: PathBuf::from("./"),
            dir_out: PathBuf::from("./dist"),
            strip_namespace: false,
            strip_tag: false,
            strip_utter: false,
            strip_variant: false,
            strip_version: false,
            minify: false,
            remove_comments: false,
        }
    }
}
#[derive(Clone)]
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub manifest: BundleManifest,
    pub resolver: OutputResolver,
    pub optimizer: AssetOptimizer,
}

impl BundleService {
    pub fn new(registry: Registry, manifest: BundleManifest, utter: UtterRegistry) -> Self {
        Self {
            registry,
            symbols: SymbolRegistry::new(),
            resolver: OutputResolver::new(manifest.clone()),
            optimizer: AssetOptimizer {
                minify: manifest.minify,
                remove_comments: manifest.remove_comments,
            },
            utter_registry: utter,
            manifest,
        }
    }

    pub fn rebuild_symbols(&mut self) {
        self.symbols.reset();
        self.symbols
            .build_all(&self.registry, &self.utter_registry.utters);
    }

    pub fn compile_all(
        &self,
        files: &[FileMeta],
    ) -> Vec<Result<(FileMeta, CompiledArtifact), String>> {
        let mut results = Vec::new();

        for file in files {
            let result = self.compile(file).map(|artifact| (file.clone(), artifact));

            results.push(result);
        }

        results
    }

    pub fn compile(&self, file: &FileMeta) -> Result<CompiledArtifact, String> {
        let cap = file.utter.as_deref().unwrap_or("identity");

        let utter = self
            .utter_registry
            .utters
            .get(cap)
            .or_else(|| self.utter_registry.utters.get("identity"))
            .ok_or_else(|| format!("No utter engine for '{}'", cap))?;

        let handler = self
            .utter_registry
            .handlers
            .get(&file.ext)
            .or_else(|| self.utter_registry.handlers.get("identity"))
            .ok_or_else(|| format!("No handler for .{}", file.ext))?;

        let mut ir = utter.to_ir(file, &self.symbols)?;
        ir = self.optimizer.optimize(ir, file.ext.as_str());
        let web_output = handler.emit(&ir)?.into_bytes();

        let mut bundle = Vec::new();

        match (file.is_loi(), file.is_wrapped_loi()) {
            (_, true) => {
                if let Some(web_path) = self.resolver.get_web_path(file) {
                    bundle.push(Artifact {
                        path: web_path,
                        bytes: web_output,
                        kind: ArtifactKind::Web,
                    });
                }
            }

            (true, false) => {
                let raw_bytes = std::fs::read(&file.path)
                    .map_err(|e| format!("Failed to read {}: {}", file.path.display(), e))?;

                bundle.push(Artifact {
                    path: self.resolver.get_loi_path(file),
                    bytes: raw_bytes,
                    kind: ArtifactKind::Loi,
                });
            }

            (false, false) => {
                if let Some(web_path) = self.resolver.get_web_path(file) {
                    bundle.push(Artifact {
                        path: web_path,
                        bytes: web_output,
                        kind: ArtifactKind::Web,
                    });
                }
            }
        }

        Ok(CompiledArtifact { ir, bundle })
    }
}

pub struct OutputResolver {
    manifest: BundleManifest,
}

impl OutputResolver {
    pub fn new(manifest: BundleManifest) -> Self {
        Self { manifest }
    }

    pub fn get_web_path(&self, file: &FileMeta) -> Option<PathBuf> {
        let relative_dir = file
            .path
            .parent()?
            .strip_prefix(&self.manifest.dir_root)
            .ok()?;

        let mut out = self.manifest.dir_out.join(relative_dir);
        let base_name = self.get_stripped_base_name(file);

        out.push(format!("{}.{}", base_name, file.ext));
        Some(out)
    }

    pub fn get_loi_path(&self, file: &FileMeta) -> PathBuf {
        let relative_dir = file
            .path
            .parent()
            .and_then(|p| p.strip_prefix(&self.manifest.dir_root).ok())
            .unwrap_or_else(|| Path::new(""));

        let mut out = self.manifest.dir_out.join(relative_dir);
        out.push(format!("{}.loi", self.get_stripped_base_name(file)));
        out
    }

    fn get_stripped_base_name(&self, meta: &FileMeta) -> String {
        let mut parts = Vec::new();
        let m = &self.manifest;

        if !m.strip_namespace {
            parts.push(meta.namespace.clone());
        }
        parts.push(meta.name.clone());
        if !m.strip_utter
            && let Some(u) = &meta.utter
        {
            parts.push(u.clone());
        }
        if !m.strip_variant
            && let Some(v) = &meta.variant
        {
            parts.push(v.clone());
        }
        if !m.strip_version {
            parts.push(format!("v{}", meta.version));
        }
        if !m.strip_tag
            && let Some(t) = &meta.tag
        {
            parts.push(t.clone());
        }

        parts.join(".")
    }
}

pub struct AssetOptimizer {
    pub minify: bool,
    pub remove_comments: bool,
}

impl AssetOptimizer {
    pub fn optimize(&self, ir: IR, ext: &str) -> IR {
        match ir {
            IR::Raw(content) => {
                let mut optimized = content;

                // Use the flags stored in the struct!
                if self.remove_comments {
                    optimized = self.strip_comments(&optimized, ext);
                }
                if self.minify {
                    optimized = optimized.split_whitespace().collect::<Vec<_>>().join(" ");
                }

                IR::Raw(optimized)
            }
            // Return complex IR as-is
            ir => ir,
        }
    }
    fn strip_comments(&self, content: &str, lang: &str) -> String {
        let pattern = match lang {
            "js" | "ts" | "css" => r"(?s)(//.*?\n|/\*.*?\*/)",
            _ => return content.to_string(),
        };
        // Note: In production, compile the Regex once and store it in the struct
        regex::Regex::new(pattern)
            .map(|re| re.replace_all(content, "").to_string())
            .unwrap_or_else(|_| content.to_string())
    }
}
