use regex::Regex;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use crate::build::artifact::{Artifact, ArtifactKind, CompiledArtifact};
use crate::build::asset_optimizer::AssetOptimizer;
use crate::build::output_resolver::OutputResolver;
use crate::{
    backend::{
        symbol::registry::SymbolRegistry,
        utter::{handler::Handler, registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

#[derive(Clone)]
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub strip_namespace: bool,
    pub strip_tag: bool,
    pub strip_utter: bool,
    pub strip_variant: bool,
    pub strip_version: bool,
    pub minify: bool,
    pub remove_comments: bool,
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            dir_root: PathBuf::from("./targets/syntax"),
            dir_out: PathBuf::from("./output/syntax"),
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

#[derive(Default)]
pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub manifest: BundleConfig,
    pub resolver: OutputResolver,
    pub optimizer: AssetOptimizer,
}

impl BundleService {
    pub fn new(registry: Registry, manifest: BundleConfig, utter: UtterRegistry) -> Self {
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
