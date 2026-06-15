use thiserror::Error;

#[derive(serde::Serialize, Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(String),

    #[error("Lexer Error: {0}")]
    Lexer(String),

    #[error("Parser Error: {0}")]
    Parser(String),

    #[error("Analysis Error: {0}")]
    Analysis(String),

    #[error("Backend Error: {0}")]
    Backend(String),
}
