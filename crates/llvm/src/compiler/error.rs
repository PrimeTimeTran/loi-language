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

#[derive(Debug)]
pub enum CompileError {
    Frontend(String),
    Middle(String),
    Backend(String),
    Stage {
        stage: String,
        source: Box<dyn std::error::Error>,
    },
}
impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Frontend(e) => write!(f, "Frontend error: {}", e),
            CompileError::Middle(e) => write!(f, "Middle error: {}", e),
            CompileError::Backend(e) => write!(f, "Backend error: {}", e),
            CompileError::Stage { stage, source } => {
                write!(f, "Stage error in {}: {}", stage, source)
            }
        }
    }
}
impl std::error::Error for CompileError {}
