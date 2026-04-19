use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub pub_date: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedSource {
    pub name: String,
    pub url: String,
    pub section: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub feeds: Vec<FeedSource>,
}
