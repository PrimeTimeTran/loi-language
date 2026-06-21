#[derive(Debug)]
pub enum AnalysisError {
    Parse(String),
    UnsupportedLanguage(String),
    Io(String),
}
