use std::fs;
use std::path::Path;

use loi::frontend::lexer::lex;
use loi::frontend::token::Token;

pub struct LexerTestHarness {
    pub tokens: Vec<Token>,
}

impl LexerTestHarness {
    pub fn new(input: &str) -> Self {
        let tokens = lex(input).expect("Lexer failed");
        Self { tokens }
    }

    // This is the missing piece!
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
}
