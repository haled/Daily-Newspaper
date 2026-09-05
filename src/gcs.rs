use gcloud_storage::client::{Client, ClientConfig};
use gcloud_storage::http::objects::upload::{UploadObjectRequest, UploadType, Media};
use gcloud_storage::http::objects::get::GetObjectRequest;
use gcloud_storage::http::objects::download::Range;
use crate::models::History;
use std::error::Error;

pub async fn get_client() -> Result<Client, Box<dyn Error>> {
    let config = ClientConfig::default().with_auth().await?;
    Ok(Client::new(config))
}

pub async fn download_history(client: &Client, bucket: &str, object_path: &str) -> Result<History, Box<dyn Error>> {
    println!("Checking for {} in bucket {}...", object_path, bucket);
    let request = GetObjectRequest {
        bucket: bucket.to_string(),
        object: object_path.to_string(),
        ..Default::default()
    };

    let bytes = client.download_object(&request, &Range::default()).await?;
    let history: History = serde_json::from_slice(&bytes)?;
    println!("  History loaded from GCS.");
    Ok(history)
}

use gcloud_storage::http::object_access_controls::PredefinedObjectAcl;

pub async fn upload_file(
    client: &Client,
    bucket: &str,
    file_path: &str,
    object_name: &str,
    content_type: &str,
    cache_control: Option<&str>,
    make_public: bool,
) -> Result<(), Box<dyn Error>> {
    println!("Uploading {} to {}/{}...", file_path, bucket, object_name);
    let content = std::fs::read(file_path)?;
    
    let request = UploadObjectRequest {
        bucket: bucket.to_string(),
        predefined_acl: if make_public { Some(PredefinedObjectAcl::PublicRead) } else { None },
        ..Default::default()
    };

    if let Some(cc) = cache_control {
        let metadata = gcloud_storage::http::objects::Object {
            name: object_name.to_string(),
            content_type: Some(content_type.to_string()),
            cache_control: Some(cc.to_string()),
            ..Default::default()
        };
        client.upload_object(&request, content, &UploadType::Multipart(Box::new(metadata))).await?;
    } else {
        let media = Media {
            name: object_name.to_string().into(),
            content_type: content_type.to_string().into(),
            content_length: Some(content.len() as u64),
        };
        client.upload_object(&request, content, &UploadType::Simple(media)).await?;
    }

    println!("  Upload successful.");
    Ok(())
}

