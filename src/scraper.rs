use crate::models::Article;
use feed_rs::parser;
use reqwest::Client;
use std::error::Error;
use html_escape::decode_html_entities;

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
        
        // Strip HTML tags (simple approach for snippet)
        let snippet = strip_html_tags(&raw_snippet);
        let snippet = decode_html_entities(&snippet).to_string();
        
        // Truncate snippet
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
