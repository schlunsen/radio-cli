use std::fmt;

// Define a custom error type that is Send + Sync
#[derive(Debug)]
#[allow(dead_code)]
pub enum RcastError {
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for RcastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RcastError::NetworkError(e) => write!(f, "Network error: {}", e),
            RcastError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for RcastError {}

pub struct RcastStation {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub bitrate: Option<String>,
    pub genre: Option<String>,
    pub listeners: Option<u32>,
}

// Function to fetch stations from rcast.net
pub async fn fetch_stations() -> Result<Vec<RcastStation>, RcastError> {
    // URL for the Icecast stations from rcast.net
    let url = "https://www.rcast.net/dir?action=search&search=icecast&sortby=1";

    // Use reqwest to send the HTTP request
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .map_err(|e| RcastError::NetworkError(format!("Failed to build client: {}", e)))?;

    // Send request
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| RcastError::NetworkError(e.to_string()))?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(RcastError::NetworkError(format!(
            "Failed to fetch stations: HTTP {}",
            response.status()
        )));
    }

    // Get the HTML content
    let html = response
        .text()
        .await
        .map_err(|e| RcastError::NetworkError(e.to_string()))?;

    // Parse the HTML to extract stations
    let stations = parse_stations_from_html(&html)?;

    Ok(stations)
}

// Function to parse the HTML and extract station information
fn parse_stations_from_html(html: &str) -> Result<Vec<RcastStation>, RcastError> {
    let mut stations = Vec::new();
    let mut station_ids = std::collections::HashSet::new();

    // Look for station stream declarations in the JavaScript
    // Format: var stream207313 = { mp3: "https://stream.rcast.net/207313" }
    let var_pattern = "var stream";
    let mut search_pos = 0;

    while let Some(pos) = html[search_pos..].find(var_pattern) {
        let var_pos = search_pos + pos;
        let after_var = var_pos + var_pattern.len();

        // Extract the station ID that follows "var stream"
        if let Some(space_pos) = html[after_var..].find(" ") {
            let station_id = &html[after_var..after_var + space_pos];

            // Only accept numeric IDs
            if station_id.chars().all(|c| c.is_ascii_digit()) {
                // Add if not already seen
                if !station_ids.contains(station_id) {
                    station_ids.insert(station_id.to_string());

                    // Look for station name (station IDs are often referenced by currentsong_ID)
                    let id_marker = format!("id=\"currentsong_{}", station_id);
                    let mut name = format!("Icecast Station {}", station_id);

                    if let Some(id_pos) = html.find(&id_marker) {
                        // Search backwards from this position for h4 tags which often contain station names
                        let name_start = if id_pos > 2000 { id_pos - 2000 } else { 0 };
                        let name_context = &html[name_start..id_pos];

                        // Look for h4 tags that might contain station name
                        if let Some(h4_pos) = name_context.rfind("<h4") {
                            if let Some(tag_end) = name_context[h4_pos..].find(">") {
                                let h4_start = h4_pos + tag_end + 1;
                                if let Some(h4_end) = name_context[h4_start..].find("</h4>") {
                                    let raw_name = &name_context[h4_start..h4_start + h4_end];
                                    let cleaned = clean_html(raw_name);
                                    if !cleaned.is_empty() && cleaned.len() < 100 {
                                        name = cleaned;
                                    }
                                }
                            }
                        }
                    }

                    // Create stream URL
                    let url = format!("https://stream.rcast.net/{}", station_id);

                    // Add station to our list
                    stations.push(RcastStation {
                        name,
                        url,
                        description: Some(format!("Icecast Radio Station, ID: {}", station_id)),
                        bitrate: None,
                        genre: None,
                        listeners: None,
                    });
                }
            }

            // Continue searching from after this occurrence
            search_pos = after_var + space_pos;
        } else {
            // No space found after var stream, move past this occurrence
            search_pos = after_var;
        }
    }

    Ok(stations)
}

// Helper function to clean HTML text
fn clean_html(text: &str) -> String {
    // Basic HTML cleaning - remove tags, decode entities, etc.
    let result = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("<br>", " ")
        .replace("<br/>", " ")
        .replace("<br />", " ");

    // Remove any HTML tags
    let mut cleaned = String::with_capacity(result.len());
    let mut in_tag = false;

    for c in result.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => cleaned.push(c),
            _ => {}
        }
    }

    // Normalize whitespace
    let mut normalized = String::with_capacity(cleaned.len());
    let mut last_was_space = true; // Start true to trim leading whitespace

    for c in cleaned.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(c);
            last_was_space = false;
        }
    }

    // Trim trailing whitespace
    normalized.trim().to_string()
}

#[allow(dead_code)]
// Convert a RcastStation to a database Station
pub fn rcast_to_db_station(rcast_station: &RcastStation) -> crate::db::Station {
    crate::db::Station {
        id: 0, // This will be assigned by the database
        name: rcast_station.name.clone(),
        url: rcast_station.url.clone(),
        favorite: false,
        description: rcast_station.description.clone(),
    }
}
