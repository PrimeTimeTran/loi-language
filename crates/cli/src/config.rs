use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub workspace: Option<String>,
}

pub fn load() -> Option<Config> {
    None
}
