use crate::{
    frontend::ast::AST,
    middle::ir::{IR, IROp},
};

#[derive(Default)]
pub struct FrontendPipeline;

impl FrontendPipeline {
    pub fn run(&self, source: &str) -> AST {
        AST::new()
    }
}
#[derive(Default)]
pub struct MiddlePipeline;

impl MiddlePipeline {
    pub fn run(&self, ast: AST) -> IR {
        IR {
            raw: String::new(),
            nodes: ast.stmts.into_iter().map(|_| IROp::Nop).collect(),
            symbols: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct BackendPipeline;

impl BackendPipeline {
    pub fn run(&self, ir: IR) -> Vec<u8> {
        format!("{:?}", ir.nodes).into_bytes()
    }
}
