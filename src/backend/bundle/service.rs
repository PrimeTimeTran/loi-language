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
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub config: BundleConfig,
}

impl BundleService {
    pub fn new(registry: Registry, utter_registry: UtterRegistry, config: BundleConfig) -> Self {
        let mut symbols = SymbolRegistry {
            table: HashMap::new(),
        };

        symbols.build(&registry, &utter_registry.utters);

        Self {
            registry,
            utter_registry,
            symbols,
            config,
        }
    }

    fn web_output_path(&self, file: &FileMeta) -> Option<PathBuf> {
        let relative = file
            .path
            .strip_prefix(&self.config.dir_root)
            .unwrap_or(&file.path);

        let mut out = self.config.dir_out.clone();
        out.push(relative);
        let file_name = out.file_name()?.to_string_lossy();
        let base = file_name.strip_suffix(".loi")?;
        let ext = base.rsplit('.').next()?;
        match ext {
            "html" | "css" | "js" => {
                out.set_file_name(base.to_string());
                Some(out)
            }
            _ => None,
        }
    }
    fn loi_output_path(&self, file: &FileMeta) -> PathBuf {
        let relative = file
            .path
            .strip_prefix(&self.config.dir_root)
            .unwrap_or(&file.path);

        let mut out = self.config.dir_out.clone();
        out.push(relative);

        out
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

        let ir = utter.to_ir(file, &self.symbols)?;
        let web_output = handler.emit(&ir)?.into_bytes();

        let mut bundle = Vec::new();

        match (file.is_loi(), file.is_wrapped_loi()) {
            (_, true) => {
                if let Some(web_path) = self.web_output_path(file) {
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
                    path: self.loi_output_path(file),
                    bytes: raw_bytes,
                    kind: ArtifactKind::Loi,
                });
            }

            (false, false) => {
                if let Some(web_path) = self.web_output_path(file) {
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
