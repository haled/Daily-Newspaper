mod models;
mod scraper;
mod template_context;

use reqwest::Client;
use std::fs;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use chrono::Local;
use askama::Template;
use crate::template_context::{NewspaperTemplate, Section};
use crate::models::Article;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("DailyNewspaperAggregator/1.0")
        .build()?;

    // Load configuration
    let config_data = fs::read_to_string("feeds.json")?;
    let config: models::AppConfig = serde_json::from_str(&config_data)?;

    let mut section_raw_articles: BTreeMap<String, Vec<Article>> = BTreeMap::new();

    for feed in config.feeds {
        println!("Fetching {} ({})...", feed.name, feed.section);
        match scraper::fetch_feed(&client, &feed.url, &feed.name).await {
            Ok(articles) => {
                section_raw_articles
                    .entry(feed.section)
                    .or_default()
                    .extend(articles.into_iter().take(15)); // Take more for variety
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed.name, e);
            }
        }
    }

    let mut sections = Vec::new();
    let ordered_sections = vec!["News", "Finance", "Technology"];
    
    for section_name in ordered_sections {
        if let Some(mut articles) = section_raw_articles.remove(section_name) {
            process_articles(&mut articles);
            sections.push(Section {
                name: section_name.to_string(),
                articles,
            });
        }
    }

    for (name, mut articles) in section_raw_articles {
        process_articles(&mut articles);
        sections.push(Section { name, articles });
    }

    let now = Local::now();
    let date = now.format("%A, %B %e, %Y").to_string();
    
    // Volume: last 2 digits of year, Number: day of year
    use chrono::Datelike;
    let volume = format!("{:02}", now.year() % 100);
    let issue_number = now.ordinal();

    let template = NewspaperTemplate { 
        sections, 
        date,
        volume,
        issue_number,
    };

    let html = template.render()?;
    fs::write("index.html", &html)?;
    println!("Local file generated: index.html");

    Ok(())
}

fn process_articles(articles: &mut Vec<Article>) {
    if articles.is_empty() { return; }

    // 1. Calculate weights based on similarity
    let mut sources_for_cluster = vec![HashSet::new(); articles.len()];

    for i in 0..articles.len() {
        sources_for_cluster[i].insert(articles[i].source.clone());
        for j in (i + 1)..articles.len() {
            if is_similar(&articles[i].title, &articles[j].title) {
                sources_for_cluster[i].insert(articles[j].source.clone());
                sources_for_cluster[j].insert(articles[i].source.clone());
            }
        }
    }

    for i in 0..articles.len() {
        articles[i].weight = sources_for_cluster[i].len() as u32;
    }

    // 2. Deduplicate: if similar, keep only one (the one from more sources or first one)
    let mut to_remove = HashSet::new();
    for i in 0..articles.len() {
        if to_remove.contains(&i) { continue; }
        for j in (i + 1)..articles.len() {
            if is_similar(&articles[i].title, &articles[j].title) {
                to_remove.insert(j);
            }
        }
    }

    let mut i = 0;
    let mut processed = Vec::new();
    for art in articles.drain(..) {
        if !to_remove.contains(&i) {
            processed.push(art);
        }
        i += 1;
    }
    *articles = processed;

    // 3. Separate major and minor, sort by weight
    articles.sort_by(|a, b| b.weight.cmp(&a.weight));
    
    let mut major: Vec<Article> = Vec::new();
    let mut minor: Vec<Article> = Vec::new();
    
    for mut art in articles.drain(..) {
        if art.weight >= 2 {
            art.span = 2;
            major.push(art);
        } else {
            art.span = 1;
            minor.push(art);
        }
    }

    // 4. Interleave to create [Minor] [Major] [Minor] pattern for 4 columns
    let mut interleaved = Vec::new();
    let mut major_iter = major.into_iter();
    let mut minor_iter = minor.into_iter();

    while let Some(m_art) = major_iter.next() {
        // Try to get two minor articles for this major one
        if let Some(min1) = minor_iter.next() {
            interleaved.push(min1);
        }
        interleaved.push(m_art);
        if let Some(min2) = minor_iter.next() {
            interleaved.push(min2);
        }
    }
    
    // Add remaining minor articles
    interleaved.extend(minor_iter);

    *articles = interleaved;
    articles.truncate(16); // 4 rows of 4
}

fn is_similar(a: &str, b: &str) -> bool {
    let a_words: HashSet<String> = a.to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|s| s.to_string())
        .collect();
    let b_words: HashSet<String> = b.to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|s| s.to_string())
        .collect();

    if a_words.is_empty() || b_words.is_empty() {
        return false;
    }

    let intersection = a_words.intersection(&b_words).count();
    let smaller = a_words.len().min(b_words.len());
    
    // If 40% of words in the shorter title match, consider similar
    (intersection as f32 / smaller as f32) > 0.4
}
