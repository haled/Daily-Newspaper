# Daily Newspaper RSS Aggregator (Rust)

A Rust-based application that aggregates news from multiple RSS feeds and generates a single, aesthetically pleasing HTML "newspaper" page, optimized for a 9-inch tablet. 

I miss my daily newspaper too! This project recreates that experience by fetching headlines and snippets from your favorite sources and presenting them in a classic multi-column layout with traditional serif typography.

## Features

- **RSS Aggregation:** Concurrently fetches news from multiple sources (BBC, NYT, Ars Technica, The Guardian, and Hacker News).
- **Newspaper Layout:** Uses CSS Grid and Flexbox to create a traditional multi-column layout.
- **Classic Typography:** Features 'Playfair Display' for headlines and 'Merriweather' for body text for a print-like feel.
- **Article Snippets:** Displays a truncated summary for each headline to give you the "lead" of the story.
- **Tablet Optimized:** Specifically designed for a 9-inch tablet viewport (1024px to 1280px).
- **Offline Ready:** Generates a single, self-contained `index.html` file that can be saved for offline viewing.
- **GCP Integrated:** Ready for deployment to Google Cloud Run with automatic uploads to Google Cloud Storage (GCS).

## Prerequisites

- **Rust:** You'll need the [Rust toolchain](https://rustup.rs/) installed.
- **Google Cloud SDK (optional):** Required for deployment to GCP.
- **A GCS Bucket (optional):** Required if you want to upload the generated newspaper to the cloud.

## Getting Started Locally

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/Daily-Newspaper.git
    cd Daily-Newspaper
    ```

2.  **Run the aggregator:**
    ```bash
    cargo run
    ```
    This will fetch the latest news and generate an `index.html` file in the project root.

3.  **View the newspaper:**
    Open `index.html` in your web browser.

## Deployment to Google Cloud Platform

To have your newspaper ready every morning at 5:30 AM:

### 1. Create a GCS Bucket
Create a bucket in your GCP project to host the generated newspaper.

### 2. Build and Push the Container
Use Google Cloud Build to containerize the application:
```bash
gcloud builds submit --tag gcr.io/[PROJECT_ID]/daily-newspaper
```

### 3. Deploy to Cloud Run
Deploy the image to Cloud Run as a job or service. Ensure you set the `GCS_BUCKET` environment variable:
```bash
gcloud run deploy daily-newspaper \
  --image gcr.io/[PROJECT_ID]/daily-newspaper \
  --set-env-vars GCS_BUCKET=[YOUR_BUCKET_NAME] \
  --platform managed \
  --region [YOUR_REGION] \
  --no-allow-unauthenticated
```
*Note: Ensure the Cloud Run service account has `roles/storage.objectCreator` permissions for your bucket.*

### 4. Schedule the Execution
Use Cloud Scheduler to trigger your Cloud Run service every day at 5:30 AM:
- **Frequency:** `30 5 * * *`
- **Target:** HTTP (pointing to your Cloud Run URL)

## Configuration

The list of RSS feeds is currently hardcoded in `src/main.rs`. You can easily modify the `feeds` vector to add or remove sources:

```rust
let feeds = vec![
    ("BBC News", "https://feeds.bbci.co.uk/news/rss.xml"),
    ("My Local News", "https://example.com/rss"),
    // ...
];
```

## Built With

- **[Rust](https://www.rust-lang.org/):** The core application logic.
- **[feed-rs](https://github.com/crepererum/feed-rs):** Robust RSS/Atom parsing.
- **[askama](https://github.com/djc/askama):** Type-safe HTML templating.
- **[reqwest](https://github.com/seanmonstar/reqwest):** Asynchronous HTTP requests.
- **[google-cloud-storage](https://github.com/yoshuawuyts/google-cloud-storage-rs):** GCS integration.
- **[Google Fonts](https://fonts.google.com/):** For traditional newspaper typography.

## License

This project is licensed under the MIT License.
