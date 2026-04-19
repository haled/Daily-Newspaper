# GEMINI.md - Daily-Newspaper

## Project Overview
**Daily-Newspaper** is a Rust-based RSS aggregator designed to recreate the experience of reading a traditional daily newspaper. It concurrently fetches headlines and snippets from multiple RSS feeds (e.g., BBC, NYT, Reuters) and renders them into a single, aesthetically pleasing, self-contained HTML file (`index.html`).

### Key Features
- **RSS Aggregation:** Concurrently fetches news from multiple sources.
- **Sectioned Layout:** Organizes news into distinct sections (News, Finance, Technology).
- **Dynamic Headline Clustering:** Groups similar headlines from different sources to identify "major" stories.
- **Visual Weighting:** Prominent stories (covered by multiple sources) are given larger fonts and span 2 columns in the center of the layout.
- **Traditional Layout:** Uses a 4-column CSS Grid for a classic newspaper feel.
- **Dynamic Header:** Automatically calculates Volume (last 2 digits of year) and Issue Number (day of year).
- **Tablet Optimized:** Specifically designed for 9-inch tablet viewports (1024px to 1280px).

### Core Technologies
- **Language:** Rust (Edition 2024)
- **Runtime:** `tokio` (Asynchronous execution)
- **HTTP Client:** `reqwest`
- **RSS Parser:** `feed-rs`
- **Templating:** `askama` (Type-safe HTML templates)

---

## Building and Running

### Key Commands
- **Local Development:**
  ```bash
  cargo run
  ```
  This command fetches the feeds, clusters headlines, generates `index.html`, and prints progress.

---

## Architecture & Conventions

### Directory Structure
- `src/`: Rust source files.
  - `main.rs`: Orchestrates fetching, headline clustering (similarity logic), and interleaving layout logic.
  - `models.rs`: Defines `Article` with `weight` and `span` fields.
  - `scraper.rs`: Normalizes disparate feed formats.
  - `template_context.rs`: Defines the `NewspaperTemplate` and `Section` structures.
- `templates/`: HTML templates for rendering.
  - `newspaper.html`: 4-column responsive grid with dynamic font-sizing based on article weight.
- `feeds.json`: Configuration for RSS sources categorized by section.

### Development Guidelines
- **Headline Similarity:** Uses a word-overlap algorithm (40% match of words > 3 chars) to cluster stories.
- **Interleaving Logic:** To ensure a balanced look, the app interleaves "Minor" and "Major" articles to keep 2-column spans centered in the 4-column grid.
- **Surgical Logic:** Keep the `scraper` focused on normalization; keep `main.rs` focused on editorial logic (clustering/weighting).
- **Styling:** CSS is embedded directly within `templates/newspaper.html`.
