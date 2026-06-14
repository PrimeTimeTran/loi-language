use crate::frontend::token::Token;

#[derive(Default, Debug, Clone)]
pub struct LexerState {
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Default, Debug, Clone)]
pub struct LexerConfig {
    pub allow_unicode_identifiers: bool,
    pub allow_raw_strings: bool,
    pub comment_support: bool,
}
#[derive(Debug)]
pub struct Lexer {
    pub state: LexerState,
    pub config: LexerConfig,
}
#[derive(Debug)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub pos: usize,
}
