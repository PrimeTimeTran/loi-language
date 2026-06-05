use std::env;



pub struct Config {
    pub input: String,
    pub output: String,
    pub watch: bool,
    pub debug: bool,
}

impl Config {
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();

        let input = args
            .get(1)
            .cloned()
            .unwrap_or("targets/examples".to_string());

        let output = args.get(2).cloned().unwrap_or("tmp/output".to_string());

        let watch = args.contains(&"--watch".to_string());
        let debug = args.contains(&"--debug".to_string());

        Self {
            input,
            output,
            watch,
            debug,
        }
    }
}

pub fn from_args() -> Config {
    let args: Vec<String> = std::env::args().collect();

    let mut watch = false;
    let mut input = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--watch" => watch = true,
            other => input = Some(other.to_string()),
        }
    }

    Config {
        input: input.unwrap_or("targets/examples".to_string()),
        output: "tmp/output".to_string(),
        watch,
        debug: false,
    }
}

pub fn run() {
    let config = Config::from_args();

    if config.watch {
        return crate::watcher::watch(config).unwrap();
    }

    match crate::pipeline::compile_targets(&config) {
        Ok(_) => println!("🎉 All files compiled successfully"),
        Err(e) => {
            eprintln!("💥 Compilation failed:\n{}", e);
            std::process::exit(1);
        }
    }
}
