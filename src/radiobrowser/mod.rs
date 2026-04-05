use std::fmt;
use serde::Deserialize;

#[derive(Debug)]
pub enum RadioBrowserError {
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for RadioBrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RadioBrowserError::NetworkError(e) => write!(f, "Network error: {}", e),
            RadioBrowserError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for RadioBrowserError {}

#[derive(Debug, Deserialize)]
struct ApiStation {
    name: String,
    url_resolved: String,
    #[serde(default)]
    codec: Option<String>,
    #[serde(default)]
    bitrate: Option<u32>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    clickcount: Option<u32>,
    #[serde(default)]
    votes: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RadioBrowserStation {
    pub name: String,
    pub url: String,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub tags: Option<String>,
    pub country: Option<String>,
    pub clickcount: Option<u32>,
    pub votes: Option<u32>,
}

#[derive(Clone)]
pub struct RadioBrowserFilter {
    pub limit: u32,
    pub order: String,
    pub reverse: bool,
    pub hidebroken: bool,
    pub tag: Option<String>,
    pub country: Option<String>,
    pub name: Option<String>,
}

impl Default for RadioBrowserFilter {
    fn default() -> Self {
        Self {
            limit: 100,
            order: "clickcount".to_string(),
            reverse: true,
            hidebroken: true,
            tag: None,
            country: None,
            name: None,
        }
    }
}

pub async fn fetch_stations(
    filter: &RadioBrowserFilter,
) -> Result<Vec<RadioBrowserStation>, RadioBrowserError> {
    let base = "https://de1.api.radio-browser.info/json/stations/search";

    let client = reqwest::Client::builder()
        .user_agent("RadioCLI/1.4.0 (github.com/schlunsen/radio-cli)")
        .build()
        .map_err(|e| RadioBrowserError::NetworkError(e.to_string()))?;

    let mut params: Vec<(&str, String)> = vec![
        ("limit", filter.limit.to_string()),
        ("order", filter.order.clone()),
        ("reverse", filter.reverse.to_string()),
        ("hidebroken", filter.hidebroken.to_string()),
    ];

    if let Some(ref tag) = filter.tag {
        params.push(("tag", tag.clone()));
    }
    if let Some(ref c) = filter.country {
        params.push(("country", c.clone()));
    }
    if let Some(ref n) = filter.name {
        params.push(("name", n.clone()));
    }

    let resp = client
        .get(base)
        .query(&params)
        .send()
        .await
        .map_err(|e| RadioBrowserError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(RadioBrowserError::NetworkError(format!(
            "HTTP {}",
            resp.status()
        )));
    }

    let api_stations: Vec<ApiStation> = resp
        .json()
        .await
        .map_err(|e| RadioBrowserError::ParseError(e.to_string()))?;

    Ok(api_stations
        .into_iter()
        .map(|s| RadioBrowserStation {
            name: s.name,
            url: s.url_resolved,
            codec: s.codec,
            bitrate: s.bitrate,
            tags: s.tags,
            country: s.country,
            clickcount: s.clickcount,
            votes: s.votes,
        })
        .collect())
}

pub fn build_rb_description(s: &RadioBrowserStation) -> String {
    let mut parts = Vec::new();
    if let Some(ref tags) = s.tags {
        parts.push(format!("Tags: {}", tags));
    }
    if let Some(ref country) = s.country {
        parts.push(format!("Country: {}", country));
    }
    if let Some(ref codec) = s.codec {
        parts.push(format!("Codec: {}", codec));
    }
    if let Some(br) = s.bitrate {
        if br > 0 {
            parts.push(format!("{}kbps", br));
        }
    }
    if let Some(cc) = s.clickcount {
        parts.push(format!("Clicks: {}", cc));
    }
    parts.join(" | ")
}
