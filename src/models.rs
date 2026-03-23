use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub pub_date: String,
    pub source: String,
}
