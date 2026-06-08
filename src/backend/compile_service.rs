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
pub struct OutputArtifact {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
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

        match file.ext.as_str() {
            "html" | "css" | "js" => {
                let s = out.to_string_lossy();
                let stripped = s.strip_suffix(".loi")?;
                Some(PathBuf::from(stripped))
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

        // 1. compiled output (html/css/js/etc)
        let web_output = handler.emit(&ir)?.into_bytes();

        let mut outputs = Vec::new();

        // 2. ALWAYS include original file (runtime .loi)
        let raw_bytes = std::fs::read(&file.path)
            .map_err(|e| format!("Failed to read {}: {}", file.path.display(), e))?;

        outputs.push(OutputArtifact {
            path: self.loi_output_path(file),
            bytes: raw_bytes,
        });

        // 3. optional web output
        if let Some(web_path) = self.web_output_path(file) {
            outputs.push(OutputArtifact {
                path: web_path,
                bytes: web_output,
            });
        }

        Ok(CompiledArtifact { ir, outputs })
    }

    // pub fn compile(&self, file: &FileMeta) -> Result<CompiledArtifact, String> {
    //     // 1. Resolve utter (default → identity engine)
    //     let cap = file.utter.as_deref().unwrap_or("identity");

    //     let utter = self
    //         .utter_registry
    //         .utters
    //         .get(cap)
    //         .or_else(|| self.utter_registry.utters.get("identity"))
    //         .ok_or_else(|| {
    //             format!(
    //                 "No utter engine for '{}' (and no identity fallback registered)",
    //                 cap
    //             )
    //         })?;

    //     // 2. Resolve handler (default → identity handler)
    //     let handler = self
    //         .utter_registry
    //         .handlers
    //         .get(&file.ext)
    //         .or_else(|| self.utter_registry.handlers.get("identity"))
    //         .ok_or_else(|| format!("No handler for .{} (and no identity handler)", file.ext))?;

    //     println!(
    //         "⚡ Compiling '{}' with utter '{}' + handler '{}'",
    //         file.name, cap, file.ext
    //     );

    //     // 3. IR stage (NEVER fails the pipeline for missing semantics)
    //     let ir = utter.to_ir(file, &self.symbols)?;

    //     // 4. Emit stage
    //     let output = handler.emit(&ir)?;

    //     Ok(CompiledArtifact {
    //         ir,
    //         output,
    //         extension: file.ext.clone(),
    //     })
    // }
}
