use std::path::Path;

use anyhow::Error;
use dyn_clone::DynClone;

pub trait CompileEngineProvider: DynClone + Send + Sync {
    // - Single file
    // - REPL command
    fn compile(&self, path: &Path) -> Result<String, Error>;
    //
    fn compile_target(&self) -> &Path;
    // Dir, recursively.
    fn bundle_target(&self) -> &Path;
    // Dir, recursively.
}
