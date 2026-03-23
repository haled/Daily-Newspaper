mod models;
mod scraper;
mod template_context;

use reqwest::Client;
use std::fs;
use std::error::Error;
use chrono::Local;
use askama::Template;
use crate::template_context::NewspaperTemplate;

use google_cloud_storage::client::{Client as GCSClient, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("DailyNewspaperAggregator/1.0")
        .build()?;

    // Sources
    let feeds = vec![
        ("BBC News", "https://feeds.bbci.co.uk/news/rss.xml"),
        ("NYT Home Page", "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml"),
        ("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index"),
        ("The Guardian", "https://www.theguardian.com/uk/rss"),
        ("Hacker News", "https://news.ycombinator.com/rss"),
    ];

    let mut all_articles = Vec::new();

    for (name, url) in feeds {
        println!("Fetching {}...", name);
        match scraper::fetch_feed(&client, url, name).await {
            Ok(articles) => {
                // Take top 5 from each for variety
                all_articles.extend(articles.into_iter().take(5));
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", name, e);
            }
        }
    }

    let date = Local::now().format("%A, %B %e, %Y").to_string();
    
    let template = NewspaperTemplate {
        articles: all_articles,
        date,
    };

    let html = template.render()?;
    fs::write("index.html", &html)?;
    println!("Local file generated: index.html");

    // Optional GCS Upload
    if let Ok(bucket_name) = std::env::var("GCS_BUCKET") {
        println!("Uploading to GCS bucket: {}...", bucket_name);
        let config = ClientConfig::default().with_auth().await?;
        let gcs_client = GCSClient::new(config);
        
        let upload_type = UploadType::Simple(Media::new("index.html"));
        gcs_client.upload_object(&UploadObjectRequest {
            bucket: bucket_name,
            name: "index.html".to_string(),
            content_type: Some("text/html".to_string()),
            ..Default::default()
        }, html.into_bytes(), &upload_type).await?;
        println!("Upload successful!");
    }

    Ok(())
}
