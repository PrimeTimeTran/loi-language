pub mod assertion;
pub mod harness;
pub mod helpers;
pub mod kernel;
pub mod lexer;
pub mod llvm;
pub mod mock_engine;
pub mod pipeline;
pub mod snapshot;
pub use assertion::*;
pub use harness::*;
pub use helpers::*;
pub use kernel::*;
pub use lexer::*;
pub use llvm::*;
pub use mock_engine::*;
pub use pipeline::*;
pub use snapshot::*;

use loi::test_utils;

#[test]
fn can_use_src_helpers() {
    let env = test_utils::lib_helper();

    assert_eq!(env, "Loi", "Loading a ./src/* mod failed");
}

pub fn common_mod_helper(name: &str) {
    println!("Hello, {}! This is a shared function.", name);
}
