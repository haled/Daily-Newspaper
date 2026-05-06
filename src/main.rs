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

    // Load publish history
    let history_path = "history.json";
    let mut history: models::History = if let Ok(data) = fs::read_to_string(history_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        models::History::default()
    };

    let today_str = Local::now().format("%Y-%m-%d").to_string();

    let weather = match scraper::fetch_weather(&client, &config.weather.location, &config.weather.units).await {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("Error fetching weather: {}", e);
            None
        }
    };

    // Fetch Sports Scores for yesterday
    let yesterday = (Local::now() - chrono::Duration::days(1)).format("%Y%m%d").to_string();
    let mut all_scores = Vec::new();
    
    let leagues_to_check = vec![
        ("nfl", config.sports_teams.nfl.clone()),
        ("nba", config.sports_teams.nba.clone()),
        ("mlb", config.sports_teams.mlb.clone()),
        ("nhl", config.sports_teams.nhl.clone()),
        ("mls", config.sports_teams.mls.clone()),
    ];

    for (league, team_opt) in leagues_to_check {
        if let Some(team) = team_opt {
            if !team.trim().is_empty() {
                match scraper::fetch_scores(&client, league, &team, &yesterday).await {
                    Ok(mut scores) => all_scores.append(&mut scores),
                    Err(e) => eprintln!("Error fetching scores for {} ({}): {}", team, league, e),
                }
            }
        }
    }

    let sports_scores = if !all_scores.is_empty() {
        Some(crate::template_context::SportsScoresData { scores: all_scores })
    } else {
        None
    };

    let mut section_raw_articles: BTreeMap<String, Vec<Article>> = BTreeMap::new();
    let mut section_orders: BTreeMap<String, u32> = BTreeMap::new();

    for feed in config.feeds {
        println!("Fetching {} ({})...", feed.name, feed.section);
        
        // Track the sort order for this section (take the minimum if multiple feeds specify it)
        section_orders.entry(feed.section.clone())
            .and_modify(|e| *e = (*e).min(feed.sort_order))
            .or_insert(feed.sort_order);

        match scraper::fetch_feed(&client, &feed.url, &feed.name).await {
            Ok(articles) => {
                let filtered: Vec<Article> = articles.into_iter()
                    .filter(|a| {
                        !history.published_articles.contains_key(&a.link) || 
                        history.published_articles.get(&a.link) == Some(&today_str)
                    })
                    .take(15)
                    .collect();

                section_raw_articles
                    .entry(feed.section)
                    .or_default()
                    .extend(filtered);
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed.name, e);
            }
        }
    }

    // --- Team-Specific Sports Headlines ---
    let leagues = vec![
        ("NFL", config.sports_teams.nfl, "https://www.espn.com/espn/rss/nfl/news"),
        ("NBA", config.sports_teams.nba, "https://www.espn.com/espn/rss/nba/news"),
        ("MLB", config.sports_teams.mlb, "https://www.espn.com/espn/rss/mlb/news"),
        ("NHL", config.sports_teams.nhl, "https://www.espn.com/espn/rss/nhl/news"),
        ("MLS", config.sports_teams.mls, "https://news.google.com/rss/search?q=when:24h+source:MLSsoccer.com&hl=en-US&gl=US&ceid=US:en"),
    ];

    for (league_name, team_opt, feed_url) in leagues {
        if let Some(team) = team_opt {
            if !team.trim().is_empty() {
                println!("Searching for {} in {} feed...", team, league_name);
                match scraper::fetch_feed(&client, feed_url, league_name).await {
                    Ok(articles) => {
                        let filtered: Vec<Article> = articles.into_iter()
                            .filter(|a| {
                                let team_lower = team.to_lowercase();
                                let matches_team = a.title.to_lowercase().contains(&team_lower) || 
                                                 a.snippet.to_lowercase().contains(&team_lower);
                                let is_new_or_today = !history.published_articles.contains_key(&a.link) || 
                                                    history.published_articles.get(&a.link) == Some(&today_str);
                                matches_team && is_new_or_today
                            })
                            .collect();
                        
                        if !filtered.is_empty() {
                            println!("  Found {} matches for {}.", filtered.len(), team);
                            section_raw_articles
                                .entry("Sports".to_string())
                                .or_default()
                                .extend(filtered);
                            
                            // Ensure "Sports" has a sort order if not already defined
                            section_orders.entry("Sports".to_string()).or_insert(4);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching {} feed for team {}: {}", league_name, team, e);
                    }
                }
            }
        }
    }
    // --------------------------------------

    // Sort sections by their assigned sort_order
    let mut ordered_sections: Vec<String> = section_orders.keys().cloned().collect();
    ordered_sections.sort_by_key(|name| section_orders.get(name).unwrap_or(&u32::MAX));

    let mut sections = Vec::new();
    for section_name in ordered_sections {
        if let Some(mut articles) = section_raw_articles.remove(&section_name) {
            process_articles(&mut articles, section_name == "News - Global");
            sections.push(Section {
                name: section_name,
                articles,
            });
        }
    }

    let now = Local::now();
    let date = now.format("%A, %B %e, %Y").to_string();
    
    // Volume: last 2 digits of year, Number: day of year
    use chrono::Datelike;
    let volume = format!("{:02}", now.year() % 100);
    let issue_number = now.ordinal();

    // Update history with selected articles
    for section in &sections {
        for article in &section.articles {
            history.published_articles.insert(article.link.clone(), today_str.clone());
        }
    }

    let template = NewspaperTemplate { 
        sections, 
        date,
        volume,
        issue_number,
        weather,
        sports_scores,
    };

    let html = template.render()?;
    fs::write("index.html", &html)?;
    println!("Local file generated: index.html");

    // Prune history (keep last 30 days)
    let thirty_days_ago = (Local::now() - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
    history.published_articles.retain(|_, date| *date >= thirty_days_ago);

    let history_data = serde_json::to_string_pretty(&history)?;
    fs::write(history_path, history_data)?;

    Ok(())
}

fn process_articles(articles: &mut Vec<Article>, is_global: bool) {
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

    // 4. Interleave
    let mut interleaved = Vec::new();
    let mut major_iter = major.into_iter();
    let mut minor_iter = minor.into_iter();

    // For the Global section, we want a Major article (if any) to appear first
    // so it sits between the weather and sports side-boxes in the top row.
    if is_global {
        if let Some(m_art) = major_iter.next() {
            interleaved.push(m_art);
        }
    }

    // Standard pattern: [Minor] [Major] [Minor] to keep Major centered in 4 columns
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
