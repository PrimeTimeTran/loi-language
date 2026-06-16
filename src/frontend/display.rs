use core::fmt;

use crate::frontend::ast::{AST, AssignOp, BinOp, DeclKind, Expr, Stmt, UnOp};

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AST {{")?;
        for stmt in &self.stmts {
            writeln!(f, "  {}", stmt)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Let { name, kind, value } => write!(f, "let {}: {} = {};", name, kind, value),
            Stmt::Print { expr } => write!(f, "print({});", expr),
            Stmt::ExprStmt { expr } => write!(f, "{}", expr),
            Stmt::Function { name, params, body } => {
                write!(f, "fn {}({}) {{ ... }}", name, params.join(", "))
            }
            Stmt::Return { value } => match value {
                Some(v) => write!(f, "return {};", v),
                None => write!(f, "return;"),
            },
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "if ({}) {{ ... }}", condition)
            }
            Stmt::While { condition, body } => write!(f, "while ({}) {{ ... }}", condition),
            Stmt::Loop { body } => write!(f, "loop {{ ... }}"),
            Stmt::For {
                iterator,
                iterable,
                body,
            } => write!(f, "for {} in {} {{ ... }}", iterator, iterable),
            Stmt::DoWhile { body, condition } => write!(f, "do {{ ... }} while ({})", condition),
            Stmt::Block { body } => write!(f, "{{ ... }}"),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "identifier({})", name),
            _ => self.format_prec(f, 0),
        }
    }
}

impl std::fmt::Display for DeclKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclKind::MutableStatic => write!(f, "="),
            DeclKind::ImmutableStatic => write!(f, "=!"),
            DeclKind::Dynamic => write!(f, "=?"),
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Mod => write!(f, "%"),
            BinOp::Power => write!(f, "^"),
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Assign => write!(f, "="),
        }
    }
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::AddrOf => write!(f, "&"),
        }
    }
}

impl std::fmt::Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignOp::Assign => write!(f, "="),
            AssignOp::Immutable => write!(f, "=!"),
            AssignOp::Dynamic => write!(f, "=?"),
        }
    }
}
