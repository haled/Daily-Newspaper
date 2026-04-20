use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub pub_date: String,
    pub source: String,
    #[serde(default)]
    pub weight: u32,
    #[serde(default = "default_span")]
    pub span: u32,
}

fn default_span() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct FeedSource {
    pub name: String,
    pub url: String,
    pub section: String,
    pub sort_order: u32,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub feeds: Vec<FeedSource>,
}
