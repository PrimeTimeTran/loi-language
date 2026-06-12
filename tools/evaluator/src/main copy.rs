use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{File, Item};
use walkdir::WalkDir;

#[derive(Default)]
struct ExtractConfig {
    include_structs: bool,
    include_enums: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: extractor <path1> <path2> ...");
        std::process::exit(1);
    }

    let config = ExtractConfig {
        include_structs: true,
        include_enums: true,
    };

    let mut output = String::new();

    for path in &args[1..] {
        let p = PathBuf::from(path);

        if p.is_dir() {
            for entry in WalkDir::new(p)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
            {
                process_file(entry.path(), &config, &mut output);
            }
        } else {
            process_file(&p, &config, &mut output);
        }
    }

    let out_path = Path::new("./extracted_types.rs");
    fs::write(out_path, output).expect("failed to write output");

    println!("Wrote extracted_types.rs");
}

fn process_file(path: &Path, config: &ExtractConfig, output: &mut String) {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return,
    };

    let ast: File = match syn::parse_file(&src) {
        Ok(file) => file,
        Err(_) => return,
    };

    for item in ast.items {
        match item {
            Item::Struct(s) if config.include_structs => {
                output.push_str(&format!("{}\n\n", quote_item(&Item::Struct(s))));
            }
            Item::Enum(e) if config.include_enums => {
                output.push_str(&format!("{}\n\n", quote_item(&Item::Enum(e))));
            }
            _ => {}
        }
    }
}

fn quote_item(item: &Item) -> String {
    use prettyplease::unparse;
    unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    })
}
