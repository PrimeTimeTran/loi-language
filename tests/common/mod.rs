pub mod assertion;
pub mod harness;
pub mod helpers;
pub mod lexer;
pub mod llvm;
pub mod mock_engine;
pub mod pipeline;
#[path = "00-pulse-helper.rs"]
pub mod pulse_helper;
pub mod snapshot;

pub use assertion::*;
pub use harness::*;
pub use helpers::*;
pub use lexer::*;
pub use llvm::*;
pub use mock_engine::*;
pub use pipeline::*;
pub use snapshot::*;

use loi::test_utils;

#[test]
fn test_something() {
    let env = test_utils::lib_helper();
    // ...
}

pub fn common_mod_helper(name: &str) {
    println!("Hello, {}! This is a shared function.", name);
}
