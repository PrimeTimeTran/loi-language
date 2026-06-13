use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
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
            message: message.into(),
            code: None,
            span,
            severity: Severity::Warning,
            notes: vec![],
            suggestions: vec![],
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
    pub fn check_halt(&self) -> Result<(), String> {
        if self.has_errors() {
            return Err("frontend errors".into());
        }
        Ok(())
    }
    pub fn new(halt_on_error: bool) -> Self {
        Self {
            halt_on_error,
            ..Default::default()
        }
    }

    pub fn emit(&mut self, diag: Diagnostic) -> bool {
        if matches!(diag.severity, Severity::Error) {
            self.error_count += 1;
        }
        self.diagnostics.push_back(diag);

        self.halt_on_error && self.has_errors()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
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

use std::path::PathBuf;

use crate::middle::ir::Span;

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
