use serde::Deserialize;

#[derive(Deserialize)]
pub struct FSConfig {
    pub name: String,
    pub version: String,
    pub entry_points: Vec<String>,
    pub ignore: Vec<String>,
}
