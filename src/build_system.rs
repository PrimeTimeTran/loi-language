use std::path::PathBuf;

use crate::{
    backend::{
        bundle::service::{BundleConfig, BundleService},
        utter::registry::UtterRegistry,
    },
    registry::registry::Registry,
};

pub struct BuildSystem {
    pub compiler_service: BundleService,
    pub dir_out: PathBuf,
    pub dir_root: PathBuf,
    pub utters: UtterRegistry,
    pub registry: Registry,
    pub watch: bool,
}

impl BuildSystem {
    pub fn new(dir_root: PathBuf, dir_out: PathBuf) -> Self {
        let registry = Registry::scan(&dir_root);
        let utters = UtterRegistry::new();
        let config = BundleConfig {
            dir_root: dir_root.clone(),
            dir_out: dir_out.clone(),
        };

        let compiler_service = BundleService::new(registry.clone(), utters.clone(), config);

        Self {
            registry,
            utters,
            dir_root,
            dir_out,
            compiler_service,
            watch: false,
        }
    }

    pub fn test() -> Self {
        let dir_root = std::env::current_dir().unwrap().join("targets/fs");
        let dir_out = std::env::current_dir().unwrap().join("targets/fs_out_test");
        let registry = Registry::scan(&dir_root);
        let utters = UtterRegistry::new();
        let config = BundleConfig {
            dir_root: dir_root.clone(),
            dir_out: dir_out.clone(),
        };
        let compiler_service = BundleService::new(registry.clone(), utters.clone(), config);

        Self {
            watch: false,
            compiler_service,
            registry,
            utters,
            dir_root,
            dir_out,
        }
    }
}
