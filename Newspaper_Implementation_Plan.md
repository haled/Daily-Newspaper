# Design Document: Daily Newspaper RSS Aggregator (Rust)

## 1. Overview
A Rust-based application that aggregates news from multiple RSS feeds and generates a single, aesthetically pleasing HTML "newspaper" page, optimized for a 9-inch tablet.

## 2. Core Components

### A. RSS Aggregator (Rust Backend)
- **Crates to Use:**
  - `feed-rs`: Robust parsing for RSS 0.9, 1.0, 2.0, and Atom.
  - `reqwest`: For making HTTP requests to fetch feeds.
  - `tokio`: For asynchronous feed fetching.
  - `serde`: For data serialization.
  - `askama` or `tera`: Type-safe template engines.
  - `html-escape`: To sanitize snippets from the feed.

- **Data Flow:**
  1. Read a configuration file (YAML/JSON) containing a list of RSS URLs and categories.
  2. Concurrently fetch all feeds.
  3. Normalize the feed items into a internal `Article` struct:
     - **Title:** Headline.
     - **Link:** URL to the full story.
     - **Snippet:** A truncated version (e.g., first 200 characters) of the description or summary.
     - **PubDate:** Formatted date.
     - **Source:** Name of the publication.
  4. Pass the aggregated data to the template engine.

### B. Newspaper Layout (HTML/CSS)
- **Traditional Typography:**
  - **Headlines:** Use serif fonts like **'Playfair Display'**, **'Merriweather'**, or **'Lora'**. High contrast, bold weights for main headlines.
  - **Body Text:** Use legible serif fonts like **'Georgia'** or **'Source Serif Pro'**. Tight line spacing and justification to mimic newsprint.
  - **Google Fonts:** Integration to ensure consistent appearance on tablets.
- **Responsive Grid:** Use CSS Grid to create a multi-column newspaper layout.
- **Article Snippets:**
  - Display the first paragraph or a truncated summary below each headline.
  - Use a smaller font size for snippets compared to headlines but maintain high readability.
- **Tablet Optimization (9-inch):**
  - Target a viewport width around 1024px to 1280px.
  - Use 3-4 columns for a classic look.
  - Ensure touch-friendly tap targets for article links.

## 3. GCP Integration & Automation (The "Breakfast" Requirement)

To ensure the news is ready for your breakfast on a 9-inch tablet:

### A. Execution Environment
- **Google Cloud Run Jobs:** The Rust application is packaged into a container and runs
  as the Cloud Run Job `daily-newspaper-job` (region `us-central1`).
- **Cloud Scheduler:** The cron job `daily-newspaper-schedule` triggers the job daily at
  **5:00 AM** (America/Chicago). This ensures the generation process is completed well
  before breakfast.
- **Continuous delivery:** Pushing to `main` re-deploys the job via GitHub Actions
  (`.github/workflows/deploy.yml`), which also fires one immediate execution.

### B. Self-Contained HTML Delivery
- **Inline Assets:** The Rust app generates a single, self-contained HTML file with all
  CSS inlined. This makes it easy to download and read offline.
- **Primary host — Firebase Hosting:** The generated file is published to
  `/newspaper/today/index.html` via the Hosting REST API (clone the live version,
  upload the new file, finalize, release) — see `NEWSPAPER_TODAY_PUBLISHER.md`. It is
  served at `https://valiant-azimuth-296116.web.app/newspaper/today/` (and
  `https://www.darrenehale.com/newspaper/today/`), which you can bookmark on your
  tablet.
- **Legacy GCS upload:** Uploads to the GCS bucket are retained but best-effort; the
  original bucket no longer exists, so failures are logged and the run continues.
- **Offline Viewing:** Since the HTML is self-contained, you can "Save for Offline" in
  your tablet's browser or the app can be configured as a basic PWA to cache the latest
  edition automatically.

## 4. Implementation Plan

### Phase 1: Basic Scraper & HTML Generation
- Initialize a new Rust project (`cargo init`).
- Implement RSS fetching and HTML template generation using `askama`.
- Ensure the template inlines all CSS and fonts (via Google Fonts or locally).

### Phase 2: GCP Integration
- Create a `Dockerfile` for the Rust application.
- Implement Firebase Hosting publishing (`src/firebase.rs`) using the clone-based
  single-file deploy (steps documented in `NEWSPAPER_TODAY_PUBLISHER.md`).
- Implement the legacy GCS upload logic (`src/gcs.rs`, best-effort).
- Set up the Cloud Run Job and Cloud Scheduler trigger (5:00 AM).

### Phase 3: Layout & Typography Refinement
- Finalize the newspaper-style CSS with a multi-column grid and traditional serif fonts.
- Optimize the layout for the 1024px-1280px viewport of a 9-inch tablet.

### Phase 4: Output Generation
- The application will output a single `index.html` file (and potentially a `styles.css`).
- This file can then be hosted locally or served via a simple web server for the tablet to access.

## 5. Example Layout Strategy
- **Header:** Large title (e.g., "The Daily Digital") with date and weather (optional).
- **Main Column:** 2/3 width for top stories.
- **Sidebar:** 1/3 width for "Briefs" or "Local News".
- **Footer:** Links to sources and generation time.
