use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SpecFile {
    #[allow(dead_code)]
    pub version: u32,
    pub areas: Vec<Area>,
}

#[derive(Debug, Deserialize)]
pub struct Area {
    pub name: String,
    pub root: String, // "~/System"
    pub required: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub path: String,
    #[serde(default)]
    pub children: Vec<Node>,
}
