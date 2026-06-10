use std::fs;
use std::path::PathBuf;

use quote::quote;
use std::process::Command;
use syn::{File, Item, Variant, parse_file};

const GROUPS: &[&str] = &[
    "Comments",
    "KeywordScope",
    "Keyword",
    "Declarations",
    "LogicWord",
    "Scope",
    "Logic",
    "Arithmetic",
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
        r##"use logos::Logos;
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
            #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
            Number(f64),

            #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
            String(String),

            #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
            Ident(String),

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
