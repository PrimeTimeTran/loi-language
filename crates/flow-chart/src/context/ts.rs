// use std::rc::Rc;

// use swc_core::{
//     common::{FileName, SourceFile, SourceMap, sync::Lrc},
//     ecma::{
//         ast::{ClassDecl, EsVersion, FnDecl, Pat, Program, VarDeclarator},
//         parser::{Parser, StringInput, Syntax, lexer::Lexer, parse_file_as_program},
//         visit::Visit,
//     },
// };

// pub struct TsContext {
//     pub cm: Lrc<SourceMap>,
// }

// impl TsContext {
//     pub fn new() -> Self {
//         Self {
//             cm: Lrc::new(SourceMap::default()),
//         }
//     }

//     pub fn file(&self, name: &str, source: &str) -> Lrc<SourceFile> {
//         self.cm.new_source_file(
//             Rc::new(FileName::Custom(name.to_string())),
//             source.to_string(),
//         )
//     }

//     pub fn ts_parser(&self, source: &str) -> (Lrc<SourceFile>, Parser<Lexer>) {
//         let fm = self.file("input.ts", source);
//         let input = StringInput::from(&*fm);

//         let lexer = Lexer::new(
//             Syntax::Typescript(Default::default()),
//             EsVersion::Es2022,
//             input,
//             None,
//         );

//         (fm, Parser::new_from(lexer))
//     }

//     pub fn with_parser<F, R>(&self, source: &str, f: F) -> R
//     where
//         F: FnOnce(&mut Parser<Lexer>) -> R,
//     {
//         let fm = self.file("input.ts", source);
//         let input = StringInput::from(&*fm);

//         let lexer = Lexer::new(
//             Syntax::Typescript(Default::default()),
//             EsVersion::Es2022,
//             input,
//             None,
//         );

//         let mut parser = Parser::new_from(lexer);

//         f(&mut parser)
//     }

//     pub fn parse_ts(
//         &self,
//         name: &str,
//         source: &str,
//     ) -> Result<Program, swc_core::ecma::parser::error::Error> {
//         let fm = self.cm.new_source_file(
//             FileName::Custom(name.to_string()).into(),
//             source.to_string(),
//         );

//         parse_file_as_program(
//             &fm,
//             Syntax::Typescript(Default::default()),
//             EsVersion::Es2022,
//             None,
//             &mut vec![],
//         )
//     }
// }
