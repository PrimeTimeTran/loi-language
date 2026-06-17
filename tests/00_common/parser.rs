use loi::{
    compiler::diagnostic::DiagnosticStore,
    frontend::{
        ast::{AST, DeclKind, Expr, HashF64, Stmt},
        lexer::lex,
        parser::parse_program,
        token::Token,
        types::TokenStream,
    },
    tok,
};

pub struct ParserTestHarness {
    pub ast: AST,
    pub diagnostics: DiagnosticStore,
}

impl ParserTestHarness {
    pub fn new(input: &str, halt_on_error: bool) -> Self {
        let tokens = lex(input).expect("Lexer failed");
        let mut token_stream = TokenStream::new(tokens);
        let mut diagnostics = DiagnosticStore::new(halt_on_error);
        let ast = parse_program(&mut token_stream, &mut diagnostics)
            .expect("Parser failed to produce AST");

        Self { ast, diagnostics }
    }

    pub fn assert_ast(&self, expected_stmts: Vec<Stmt>) {
        assert_eq!(
            self.ast.stmts, expected_stmts,
            "AST mismatch!\nActual: {:#?}\nExpected: {:#?}",
            self.ast.stmts, expected_stmts
        );
    }
    pub fn assert_no_diagnostics(&self) {
        assert!(
            self.diagnostics.is_empty(),
            "Expected no diagnostics, but found: {:?}",
            self.diagnostics
        );
    }

    pub fn assert_stmt_count(&self, count: usize) {
        assert_eq!(self.ast.stmts.len(), count);
    }
}

pub fn fn_decl(name: &str, params: Vec<&str>, body: Vec<Stmt>) -> Stmt {
    Stmt::Function {
        name: name.to_string(),
        params: params.into_iter().map(|s| s.to_string()).collect(),
        body,
    }
}

pub fn let_decl(name: &str, kind: DeclKind, val: f64) -> Stmt {
    let token = tok!(num val);
    let number_val = match token {
        Token::Number(n) => HashF64(n),
        _ => panic!("Factory error: tok!(num) did not produce a Token::Number"),
    };

    // 3. Construct the AST node
    Stmt::Let {
        name: name.to_string(),
        kind,
        value: Expr::Number(number_val),
    }
}
