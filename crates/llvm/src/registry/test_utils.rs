#[cfg(test)]
pub mod test_helpers {
    use std::path::PathBuf;

    pub fn get_test_root() -> PathBuf {
        PathBuf::from("/virtual/root")
    }
}

#[cfg(test)]
mod test_utils {
    use crate::{backend::utter::registry::UtterRegistry, build::build_system::BuildSystem};

    pub fn setup_test_context() -> BuildSystem {
        // let registry = crate::registry::prog_registry::Registry::from_files(vec![]);
        // let utters = UtterRegistry::new();
        BuildSystem::test()
    }
}
