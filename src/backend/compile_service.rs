use std::{collections::HashMap, path::PathBuf};

use crate::{
    backend::{
        symbol_registry::SymbolRegistry,
        utter::{handler::Handler, registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

pub trait UtterProvider {
    fn get_utter_for(&self, file: &FileMeta) -> Option<Box<dyn Utter>>;
}

#[derive(Clone, Debug)]
pub enum OutputKind {
    Web,
    Loi,
}

#[derive(Clone, Debug)]
pub struct OutputArtifact {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub kind: OutputKind,
}

pub struct CompiledArtifact {
    pub ir: IR,
    pub outputs: Vec<OutputArtifact>,
}
#[derive(Clone)]
pub struct CompilerConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct CompilerService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub config: CompilerConfig,
}

impl CompilerService {
    pub fn new(registry: Registry, utter_registry: UtterRegistry, config: CompilerConfig) -> Self {
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

        let new_name = if file_name.ends_with(".html.loi") {
            file_name.trim_end_matches(".loi").to_string()
        } else if file_name.ends_with(".css.loi") {
            file_name.trim_end_matches(".loi").to_string()
        } else if file_name.ends_with(".js.loi") {
            file_name.trim_end_matches(".loi").to_string()
        } else {
            return None;
        };

        out.set_file_name(new_name);
        Some(out)
    }

    // fn loi_output_path(&self, file: &FileMeta) -> PathBuf {
    //     let relative = file
    //         .path
    //         .strip_prefix(&self.config.dir_root)
    //         .unwrap_or(&file.path);

    //     let mut out = self.config.dir_out.clone();
    //     out.push(relative);

    //     out
    // }

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

        // 1. compiled output (html/css/js/etc)
        let web_output = handler.emit(&ir)?.into_bytes();

        let mut outputs = Vec::new();

        let path_str = file.path.to_string_lossy();
        let is_loi = path_str.ends_with(".loi");

        // strip ".loi" once to detect wrapper type
        let is_wrapped_loi = is_loi
            && (path_str.contains(".html.loi")
                || path_str.contains(".css.loi")
                || path_str.contains(".js.loi"));

        let web_output = handler.emit(&ir)?.into_bytes();

        let relative = file
            .path
            .strip_prefix(&self.config.dir_root)
            .unwrap_or(&file.path);

        // --------------------------------------
        // CASE 1: WRAPPED LOI (tsx-style)
        // --------------------------------------
        if is_wrapped_loi {
            // emit web file ONLY (no .loi output needed unless you explicitly want it)

            if let Some(web_path) = self.web_output_path(file) {
                outputs.push(OutputArtifact {
                    path: web_path,
                    bytes: web_output,
                    kind: OutputKind::Web,
                });
            }
        }
        // --------------------------------------
        // CASE 2: PURE LOI FILES
        // --------------------------------------
        else if is_loi {
            let raw_bytes = std::fs::read(&file.path)
                .map_err(|e| format!("Failed to read {}: {}", file.path.display(), e))?;

            let mut out = self.config.dir_out.clone();
            out.push(relative);

            outputs.push(OutputArtifact {
                path: out,
                bytes: raw_bytes,
                kind: OutputKind::Loi,
            });
        }
        // --------------------------------------
        // CASE 3: NORMAL WEB FILES (optional fallback)
        // --------------------------------------
        else {
            if let Some(web_path) = self.web_output_path(file) {
                outputs.push(OutputArtifact {
                    path: web_path,
                    bytes: web_output,
                    kind: OutputKind::Web,
                });
            }
        }
        Ok(CompiledArtifact { ir, outputs })
    }
}
