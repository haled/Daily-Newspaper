# The Daily Paper (RSS Aggregator)

A Rust-based application that aggregates news from multiple RSS feeds and generates a single, aesthetically pleasing HTML "newspaper" page, optimized for tablets and desktops.

I miss my daily newspaper too! This project recreates that experience by fetching headlines and snippets from your favorite sources and presenting them in a classic multi-column layout with traditional serif typography.

## Features

- **RSS Aggregation:** Concurrently fetches news from major sources (Reuters, CNN, Fox News, BBC, NYT, Hacker News, and more).
- **Newspaper-Style Weather:** A dedicated weather box at the top of the Global section fetching forecasts from `weather.gov`.
- **Last Night's Scores:** Real-time sports scores for your favorite teams fetched from the ESPN API.
- **Sports Team Tracking:** Automatically filters league-wide feeds (NFL, NBA, MLB, NHL, MLS) for headlines about your specific teams.
- **Dynamic Section Ordering:** Section sequence is fully configurable via `sort_order` properties in the settings.
- **Headline Persistence:** Prevents duplicate headlines across different days using a local `history.json` tracking system.
- **Visual Weighting:** Automatically identifies "major" stories and gives them larger headlines spanning 2 columns.
- **4-Column Layout:** Uses a modern CSS Grid to create a traditional 4-column newspaper feel.
- **Dynamic Header:** Automatically calculates Volume and Issue numbers based on the current date.
- **Classic Typography:** Features 'Playfair Display' for headlines and 'Merriweather' for body text.

## Prerequisites

- **Rust:** You'll need the [latest Rust toolchain](https://rustup.rs/) installed.

## Getting Started Locally

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/haled/Daily-Newspaper.git
    cd Daily-Newspaper
    ```

2.  **Run the aggregator:**
    ```bash
    cargo run
    ```
    This will fetch the latest news, update weather and scores, and generate an `index.html` file.

3.  **View the newspaper:**
    Open `index.html` in your web browser.

## Configuration

All settings are managed in `feeds.json`. You can configure your location for weather, your favorite sports teams, and the order of your sections:

```json
{
  "weather": {
    "location": "63101",
    "units": "F"
  },
  "sports_teams": {
    "nfl": "Chiefs",
    "mlb": "Cardinals",
    "nhl": "Blues",
    "mls": "St. Louis CITY SC"
  },
  "feeds": [
    {
      "name": "Reuters World",
      "url": "...",
      "section": "News - Global",
      "sort_order": 1
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

## License

This project is licensed under the MIT License.
