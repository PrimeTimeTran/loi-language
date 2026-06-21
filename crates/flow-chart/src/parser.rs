use swc_core::common::{FileName, SourceMap, sync::Lrc};
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{Parser, StringInput, Syntax};

pub struct ParserContext {
    pub cm: Lrc<SourceMap>,
}

impl ParserContext {
    pub fn with_parser<F, R>(&self, name: &str, source: &str, f: F) -> R
    where
        F: FnOnce(&mut Parser<Lexer>) -> R,
    {
        let fm = self.cm.new_source_file(
            FileName::Custom(name.to_string()).into(),
            source.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        f(&mut parser)
    }
}
