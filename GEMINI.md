# GEMINI.md - Daily-Newspaper

## Project Overview
**Daily-Newspaper** is a Rust-based RSS aggregator designed to recreate the experience of reading a traditional daily newspaper. It concurrently fetches headlines and snippets from multiple RSS feeds and renders them into a single, aesthetically pleasing, self-contained HTML file (`index.html`).

### Key Features
- **RSS Aggregation:** Concurrently fetches news from multiple sources.
- **Weather Box:** Integrated weather forecast using `weather.gov` geocoding and API.
- **Sports Scores:** Live score tracking for specified teams via ESPN API.
- **Team-Specific Filtering:** Keyword-based filtering of major sports league feeds.
- **Headline Persistence:** Uses `history.json` to prevent displaying the same article on different days.
- **Dynamic Section Layout:** 4-column CSS Grid with configurable section ordering.
- **Visual Weighting:** Major stories (covered by multiple sources) are given larger fonts and span 2 columns.
- **Dynamic Header:** Automatically calculates Volume (last 2 digits of year) and Issue Number (day of year).

### Core Technologies
- **Language:** Rust (Edition 2024)
- **Runtime:** `tokio` (Asynchronous execution)
- **HTTP Client:** `reqwest`
- **RSS Parser:** `feed-rs`
- **Templating:** `askama` (Type-safe HTML templates)
- **Persistence:** `serde_json` for `history.json` and `feeds.json`.

---

## Building and Running

### Key Commands
- **Local Development:**
  ```bash
  cargo run
  ```
  This command fetches the feeds, updates weather/scores, clusters headlines, and generates `index.html`.

---

## Architecture & Conventions

### Directory Structure
- `src/`: Rust source files.
  - `main.rs`: Orchestrates fetching, headline clustering, persistence logic, and layout processing.
  - `models.rs`: Defines config, article, and history structures.
  - `scraper.rs`: Handles RSS normalization, `weather.gov` geocoding/forecasts, and ESPN score fetching.
  - `template_context.rs`: Defines the data structures passed to the HTML template.
- `templates/`: HTML templates for rendering.
- `feeds.json`: Main configuration for weather, teams, and RSS sources.
- `history.json`: (Local only) Tracks published article links to avoid repeats.

### Development Guidelines
- **Headline Similarity:** Uses a word-overlap algorithm (40% match of words > 3 chars) to cluster stories.
- **Publish History:** Articles are only shown once per 30-day window (unless published on the same day as multiple runs).
- **Layout Logic:** The Global section prioritizes a 2-column "Major" article to fill the space between the weather and sports side-boxes.
- **Surgical Logic:** Keep the `scraper` focused on API normalization; keep `main.rs` focused on editorial logic and persistence.
- **Styling:** CSS and layout logic are embedded within `templates/newspaper.html`. Links are configured to open in new tabs.
