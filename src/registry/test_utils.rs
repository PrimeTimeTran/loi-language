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
        backend::{compiler_service::CompilerService, utter::registry::UtterRegistry},
        context::CompileContext,
        registry::registry::Registry,
    };

    use super::*;
    pub fn setup_test_context() -> CompileContext {
        let registry = Registry::from_files(vec![]);
        let utters = UtterRegistry::new();

        CompileContext {
            compiler_service: CompilerService::new(registry.clone(), utters.clone()),
            registry,
            utters,
        }
    }
}
