use std::fs;
use std::path::Path;

use loi::frontend::{lexer::lex, token::Token};

pub struct LexerTestHarness {
    pub tokens: Vec<Token>,
}

impl LexerTestHarness {
    pub fn new(input: &str) -> Self {
        let tokens = lex(input).expect("Lexer failed");
        Self { tokens }
    }
    fn ident(s: &str) -> Token {
        Token::Ident(s.to_string())
    }

    fn num(n: i32) -> Token {
        Token::Number(n as f64)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read fixture file");
        Self::new(&content)
    }

    pub fn assert_snapshot(&self, name: &str) {
        insta::with_settings!({
            snapshot_path => "../snapshots/lexer",
        }, {
            insta::assert_debug_snapshot!(name, self.tokens);
        });
    }

    pub fn assert_has_token(&self, token: Token) {
        assert!(
            self.tokens.contains(&token),
            "Token stream missing expected token: {:?}\nFull stream: {:?}",
            token,
            self.tokens
        );
    }

    pub fn assert_no_tokens_of_type<F>(&self, predicate: F)
    where
        F: Fn(&Token) -> bool,
    {
        let found = self.tokens.iter().any(predicate);
        assert!(!found, "Token stream contains forbidden tokens!");
    }
    pub fn assert_tokens(&self, expected: Vec<Token>) {
        let mut actual = self.tokens.clone();

        if let Some(Token::EOF) = actual.last() {
            actual.pop();
        }

        assert_eq!(actual, expected);
    }
    pub fn assert_contains(&self, token: Token) {
        assert!(
            self.tokens.contains(&token),
            "Expected token {:?} not found. Full stream: {:?}",
            token,
            self.tokens
        );
    }
    pub fn assert_compound_assigns_correct(&self) {
        assert!(
            self.tokens.contains(&Token::Immutable) || self.tokens.contains(&Token::Dynamic),
            "Expected compound assignment tokens (=! or =?) missing"
        );
    }
    pub fn assert_lex(input: &str, expected: Vec<Token>) {
        let harness = Self::new(input);
        harness.assert_tokens(expected);
    }
}
