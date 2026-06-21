use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonNode {
    pub name: String,
    pub r#type: String,
    pub content: Option<String>,
    pub children: Option<Vec<JsonNode>>,
}
