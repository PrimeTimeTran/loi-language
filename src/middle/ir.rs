use crate::frontend::ast::Expr;

#[derive(Debug, Clone)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
}

// -------------------------------------------------
// IR ROOT
// -------------------------------------------------
#[derive(Debug, Clone)]
pub enum IR {
    Module {
        body: Vec<IR>,
    },

    // -------------------------
    // VARIABLES
    // -------------------------
    Assign {
        name: String,
        value: TypedExpr,
    },

    Load {
        name: String,
    },

    // -------------------------
    // EXPRESSIONS (statement form)
    // -------------------------
    ExprStmt {
        expr: TypedExpr,
    },

    // -------------------------
    // I/O
    // -------------------------
    Print {
        value: TypedExpr,
    },

    // -------------------------
    // CONTROL FLOW (HIGH LEVEL)
    // -------------------------
    If {
        condition: TypedExpr,
        then_branch: Vec<IR>,
        else_branch: Vec<IR>,
    },

    While {
        condition: TypedExpr,
        body: Vec<IR>,
    },

    Block {
        body: Vec<IR>,
    },

    // -------------------------
    // LOW LEVEL BRANCHING (LLVM LATER)
    // -------------------------
    Branch {
        condition: Option<TypedExpr>,
        true_label: String,
        false_label: Option<String>,
    },

    // -------------------------
    // FUNCTIONS
    // -------------------------
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<IR>,
        return_type: Type,
    },

    Call {
        name: String,
        args: Vec<TypedExpr>,
    },

    Return {
        value: Option<TypedExpr>,
    },
    Nop,
}

// -------------------------------------------------
// TYPED EXPRESSION (SEMANTIC OUTPUT)
// -------------------------------------------------
pub struct TypedExpr(pub Expr, pub Type);

impl std::fmt::Debug for TypedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedExpr")
            .field("expr", &self.0)
            .field("ty", &self.1)
            .finish()
    }
}

impl Clone for TypedExpr {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

// -------------------------------------------------
// OPERATORS
// -------------------------------------------------
#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
}
