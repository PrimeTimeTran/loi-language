macro_rules! log_stage {
    ($stage:expr, $($arg:tt)*) => {
        println!(
            "[{:>8}] {}",
            $stage,
            format!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! tok {
    (ident $s:expr) => {
        Token::Ident($s.to_string())
    };
    (num $n:expr) => {
        Token::Number($n as f64)
    };
    (fn) => {
        Token::Function
    };
    (lparen) => {
        Token::LParen
    };
    (rparen) => {
        Token::RParen
    };
    (lbrace) => {
        Token::LBrace
    };
    (rbrace) => {
        Token::RBrace
    };
    (assign) => {
        Token::Assign
    };
    (semi) => {
        Token::Semicolon
    };
    (comma) => {
        Token::Comma
    };
}
