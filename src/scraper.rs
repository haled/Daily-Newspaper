use crate::models::Article;
use crate::template_context::{WeatherData, DailyForecast, SportsScore};
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

#[derive(Deserialize, Debug)]
struct EspnScoreboardResponse {
    events: Vec<EspnEvent>,
}

#[derive(Deserialize, Debug)]
struct EspnEvent {
    status: EspnStatus,
    competitions: Vec<EspnCompetition>,
}

#[derive(Deserialize, Debug)]
struct EspnStatus {
    #[serde(rename = "type")]
    status_type: EspnStatusType,
}

#[derive(Deserialize, Debug)]
struct EspnStatusType {
    detail: String,
}

#[derive(Deserialize, Debug)]
struct EspnCompetition {
    competitors: Vec<EspnCompetitor>,
}

#[derive(Deserialize, Debug)]
struct EspnCompetitor {
    score: String,
    team: EspnTeam,
}

#[derive(Deserialize, Debug)]
struct EspnTeam {
    #[serde(rename = "displayName")]
    display_name: String,
    abbreviation: String,
}

pub async fn fetch_scores(client: &Client, league: &str, team_name: &str, date_str: &str) -> Result<Vec<SportsScore>, Box<dyn Error>> {
    let league_path = match league {
        "nfl" => "football/nfl",
        "nba" => "basketball/nba",
        "mlb" => "baseball/mlb",
        "nhl" => "hockey/nhl",
        "mls" => "soccer/usa.1",
        _ => return Ok(vec![]),
    };

    let url = format!("https://site.api.espn.com/apis/site/v2/sports/{}/scoreboard?dates={}", league_path, date_str);
    let res: EspnScoreboardResponse = client.get(url).send().await?.json().await?;

    let mut scores = Vec::new();
    let team_lower = team_name.to_lowercase();

    for event in res.events {
        let competition = &event.competitions[0];
        let mut target_competitor = None;
        let mut opponent_competitor = None;

        for competitor in &competition.competitors {
            if competitor.team.display_name.to_lowercase().contains(&team_lower) || 
               competitor.team.abbreviation.to_lowercase().contains(&team_lower) {
                target_competitor = Some(competitor);
            } else {
                opponent_competitor = Some(competitor);
            }
        }

        if let (Some(target), Some(opponent)) = (target_competitor, opponent_competitor) {
            scores.push(SportsScore {
                team_name: target.team.display_name.clone(),
                opponent_name: opponent.team.display_name.clone(),
                team_score: target.score.clone(),
                opponent_score: opponent.score.clone(),
                status: event.status.status_type.detail.clone(),
            });
        }
    }

    Ok(scores)
}

pub async fn fetch_weather(client: &Client, zip_code: &str, units: &str) -> Result<WeatherData, Box<dyn Error>> {
    println!("Fetching weather for zip code: {}...", zip_code);
    // 1. Get Lat/Long from weather.gov redirect
    let zip_url = format!("https://forecast.weather.gov/zipcity.php?inputstring={}", zip_code);
    let res = client.get(zip_url).send().await?;
    let final_url = res.url().to_string();
    
    // Parse lat/lon from URL like ...&lat=38.636&lon=-90.2443
    let lat = extract_param(&final_url, "lat").ok_or("Could not find latitude in redirect URL")?;
    let lon = extract_param(&final_url, "lon").ok_or("Could not find longitude in redirect URL")?;
    println!("  Geocoded to: {}, {}", lat, lon);

    // 2. Get Points data
    let points_url = format!("https://api.weather.gov/points/{},{}", lat, lon);
    let points_res: NwsPointsResponse = client.get(points_url)
        .header("User-Agent", "(Daily-Newspaper, contact@example.com)")
        .send().await?
        .json().await?;

    // 3. Get Forecast
    println!("  Fetching forecast from: {}...", points_res.properties.forecast);
    let forecast_res: NwsForecastResponse = client.get(&points_res.properties.forecast)
        .header("User-Agent", "(Daily-Newspaper, contact@example.com)")
        .send().await?
        .json().await?;

    // 4. Process Forecast into Daily High/Low
    println!("  Processing {} periods...", forecast_res.properties.periods.len());
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
            // Keep the maximum if there are multiple daytime periods (rare but possible)
            entry.0 = Some(entry.0.map_or(period.temperature, |current| current.max(period.temperature)));
        } else {
            // Keep the minimum for nighttime
            entry.1 = Some(entry.1.map_or(period.temperature, |current| current.min(period.temperature)));
        }
    }

    let mut forecast_data = Vec::new();
    let mut today_high = 0;
    let mut today_low = 0;

    for (i, date_str) in day_names.iter().enumerate() {
        if i > 3 { break; }
        
        let (high_opt, low_opt) = daily_map.get(date_str).unwrap();
        
        // If high is missing, use low (and vice versa) so we don't show 0
        let mut high = high_opt.or(*low_opt).unwrap_or(0);
        let mut low = low_opt.or(*high_opt).unwrap_or(0);

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
