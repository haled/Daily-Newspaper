# GEMINI.md - Daily-Newspaper

## Project Overview
**Daily-Newspaper** is a Rust-based RSS aggregator designed to recreate the experience of reading a traditional daily newspaper. It concurrently fetches headlines and snippets from multiple RSS feeds (e.g., BBC, NYT, Hacker News) and renders them into a single, aesthetically pleasing, self-contained HTML file (`index.html`).

### Key Features
- **RSS Aggregation:** Concurrently fetches news from multiple sources.
- **Traditional Layout:** Uses CSS Grid and Flexbox for a multi-column, classic newspaper feel.
- **Typography:** Features 'Playfair Display' for headlines and 'Merriweather' for body text via Google Fonts.
- **Tablet Optimized:** Specifically designed for 9-inch tablet viewports (1024px to 1280px).
- **GCP Ready:** Integrated with Google Cloud Storage (GCS) and suitable for deployment on Cloud Run with Cloud Scheduler.

### Core Technologies
- **Language:** Rust (Edition 2024)
- **Runtime:** `tokio` (Asynchronous execution)
- **HTTP Client:** `reqwest`
- **RSS Parser:** `feed-rs`
- **Templating:** `askama` (Type-safe HTML templates)
- **Cloud:** `google-cloud-storage` (GCS integration)

---

## Building and Running

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (latest stable version)
- (Optional) [Google Cloud SDK](https://cloud.google.com/sdk) for deployment.

### Key Commands
- **Local Development:**
  ```bash
  cargo run
  ```
  This command fetches the feeds, generates `index.html` in the root directory, and prints progress to the terminal.
- **Production Build:**
  ```bash
  cargo build --release
  ```
- **Docker Build:**
  ```bash
  docker build -t daily-newspaper .
  ```
- **Linting & Formatting:**
  ```bash
  cargo fmt
  cargo clippy
  ```

### Configuration
The list of RSS feeds is currently hardcoded in `src/main.rs`. To modify sources, update the `feeds` vector:
```rust
let feeds = vec![
    ("Source Name", "https://example.com/rss"),
];
```

### Environment Variables
- `GCS_BUCKET`: If set, the application will automatically upload the generated `index.html` to the specified Google Cloud Storage bucket.

---

## Architecture & Conventions

### Directory Structure
- `src/`: Rust source files.
  - `main.rs`: Application entry point, orchestrates fetching and rendering.
  - `models.rs`: Defines core data structures like `Article`.
  - `scraper.rs`: Handles HTTP requests and RSS/Atom parsing logic.
  - `template_context.rs`: Defines the `askama` template structure.
- `templates/`: HTML templates for rendering.
  - `newspaper.html`: The core newspaper layout and embedded CSS.
- `index.html`: The generated output file (generated at runtime).
- `Dockerfile`: Configuration for containerized deployment.

### Development Guidelines
- **Surgical Logic:** Keep the `scraper` focused on normalizing disparate feed formats into the common `Article` model.
- **Template Safety:** Use `askama` to ensure type-safe data binding between Rust and HTML.
- **Performance:** Feed fetching is asynchronous and concurrent.
- **Styling:** CSS is embedded directly within `templates/newspaper.html` to ensure the generated file is self-contained and portable.
- **Content:** The app currently takes the top 5 articles from each source to maintain a balanced layout. Snippets are truncated to approximately 300 characters.

---

## Deployment to GCP
1. **Containerize:** Build and push the image to Google Container Registry (GCR).
2. **Deploy:** Use Google Cloud Run to host the service.
3. **Schedule:** Use Cloud Scheduler with a cron expression (e.g., `30 5 * * *`) to trigger the generation every morning.
4. **Access:** Link the GCS bucket to a domain or use signed URLs for your tablet's browser.
