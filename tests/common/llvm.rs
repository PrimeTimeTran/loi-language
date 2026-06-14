use std::collections::HashMap;

use inkwell::context::Context;
use loi::backend::llvm::{CodeGenContext, LLVM};
use loi::frontend::ast::Expr;
use loi::middle::ir::{IROp, IrInstruction, TypedExpr};
use loi::middle::types::{Span, Type};

use crate::common::generate_binary_ir;

pub fn get_ir_string(ops: &[IROp]) -> String {
    let context = Context::create();
    let llvm = LLVM::default(&context, "test_module");
    // llvm.lower(&context, ops).expect("Failed to generate IR");
    llvm.ir()
}

pub struct IrTestHarness {
    pub ir: String,
}

impl IrTestHarness {
    pub fn new(ops: &[IROp]) -> Self {
        let context = Context::create();
        let llvm = LLVM::default(&context, "test_module");
        // llvm.lower(&context, ops).expect("lower_ir failed");
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
        middle::ir::TypedExpr,
        middle::types::{Span, Type},
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

pub fn add_var(target: &str, left: &str, right: &str) -> IrInstruction {
    let e1 = Expr::Var(left.to_string());
    let e2 = Expr::Var(right.to_string());
    let default_span = Span::default();
    // Note: ty is now a concrete Type, not an Option
    let te1 = TypedExpr {
        expr: e1,
        ty: Type::F64,
        span: default_span.clone(),
    };

    let te2 = TypedExpr {
        expr: e2,
        ty: Type::F64,
        span: default_span,
    };
    generate_binary_ir(target, te1, te2)
}
