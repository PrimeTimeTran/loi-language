use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct Diagnostic {
    /// Human-readable message
    pub message: String,

    /// Optional machine-readable error code (useful for tooling / LSP)
    pub code: Option<String>,

    /// Where this diagnostic occurred
    pub span: Span,

    /// Severity level
    pub severity: Severity,

    /// Optional secondary notes (like Rust “help:” messages)
    pub notes: Vec<String>,

    /// Optional suggestions (future IDE integration)
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, span: Span, severity: Severity) -> Self {
        Self {
            message: message.into(),
            code: None,
            span,
            severity,
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            code: None,
            span,
            severity: Severity::Error,
            notes: vec![],
            suggestions: vec![],
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            span,
            code: None,
            notes: vec![],
            suggestions: vec![],
            severity: Severity::Warning,
            message: message.into(),
        }
    }
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticStore {
    pub halt_on_error: bool,
    pub error_count: usize,
    pub diagnostics: VecDeque<Diagnostic>,
}
impl DiagnosticStore {
    pub fn new(halt_on_error: bool) -> Self {
        Self {
            halt_on_error,
            ..Default::default()
        }
    }
    pub fn to_compile_error(&self, stage: &str) -> CompileError {
        CompileError::Stage {
            stage: stage.to_string(),
            source: Box::new(DiagnosticError(self.clone())),
        }
    }
    pub fn emit(&mut self, diag: Diagnostic) -> bool {
        if matches!(diag.severity, Severity::Error) {
            self.error_count += 1;
        }
        self.diagnostics.push_back(diag);

        self.halt_on_error && self.has_errors()
    }

    pub fn check_halt(&self) -> Result<(), CompileError> {
        if self.has_errors() {
            return Err(self.to_compile_error("frontend"));
        }
        Ok(())
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn is_empty(&self) -> bool {
        self.error_count == 0
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
    }

    // 'flush' is standard for outputting a stream
    pub fn flush(&self) {
        for diag in &self.diagnostics {
            println!("[{:?}] {}", diag.severity, diag.message);
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Logger;

impl Logger {
    pub fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }
    pub fn test() -> Self {
        Self
    }
}

#[derive(Default)]
pub struct TraceSystem;

impl TraceSystem {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Default)]
pub struct Profiler;

impl Profiler {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Default)]
pub struct Inspector;

#[derive(Default)]
pub struct CompilerEventBus;
use crate::{middle, pipeline::CompileError};
use middle::types::Span;
use std::path::PathBuf;

/// Severity of a diagnostic
#[derive(Default, Debug, Clone, Copy)]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Error,
    Hint,
}

pub type DiagnosticId = u64;

// #[derive(Debug, Clone)]
// pub enum DiagnosticSeverity {
//     Error,
//     Warning,
//     Info,
// }

impl From<DiagnosticStore> for CompileError {
    fn from(d: DiagnosticStore) -> Self {
        CompileError::Frontend(format!("{:?}", d))
    }
}

use std::fmt;

#[derive(Debug)]
pub struct DiagnosticError(pub DiagnosticStore);

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "diagnostic errors: {:?}", self.0)
    }
}

impl std::error::Error for DiagnosticError {}
