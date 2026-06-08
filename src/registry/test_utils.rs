#[cfg(test)]
pub mod test_helpers {
    use std::path::PathBuf;

    pub fn get_test_root() -> PathBuf {
        PathBuf::from("/virtual/root")
    }
}

#[cfg(test)]
mod test_utils {
    use crate::{
        backend::{compile_service::CompilerService, utter::registry::UtterRegistry},
        build_system::BuildSystem,
        registry::registry::Registry,
    };

    use super::*;
    pub fn setup_test_context() -> BuildSystem {
        let registry = Registry::from_files(vec![]);
        let utters = UtterRegistry::new();
        BuildSystem::test()
    }
}
