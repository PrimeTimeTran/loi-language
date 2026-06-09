#![allow(warnings)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_must_use)]

pub mod backend;
mod build_system;
pub mod cli;
pub mod cmd;
pub mod diagnostics;
pub mod frontend;
pub mod middle;
pub mod pipeline;
pub mod registry;
pub mod utils;
pub mod watcher;
