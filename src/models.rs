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
pub struct WeatherConfig {
    pub location: String,
    pub units: String,
}

#[derive(Debug, Deserialize)]
pub struct SportsTeamsConfig {
    pub nfl: Option<String>,
    pub nba: Option<String>,
    pub mlb: Option<String>,
    pub nhl: Option<String>,
    pub mls: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub weather: WeatherConfig,
    pub sports_teams: SportsTeamsConfig,
    pub feeds: Vec<FeedSource>,
}
