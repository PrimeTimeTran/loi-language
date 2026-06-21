use std::path::PathBuf;

pub fn loid_root() -> std::path::PathBuf {
    dirs::home_dir().unwrap().join(".loid")
}

pub fn project_root() -> std::path::PathBuf {
    std::env::current_dir().unwrap()
}

pub fn loid_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".loid")
}
