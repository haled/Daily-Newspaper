mod models;
mod scraper;
mod template_context;

use reqwest::Client;
use std::fs;
use std::collections::BTreeMap;
use std::error::Error;
use chrono::Local;
use askama::Template;
use crate::template_context::{NewspaperTemplate, Section};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("DailyNewspaperAggregator/1.0")
        .build()?;

    // Load configuration
    let config_data = fs::read_to_string("feeds.json")?;
    let config: models::AppConfig = serde_json::from_str(&config_data)?;

    let mut sectioned_articles: BTreeMap<String, Vec<models::Article>> = BTreeMap::new();

    for feed in config.feeds {
        println!("Fetching {} ({})...", feed.name, feed.section);
        match scraper::fetch_feed(&client, &feed.url, &feed.name).await {
            Ok(articles) => {
                // Take top 5 from each for variety
                sectioned_articles
                    .entry(feed.section)
                    .or_default()
                    .extend(articles.into_iter().take(5));
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed.name, e);
            }
        }
    }

    // Convert BTreeMap to a Vec<Section> for the template
    // We want a specific order: News, Finance, Technology
    let ordered_sections = vec!["News", "Finance", "Technology"];
    let mut sections = Vec::new();
    
    for section_name in ordered_sections {
        if let Some(articles) = sectioned_articles.remove(section_name) {
            sections.push(Section {
                name: section_name.to_string(),
                articles,
            });
        }
    }

    // Any remaining sections
    for (name, articles) in sectioned_articles {
        sections.push(Section { name, articles });
    }

    let date = Local::now().format("%A, %B %e, %Y").to_string();
    
    let template = NewspaperTemplate {
        sections,
        date,
    };

    let html = template.render()?;
    fs::write("index.html", &html)?;
    println!("Local file generated: index.html");

    Ok(())
}
