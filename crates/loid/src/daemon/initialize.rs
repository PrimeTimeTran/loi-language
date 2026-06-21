use std::{fs, io::Result, path::PathBuf};

pub fn loid_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".loid")
}

pub fn init() -> Result<()> {
    let root = loid_dir();

    let dirs = [
        root.join("registry"),
        root.join("views"),
        root.join("cache/indexes"),
        root.join("cache/projections"),
        root.join("logs"),
    ];

    for dir in dirs {
        fs::create_dir_all(dir)?;
    }

    let files = [
        (
            root.join("config.toml"),
            r#"# loid configuration

            version = 1
            "#,
        ),
        (
            root.join("state.json"),
            r#"{
                "starts": 0,
                "started_at": 0,
                "longest_run": 0
                }
                "#,
        ),
        (root.join("daemon.pid"), ""),
        (root.join("socket"), ""),
        (root.join("registry/graph.db"), ""),
        (
            root.join("views/default.toml"),
            r#"# Default view

            name = "default"
            "#,
        ),
        (root.join("logs/loid.log"), ""),
    ];

    for (path, contents) in files {
        if !path.exists() {
            fs::write(path, contents)?;
        }
    }

    println!("Initialized loid at {}", root.display());

    Ok(())
}
