use crate::models::Article;
use crate::template_context::{WeatherData, DailyForecast};
use feed_rs::parser;
use reqwest::Client;
use std::error::Error;
use html_escape::decode_html_entities;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug)]
struct NwsPointsResponse {
    properties: NwsPointsProperties,
}

#[derive(Deserialize, Debug)]
struct NwsPointsProperties {
    forecast: String,
    #[serde(rename = "relativeLocation")]
    relative_location: NwsRelativeLocation,
}

#[derive(Deserialize, Debug)]
struct NwsRelativeLocation {
    properties: NwsLocationProperties,
}

#[derive(Deserialize, Debug)]
struct NwsLocationProperties {
    city: String,
    state: String,
}

#[derive(Deserialize, Debug)]
struct NwsForecastResponse {
    properties: NwsForecastProperties,
}

#[derive(Deserialize, Debug)]
struct NwsForecastProperties {
    periods: Vec<NwsForecastPeriod>,
}

#[derive(Deserialize, Debug)]
struct NwsForecastPeriod {
    #[serde(rename = "temperature")]
    temperature: i32,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "isDaytime")]
    is_daytime: bool,
}

pub async fn fetch_weather(client: &Client, zip_code: &str, units: &str) -> Result<WeatherData, Box<dyn Error>> {
    // 1. Get Lat/Long from weather.gov redirect
    let zip_url = format!("https://forecast.weather.gov/zipcity.php?inputstring={}", zip_code);
    let res = client.get(zip_url).send().await?;
    let final_url = res.url().to_string();
    
    // Parse lat/lon from URL like ...&lat=38.636&lon=-90.2443
    let lat = extract_param(&final_url, "lat").ok_or("Could not find latitude in redirect URL")?;
    let lon = extract_param(&final_url, "lon").ok_or("Could not find longitude in redirect URL")?;

    // 2. Get Points data
    let points_url = format!("https://api.weather.gov/points/{},{}", lat, lon);
    let points_res: NwsPointsResponse = client.get(points_url)
        .header("User-Agent", "(Daily-Newspaper, contact@example.com)")
        .send().await?
        .json().await?;

    // 3. Get Forecast
    let forecast_res: NwsForecastResponse = client.get(&points_res.properties.forecast)
        .header("User-Agent", "(Daily-Newspaper, contact@example.com)")
        .send().await?
        .json().await?;

    // 4. Process Forecast into Daily High/Low
    // We want today + next 3 days
    let mut daily_map: BTreeMap<String, (Option<i32>, Option<i32>)> = BTreeMap::new();
    let mut day_names: Vec<String> = Vec::new();

    for period in forecast_res.properties.periods {
        let date_str = &period.start_time[0..10]; // YYYY-MM-DD
        if !daily_map.contains_key(date_str) {
            daily_map.insert(date_str.to_string(), (None, None));
            day_names.push(date_str.to_string());
        }
        
        let entry = daily_map.get_mut(date_str).unwrap();
        if period.is_daytime {
            entry.0 = Some(period.temperature);
        } else {
            entry.1 = Some(period.temperature);
        }
    }

    let mut forecast_data = Vec::new();
    let mut today_high = 0;
    let mut today_low = 0;

    for (i, date_str) in day_names.iter().enumerate() {
        if i > 3 { break; }
        
        let (high_opt, low_opt) = daily_map.get(date_str).unwrap();
        let mut high = high_opt.unwrap_or(0);
        let mut low = low_opt.unwrap_or(0);

        // Convert to Celsius if requested
        if units == "C" {
            high = f_to_c(high);
            low = f_to_c(low);
        }

        if i == 0 {
            today_high = high;
            today_low = low;
        } else {
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
            forecast_data.push(DailyForecast {
                day: date.format("%a").to_string(),
                high,
                low,
            });
        }
    }

    Ok(WeatherData {
        location_name: format!("{}, {}", points_res.properties.relative_location.properties.city, points_res.properties.relative_location.properties.state),
        today_high,
        today_low,
        forecast: forecast_data,
        units: units.to_string(),
    })
}

fn extract_param(url: &str, param: &str) -> Option<String> {
    let part = format!("{}=", param);
    if let Some(start) = url.find(&part) {
        let remainder = &url[start + part.len()..];
        let end = remainder.find('&').unwrap_or(remainder.len());
        return Some(remainder[..end].to_string());
    }
    None
}

fn f_to_c(f: i32) -> i32 {
    ((f as f32 - 32.0) * 5.0 / 9.0).round() as i32
}

pub async fn fetch_feed(client: &Client, url: &str, source_name: &str) -> Result<Vec<Article>, Box<dyn Error>> {
    let response = client.get(url).send().await?.bytes().await?;
    let feed = parser::parse(&response[..])?;
    
    let articles = feed.entries.into_iter().map(|entry| {
        let title = entry.title.map(|t| t.content).unwrap_or_else(|| "No Title".to_string());
        let link = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
        
        let raw_snippet = entry.summary
            .map(|s| s.content)
            .or_else(|| entry.content.map(|c| c.body.unwrap_or_default()))
            .unwrap_or_default();
        
        let snippet = strip_html_tags(&raw_snippet);
        let snippet = decode_html_entities(&snippet).to_string();
        
        let snippet = if snippet.chars().count() > 300 {
            let truncated: String = snippet.chars().take(297).collect();
            format!("{}...", truncated)
        } else {
            snippet
        };

        let pub_date = entry.published
            .map(|d| d.format("%B %e, %Y").to_string())
            .unwrap_or_else(|| "Unknown Date".to_string());

        Article {
            title: decode_html_entities(&title).to_string(),
            link,
            snippet,
            pub_date,
            source: source_name.to_string(),
            weight: 1,
            span: 1,
        }
    }).collect();

    Ok(articles)
}

fn strip_html_tags(html: &str) -> String {
    let mut stripped = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            stripped.push(c);
        }
    }
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}
