#[path = "00_common/mod.rs"]
pub mod common;

#[path = "01_setup/mod.rs"]
pub mod setup;

pub mod lexer;

pub mod parser;

pub mod llvm;
// pub mod frontend;
// pub mod middle;
// pub mod backend;

pub mod pipeline;
