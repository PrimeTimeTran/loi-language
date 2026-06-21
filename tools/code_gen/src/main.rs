use std::fs;
use std::path::PathBuf;

use quote::quote;
use std::process::Command;
use syn::{parse_file, File, Item, Variant};

const GROUPS: &[&str] = &[
    "MultiChar_02",
    "SingleChar_03",
    "Structural_04",
    "Meta_05",
    "Identifiers_06",
];

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let input = root.join("src/frontend/token_seeds.rs");
    let output = root.join("src/frontend/token.rs");

    let src = fs::read_to_string(input).unwrap();
    let ast: File = parse_file(&src).unwrap();

    let mut variants: Vec<Variant> = Vec::new();

    for group in GROUPS {
        if let Some(Item::Enum(e)) = find_enum(&ast, group) {
            variants.extend(e.variants.into_iter());
        }
    }

    let mut out = String::from(
        r##"use logos::{Lexer, Logos};
            fn lex_line_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
                let remainder = lex.remainder();
                // Find the end of the line
                let len = remainder.find('\n').unwrap_or(remainder.len());
                lex.bump(len);
                logos::Filter::Skip
            }

            fn lex_block_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
                // We are currently at the start of `>
                // We need to look for <`
                let remainder = lex.remainder();
                if let Some(end) = remainder.find("<`") {
                    lex.bump(end + 2); // Advance past the end sequence
                    logos::Filter::Skip
                } else {
                    // Handle EOF or Unterminated comment error here
                    logos::Filter::Skip
                }
            }

            fn lex_raw_block(lex: &mut Lexer<Token>) -> logos::Filter<()> {
                let mut depth = 1;
                let mut cursor = 0;
                let remainder = lex.remainder();

                while depth > 0 {
                    // Look for the next occurrence of our start or end tokens
                    let next_start = remainder[cursor..].find("@{");
                    let next_end = remainder[cursor..].find("}@");

                    match (next_start, next_end) {
                        (Some(s), Some(e)) if s < e => {
                            depth += 1;
                            cursor += s + 2;
                        }
                        (None, Some(e)) => {
                            depth -= 1;
                            cursor += e + 2;
                        }
                        (Some(_), None) => {
                            depth += 1;
                            cursor += next_start.unwrap() + 2;
                        }
                        _ => {
                            // Error: Unterminated block reached EOF
                            break;
                        }
                    }
                }

                lex.bump(cursor);
                logos::Filter::Skip
            }

            #[derive(Logos, Debug, PartialEq, Clone)]
            #[logos(skip r"[ \t\n\f\r]+")]
            #[logos(skip r"#[^\n]\*")]
            pub enum Token {
        "##,
    );

    for v in &variants {
        out.push_str("    ");
        out.push_str(&quote!(#v).to_string());
        out.push(',');
        out.push('\n');
    }

    out.push_str(
        r##"
            #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
            Ident(String),

            #[regex(r"[0-9]+(?:\.[0-9]+)?", |lex| lex.slice().parse::<f64>().unwrap())]
            Number(f64),

            #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().trim_matches('"').to_string())]
            String(String),

            Error,
            EOF,
        }
        "##,
    );

    fs::write(&output, out).unwrap();

    Command::new("rustfmt")
        .arg(&output)
        .status()
        .expect("failed to run rustfmt");
}

fn find_enum(ast: &File, name: &str) -> Option<Item> {
    ast.items.iter().find_map(|item| {
        if let Item::Enum(e) = item {
            if e.ident == name {
                return Some(item.clone());
            }
        }
        None
    })
}

// use quote::quote;
// use std::{fs, path::PathBuf, process::Command};
// use syn::{Attribute, File, Item, Meta, Variant, parse_file};

// const GROUPS: &[&str] = &[
//     "WhiteSpace_01",
//     "MultiChar_02",
//     "SingleChar_03",
//     "Structural_04",
//     "Meta_05",
//     "Identifiers_06",
// ];

// fn main() {
//     let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//         .parent()
//         .unwrap()
//         .parent()
//         .unwrap()
//         .to_path_buf();

//     let input = root.join("src/frontend/token_seeds.rs");
//     let output = root.join("src/frontend/token.rs");
//     let src = fs::read_to_string(input).unwrap();
//     let ast: File = parse_file(&src).unwrap();

//     let mut token_variants = String::new(); // Changed to mut
//     let mut keyword_arms = Vec::new();
//     let mut out = String::from(
//         r##"use logos::{Lexer, Logos};
//     fn lex_line_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
//         let remainder = lex.remainder();
//         // Find the end of the line
//         let len = remainder.find('\n').unwrap_or(remainder.len());
//         lex.bump(len);
//         logos::Filter::Skip
//     }

//     fn lex_block_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
//         let remainder = lex.remainder();
//         if let Some(end) = remainder.find("<`") {
//             lex.bump(end + 2);
//             logos::Filter::Skip
//         } else {
//             logos::Filter::Skip
//         }
//     }

//     fn lex_raw_block(lex: &mut Lexer<Token>) -> logos::Filter<()> {
//         let mut depth = 1;
//         let mut cursor = 0;
//         let remainder = lex.remainder();

//         while depth > 0 {
//             // Look for the next occurrence of our start or end tokens
//             let next_start = remainder[cursor..].find("@{");
//             let next_end = remainder[cursor..].find("}@");

//             match (next_start, next_end) {
//                 (Some(s), Some(e)) if s < e => {
//                     depth += 1;
//                     cursor += s + 2;
//                 }
//                 (None, Some(e)) => {
//                     depth -= 1;
//                     cursor += e + 2;
//                 }
//                 (Some(_), None) => {
//                     depth += 1;
//                     cursor += next_start.unwrap() + 2;
//                 }
//                 _ => {
//                     // Error: Unterminated block reached EOF
//                     break;
//                 }
//             }
//         }

//         lex.bump(cursor);
//         logos::Filter::Skip
//     }
//     "##,
//     );

//     out.push_str("#[derive(Logos, Debug, PartialEq, Clone)]\npub enum Token {\n");

//     for group in GROUPS {
//         if let Some(Item::Enum(e)) = find_enum(&ast, group) {
//             for v in e.variants {
//                 let variant_name = v.ident.to_string();
//                 let token_str = get_token_str(&v);

//                 if *group == "Identifiers_06" {
//                     keyword_arms.push(format!("\"{}\" => Token::{}", token_str, variant_name));
//                 } else {
//                     // Logic to handle custom callbacks for Meta tokens
//                     if *group == "Meta_05" {
//                         if let Some(lexer_fn) = get_custom_lexer(&variant_name) {
//                             token_variants.push_str(&format!(
//                                 "    #[token({:?}, {})]\n    {},\n",
//                                 token_str, lexer_fn, variant_name
//                             ));
//                         }
//                     } else {
//                         token_variants.push_str(&format!(
//                             "    #[token({:?})]\n    {},\n",
//                             token_str, variant_name
//                         ));
//                     }
//                 }
//             }
//         }
//     }

//     out.push_str(&token_variants);
//     out.push_str("}");

//     fs::write(&output, out).unwrap();

//     Command::new("rustfmt")
//         .arg(&output)
//         .status()
//         .expect("failed to run rustfmt");
// }

// fn get_token_str(v: &Variant) -> String {
//     for attr in &v.attrs {
//         if attr.path().is_ident("token") {
//             let mut result = String::new();
//             let _ = attr.parse_nested_meta(|meta| {
//                 if let Some(lit) = meta.value()?.parse::<syn::LitStr>().ok() {
//                     result = lit.value();
//                 }
//                 Ok(())
//             });
//             if !result.is_empty() {
//                 return result;
//             }
//         }
//     }
//     v.ident.to_string().to_lowercase()
// }

// fn find_enum(ast: &File, name: &str) -> Option<Item> {
//     ast.items
//         .iter()
//         .find(|i| matches!(i, Item::Enum(e) if e.ident == name))
//         .cloned()
// }

// fn get_custom_lexer(variant_name: &str) -> Option<&str> {
//     match variant_name {
//         "LineNote" => Some("lex_line_note"),
//         "BlockNote" => Some("lex_block_note"),
//         "RawStart" | "RawEnd" => Some("lex_raw_block"),
//         _ => None,
//     }
// }
