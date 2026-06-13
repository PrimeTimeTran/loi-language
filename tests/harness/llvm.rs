use std::collections::HashMap;

use inkwell::context::Context;
use loi::backend::llvm::{CodegenState, LLVM};
use loi::middle::ir::IROp;

pub fn get_ir_string(ops: &[IROp]) -> String {
    let context = Context::create();

    let llvm = LLVM::default(&context, "test_module");

    llvm.lower(&context, ops).expect("Failed to generate IR");

    llvm.ir()
}

pub struct IrTestHarness {
    pub ir: String,
}

impl IrTestHarness {
    pub fn new(ops: &[IROp]) -> Self {
        let context = Context::create();
        let llvm = LLVM::default(&context, "test_module");
        llvm.lower(&context, ops).expect("lower_ir failed");
        Self { ir: llvm.ir() }
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
    use super::*;
    use loi::{
        frontend::ast::Expr,
        middle::ir::{Span, Type, TypedExpr},
    };
    fn dummy_span() -> Span {
        Span::default()
    }

    pub fn declare_f64(name: &str, val: f64) -> IROp {
        IROp::Declare {
            name: name.to_string(),
            value: TypedExpr {
                expr: Expr::Number(val),
                ty: Type::F64,
                span: dummy_span(),
            },
            mutable: true,
            dynamic: false,
        }
    }

    pub fn print_val(val: f64) -> IROp {
        IROp::Print {
            value: TypedExpr {
                expr: Expr::Number(val),
                ty: Type::F64,
                span: dummy_span(),
            },
        }
    }
}
