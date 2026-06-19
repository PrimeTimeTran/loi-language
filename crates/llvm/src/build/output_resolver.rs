use std::path::{Path, PathBuf};

use crate::{build::service::BundleConfig, registry::file_meta::FileMeta};

#[derive(Debug, Default)]
pub struct OutputResolver {
    manifest: BundleConfig,
}

impl OutputResolver {
    pub fn new(manifest: BundleConfig) -> Self {
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
