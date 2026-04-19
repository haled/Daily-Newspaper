# The Daily Paper (RSS Aggregator)

A Rust-based application that aggregates news from multiple RSS feeds and generates a single, aesthetically pleasing HTML "newspaper" page, optimized for tablets and desktops.

I miss my daily newspaper too! This project recreates that experience by fetching headlines and snippets from your favorite sources and presenting them in a classic multi-column layout with traditional serif typography.

## Features

- **RSS Aggregation:** Concurrently fetches news from major sources (Reuters, CNN, Fox News, BBC, NYT, Hacker News, and more).
- **Sectioned Layout:** Organizes content into **News**, **Finance**, and **Technology** sections.
- **Dynamic Headline Clustering:** Automatically identifies "major" stories covered by multiple sources.
- **Visual Weighting:** Prominent stories are given larger headlines and span 2 columns in the center of the grid for a realistic editorial look.
- **4-Column Layout:** Uses a modern CSS Grid to create a traditional 4-column newspaper feel.
- **Dynamic Header:** Automatically calculates Volume and Issue numbers based on the current date.
- **Classic Typography:** Features 'Playfair Display' for headlines and 'Merriweather' for body text.
- **Offline Ready:** Generates a single, self-contained `index.html` file.

## Prerequisites

- **Rust:** You'll need the [latest Rust toolchain](https://rustup.rs/) installed.
- **Google Cloud SDK (optional):** For deployment to GCP.

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
    This will fetch the latest news, cluster similar headlines, and generate an `index.html` file.

3.  **View the newspaper:**
    Open `index.html` in your web browser.

## Configuration

The list of RSS feeds and their sections is managed in `feeds.json`:

```json
{
  "feeds": [
    {
      "name": "Reuters World",
      "url": "https://news.google.com/rss/search?q=when:24h+source:Reuters&hl=en-US&gl=US&ceid=US:en",
      "section": "News"
    }
  ]
}
```

## Built With

- **[Rust](https://www.rust-lang.org/):** Core application logic.
- **[feed-rs](https://github.com/crepererum/feed-rs):** Robust RSS/Atom parsing.
- **[askama](https://github.com/djc/askama):** Type-safe HTML templating.
- **[reqwest](https://github.com/seanmonstar/reqwest):** Asynchronous HTTP requests.
- **[chrono](https://github.com/chronotope/chrono):** Date and time handling.
- **[Google Fonts](https://fonts.google.com/):** For 'Playfair Display' and 'Merriweather'.

## License

This project is licensed under the MIT License.
