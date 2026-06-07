// src/lib.rs

#![allow(warnings)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// =====================================================
// CORE COMPILER MODULE TREE
// =====================================================

// -----------------------------
// FRONTEND: lexer, parser, AST
// -----------------------------
pub mod frontend;

// -----------------------------
// MIDDLE: semantic analysis, IR, optimization
// -----------------------------
pub mod middle;

// -----------------------------
// BACKEND: code generation
// -----------------------------
pub mod backend;

// =====================================================
// TOOLING / INFRASTRUCTURE
// =====================================================

// Top-level pipeline orchestration
pub mod pipeline;

pub mod watcher;
// CLI entry helpers (library reuse)
pub mod cli;
pub mod cmd;
mod context;

// Shared diagnostics + error system
pub mod diagnostics;

// Shared compiler utilities
pub mod utils;

// Shared compiler utilities
pub mod registry;
pub mod scanner;
