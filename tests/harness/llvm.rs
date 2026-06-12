use inkwell::context::Context;
use loi::backend::llvm::lower_ir_to_llvm;
use loi::middle::ir::IROp;

pub fn get_ir_string(ops: &[IROp]) -> String {
    let context = Context::create();
    let module = context.create_module("test_module");
    let builder = context.create_builder();

    lower_ir_to_llvm(&context, &module, &builder, ops).expect("Failed to generate IR");

    let raw_ir = module.print_to_string().to_string();

    // Sanitize the output: strip header info so tests are stable
    raw_ir
        .lines()
        .filter(|line| !line.starts_with("; ModuleID"))
        .filter(|line| !line.starts_with("source_filename"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub struct IrTestHarness {
    pub ir: String,
}

impl IrTestHarness {
    pub fn new(ops: &[IROp]) -> Self {
        Self {
            ir: get_ir_string(ops),
        }
    }

    pub fn assert_contains(&self, snippet: &str) {
        assert!(
            self.ir.contains(snippet),
            "IR missing expected snippet: '{}'\nFull IR:\n{}",
            snippet,
            self.ir
        );
    }
    pub fn assert_snapshot(&self, name: &str) {
        insta::with_settings!({
            snapshot_path => "../snapshots/ir"
        }, {
            insta::assert_snapshot!(name, self.ir);
        });
    }
}

pub mod ir_factory {
    use loi::{
        frontend::ast::Expr,
        middle::ir::{Type, TypedExpr},
    };

    use super::*;

    pub fn declare_f64(name: &str, val: f64) -> IROp {
        IROp::Declare {
            name: name.to_string(),
            value: TypedExpr(Expr::Number(val), Type::F64),
            mutable: true,
            dynamic: false,
        }
    }

    pub fn print_val(val: f64) -> IROp {
        IROp::Print {
            value: TypedExpr(Expr::Number(val), Type::F64),
        }
    }
}
