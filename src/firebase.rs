use flate2::{Compression, GzBuilder};
use google_cloud_auth::credentials::Builder;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;

const API_BASE: &str = "https://firebasehosting.googleapis.com/v1beta1";
const OAUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
const TARGET_PATH: &str = "/newspaper/today/index.html";
const MAX_ATTEMPTS: u32 = 5;

#[derive(Deserialize)]
struct ReleasesResponse {
    releases: Vec<Release>,
}

#[derive(Deserialize)]
struct Release {
    version: ReleaseVersion,
}

#[derive(Deserialize)]
struct ReleaseVersion {
    name: String,
}

#[derive(Deserialize)]
struct CloneVersionResponse {
    name: String,
}

#[derive(Serialize)]
struct CloneVersionRequest<'a> {
    #[serde(rename = "sourceVersion")]
    source_version: &'a str,
    exclude: Vec<&'a str>,
    #[serde(rename = "finalize")]
    finalize: bool,
}

#[derive(Serialize)]
struct PopulateFilesRequest<'a> {
    files: HashMap<&'a str, &'a str>,
}

#[derive(Deserialize)]
struct PopulateFilesResponse {
    #[serde(rename = "uploadRequiredHashes")]
    upload_required_hashes: Vec<String>,
    #[serde(rename = "uploadUrl")]
    upload_url: String,
}

#[derive(Serialize)]
struct FinalizeVersionRequest<'a> {
    status: &'a str,
}

/// Publishes `index.html` to Firebase Hosting at `/newspaper/today/index.html`
/// using the single-file "clone the live version" deploy algorithm.
pub async fn publish(html: &[u8], site_id: &str) -> Result<(), Box<dyn Error>> {
    println!("Publishing to Firebase Hosting site '{}' at {}", site_id, TARGET_PATH);

    let token = bearer_token().await?;
    let client = Client::builder()
        .user_agent("DailyNewspaperAggregator/1.0")
        .build()?;

    // 1. Get the current live version
    let live_url = format!("{}/sites/{}/channels/live/releases?pageSize=1", API_BASE, site_id);
    let response = retry_request(|| client.get(&live_url).bearer_auth(&token)).await?;
    let releases: ReleasesResponse = response.json().await?;
    let src_version = releases.releases.first()
        .map(|r| r.version.name.clone())
        .ok_or("No live release found for the site; nothing to clone")?;
    println!("  Live version: {}", src_version);

    // 2. Clone the live version, excluding the file being replaced
    let clone_url = format!("{}/sites/{}/versions:clone", API_BASE, site_id);
    let clone_request = CloneVersionRequest {
        source_version: &src_version,
        exclude: vec![TARGET_PATH],
        finalize: false,
    };
    let response = retry_request(|| {
        client.post(&clone_url)
            .bearer_auth(&token)
            .json(&clone_request)
    }).await?;
    let clone: CloneVersionResponse = response.json().await?;
    let new_version = clone.name;
    println!("  Cloned version: {}", new_version);

    // 3. Deterministically gzip the bytes and hash them
    let gzipped = gzip_deterministic(html)?;
    let hash = sha256_hex(&gzipped);
    println!("  Content hash: {}", hash);

    // 4. Populate files
    let populate_url = format!("{}/sites/{}/versions/{}:populateFiles", API_BASE, site_id, new_version);
    let mut files = HashMap::new();
    files.insert(TARGET_PATH, hash.as_str());
    let populate_request = PopulateFilesRequest { files };
    let response = retry_request(|| {
        client.post(&populate_url)
            .bearer_auth(&token)
            .json(&populate_request)
    }).await?;
    let populate: PopulateFilesResponse = response.json().await?;

    // 5. Upload required bytes
    if populate.upload_required_hashes.is_empty() {
        println!("  Hash already stored; nothing to upload.");
    } else {
        for required_hash in &populate.upload_required_hashes {
            let upload_url = format!("{}/{}", populate.upload_url, required_hash);
            let response = retry_request(|| {
                client.put(&upload_url)
                    .bearer_auth(&token)
                    .header("Content-Type", "application/octet-stream")
                    .body(gzipped.clone())
            }).await?;
            if !response.status().is_success() {
                return Err(format!("Upload of hash {} failed with status {}", required_hash, response.status()).into());
            }
            println!("  Uploaded hash {}", required_hash);
        }
    }

    // 6. Finalize the version
    let finalize_url = format!("{}/sites/{}/versions/{}?update_mask=status", API_BASE, site_id, new_version);
    let finalize_request = FinalizeVersionRequest { status: "FINALIZED" };
    let response = retry_request(|| {
        client.patch(&finalize_url)
            .bearer_auth(&token)
            .json(&finalize_request)
    }).await?;
    if !response.status().is_success() {
        return Err(format!("Finalizing version failed with status {}", response.status()).into());
    }
    println!("  Version finalized.");

    // 7. Release it (go live)
    let version_name = format!("sites/{}/versions/{}", site_id, new_version);
    let release_url = format!("{}/sites/{}/releases?versionName={}", API_BASE, site_id, version_name);
    let response = retry_request(|| {
        client.post(&release_url).bearer_auth(&token)
    }).await?;
    if !response.status().is_success() {
        return Err(format!("Creating release failed with status {}", response.status()).into());
    }

    println!("  Released version {} to live channel.", new_version);
    Ok(())
}

/// Obtains a bearer access token for the cloud-platform scope using
/// Application Default Credentials.
async fn bearer_token() -> Result<String, Box<dyn Error>> {
    let credentials = Builder::default()
        .with_scopes([OAUTH_SCOPE])
        .build_access_token_credentials()?;
    let access_token = credentials.access_token().await?;
    Ok(access_token.token)
}

/// Gzips the input with a fixed mtime of 0 so identical input yields an
/// identical hash across runs.
fn gzip_deterministic(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let builder = GzBuilder::new().mtime(0);
    let mut gz = builder.write(Vec::new(), Compression::default());
    gz.write_all(input)?;
    Ok(gz.finish()?)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Sends a request, retrying transient failures (429, 5xx) with exponential
/// backoff.
async fn retry_request(
    build: impl Fn() -> RequestBuilder,
) -> Result<reqwest::Response, Box<dyn Error>> {
    let mut attempts: u32 = 0;
    loop {
        attempts += 1;
        let response = build().send().await?;
        let status = response.status();
        if !is_transient(status) {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        if attempts >= MAX_ATTEMPTS {
            return Err(format!(
                "Firebase Hosting API request failed after {} attempts ({}): {}",
                attempts, status, body
            ).into());
        }
        let delay_secs = 2u64.pow(attempts - 1);
        eprintln!("  Transient error ({}), retrying in {}s...", status, delay_secs);
        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
    }
}

fn is_transient(status: StatusCode) -> bool {
    status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn gzip_is_deterministic_and_roundtrips() {
        let input = b"hello newspaper world";
        let a = gzip_deterministic(input).unwrap();
        let b = gzip_deterministic(input).unwrap();
        assert_eq!(a, b);

        let mut decoder = flate2::read::GzDecoder::new(&a[..]);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn sha256_hex_matches_known_value() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
