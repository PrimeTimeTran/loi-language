use crate::{analyzer::AnalysisError, language::Symbol};

#[derive(Debug)]
pub struct AnalyzerOptions {
    pub include_private: bool,
    pub include_tests: bool,
}

pub trait Analyzer {
    fn analyze(
        &self,
        source: &str,
        options: &AnalyzerOptions,
    ) -> Result<Vec<Symbol>, AnalysisError>;
}
