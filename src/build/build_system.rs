use std::{path::PathBuf, time::Instant};

use crate::{
    backend::utter::registry::UtterRegistry,
    build::service::{BundleConfig, BundleService},
    init::init,
    kernel::Kernel,
    registry::registry::Registry,
};
#[derive(Debug)]
pub struct BuildContext {
    pub build_id: u64,
    pub started_at: Instant,
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub watch: bool,
    pub clean: bool,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct BuildSystem {
    pub context: BuildContext,
    pub registry: Registry,
    pub utters: UtterRegistry,
    pub bundle_service: BundleService,
}

impl BuildSystem {
    pub fn new(kernel: Kernel) -> Self {
        let config_guard = kernel.engine.config.read().unwrap();

        // 2. Access the data inside
        let dir_root = config_guard.root.clone();
        let dir_out = config_guard.output.clone();

        println!("Build root: {}", dir_root.display());
        println!("Build output: {}", dir_out.display());

        let registry = Registry::scan(dir_root.clone());
        let utters = UtterRegistry::new();

        let manifest: BundleConfig = BundleConfig {
            dir_root: dir_root.clone(),
            dir_out: dir_out.clone(),
            strip_namespace: false,
            strip_tag: false,
            strip_utter: false,
            strip_variant: false,
            strip_version: false,
            minify: true,
            remove_comments: true,
        };
        let bundle_service = BundleService::new(registry.clone(), manifest, utters.clone());

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
        let kernel = init();
        Self::new(kernel)
    }
}
