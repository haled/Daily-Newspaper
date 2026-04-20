use askama::Template;
use crate::models::Article;
use serde::Serialize;

#[derive(Serialize)]
pub struct DailyForecast {
    pub day: String,
    pub high: i32,
    pub low: i32,
}

#[derive(Serialize)]
pub struct WeatherData {
    pub location_name: String,
    pub today_high: i32,
    pub today_low: i32,
    pub forecast: Vec<DailyForecast>,
    pub units: String,
}

pub struct Section {
    pub name: String,
    pub articles: Vec<Article>,
}

#[derive(Template)]
#[template(path = "newspaper.html")]
pub struct NewspaperTemplate {
    pub sections: Vec<Section>,
    pub date: String,
    pub volume: String,
    pub issue_number: u32,
    pub weather: Option<WeatherData>,
}
