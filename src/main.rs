mod models;
mod scraper;
mod template_context;

use reqwest::Client;
use std::fs;
use std::error::Error;
use chrono::Local;
use askama::Template;
use crate::template_context::NewspaperTemplate;

use gcloud_storage::client::{Client as GCSClient, ClientConfig};
use gcloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("DailyNewspaperAggregator/1.0")
        .build()?;

    // Load configuration
    let config_data = fs::read_to_string("feeds.json")?;
    let config: models::AppConfig = serde_json::from_str(&config_data)?;

    let mut all_articles = Vec::new();

    for feed in config.feeds {
        println!("Fetching {}...", feed.name);
        match scraper::fetch_feed(&client, &feed.url, &feed.name).await {
            Ok(articles) => {
                // Take top 5 from each for variety
                all_articles.extend(articles.into_iter().take(5));
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed.name, e);
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
        
        let upload_type = UploadType::Simple(Media {
            name: "index.html".to_string().into(),
            content_type: "text/html".to_string().into(),
            content_length: Some(html.len() as u64),
        });

        gcs_client.upload_object(&UploadObjectRequest {
            bucket: bucket_name,
            ..Default::default()
        }, html.into_bytes(), &upload_type).await?;
        println!("Upload successful!");
    }

    Ok(())
}
