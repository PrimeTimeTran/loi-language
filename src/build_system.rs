use std::{path::PathBuf, time::Instant};

use crate::{
    backend::{
        bundle::service::{BundleConfig, BundleManifest, BundleService},
        utter::registry::UtterRegistry,
    },
    registry::registry::Registry,
};

pub struct BuildContext {
    pub build_id: u64,
    pub started_at: Instant,
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub watch: bool,
    pub clean: bool,
    pub verbose: bool,
}

pub struct BuildSystem {
    pub context: BuildContext,
    pub registry: Registry,
    pub utters: UtterRegistry,
    pub bundle_service: BundleService,
}

impl BuildSystem {
    pub fn new(dir_root: PathBuf, dir_out: PathBuf) -> Self {
        let registry = Registry::scan(&dir_root);
        let utters = UtterRegistry::new();

        let config = BundleManifest {
            dir_root: dir_root.clone(),
            dir_out: dir_out.clone(),
            strip_namespace: false,
            strip_tag: true,
            strip_utter: true,
            strip_variant: true,
            strip_version: true,
            minify: true,
            remove_comments: true,
        };

        let bundle_service = BundleService::new(registry.clone(), utters.clone(), config);

        Self {
            context: BuildContext {
                build_id: 0,
                started_at: Instant::now(),
                dir_root,
                dir_out,
                watch: false,
                clean: false,
                verbose: false,
            },

            registry,
            utters,
            bundle_service,
        }
    }

    pub fn test() -> Self {
        let cwd = std::env::current_dir().unwrap();

        Self::new(cwd.join("targets/fs"), cwd.join("targets/fs_out_test"))
    }
}
