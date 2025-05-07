use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Station {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub favorite: bool,
    pub description: Option<String>,
}

pub struct StationStats {
    #[allow(dead_code)]
    pub station_id: i32,
    pub total_play_time: i64,     // Total play time in seconds
    pub last_played: Option<i64>, // Unix timestamp of last play
}

pub fn init_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            favorite INTEGER NOT NULL DEFAULT 0,
            description TEXT
        )",
        [],
    )?;

    // Create stats table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS station_stats (
            station_id INTEGER PRIMARY KEY,
            total_play_time INTEGER NOT NULL DEFAULT 0,
            last_played INTEGER,
            FOREIGN KEY (station_id) REFERENCES stations(id) ON DELETE CASCADE
        )",
        [],
    )?;

    let count: i32 = conn.query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))?;
    if count == 0 {
        let stations = vec![
            // Original stations with descriptions
            (
                "Groove Salad (SomaFM)",
                "http://ice1.somafm.com/groovesalad-128-mp3",
                "Chilled electronic and downtempo beats",
            ),
            (
                "Secret Agent (SomaFM)",
                "http://ice4.somafm.com/secretagent-128-mp3",
                "The soundtrack for your stylish, mysterious, dangerous life",
            ),
            (
                "BBC Radio 1",
                "http://icecast.omroep.nl/radio1-bb-mp3",
                "BBC's flagship radio station for new music and entertainment",
            ),
            // Added FluxFM Chillhop
            (
                "FluxFM Chillhop",
                "https://streams.fluxfm.de/Chillhop/mp3-320/streams.fluxfm.de/",
                "High-quality Chillhop stream from FluxFM - relaxed beats at 320kbps",
            ),
        ];
        for (name, url, description) in stations {
            conn.execute(
                "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
                params![name, url, description],
            )?;
        }
    }
    Ok(())
}

pub fn load_stations(conn: &Connection) -> Result<Vec<Station>, Box<dyn Error>> {
    // Remove any duplicate URLs before loading stations
    remove_duplicate_urls(conn)?;

    let mut stmt = conn.prepare("SELECT id, name, url, favorite, description FROM stations")?;
    let station_iter = stmt.query_map([], |row| {
        Ok(Station {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            favorite: row.get::<_, i32>(3)? != 0,
            description: row.get(4)?,
        })
    })?;
    let mut stations = Vec::new();
    for station in station_iter {
        stations.push(station?);
    }
    Ok(stations)
}

pub fn toggle_favorite(
    conn: &Connection,
    station_id: i32,
    new_favorite: bool,
) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE stations SET favorite = ?1 WHERE id = ?2",
        params![new_favorite as i32, station_id],
    )?;
    Ok(())
}

pub fn add_station(
    conn: &Connection,
    name: &str,
    url: &str,
    description: Option<&str>,
) -> Result<i32, Box<dyn Error>> {
    conn.execute(
        "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
        params![name, url, description],
    )?;

    // Get the ID of the newly inserted station
    let id: i32 = conn.last_insert_rowid() as i32;
    Ok(id)
}

pub fn delete_station(conn: &Connection, station_id: i32) -> Result<(), Box<dyn Error>> {
    conn.execute("DELETE FROM stations WHERE id = ?1", params![station_id])?;
    Ok(())
}

pub fn update_station(
    conn: &Connection,
    station_id: i32,
    name: &str,
    url: &str,
    description: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE stations SET name = ?1, url = ?2, description = ?3 WHERE id = ?4",
        params![name, url, description, station_id],
    )?;
    Ok(())
}

// Station usage statistics functions

pub fn update_station_stats(
    conn: &Connection,
    station_id: i32,
    play_time: i64,
) -> Result<(), Box<dyn Error>> {
    // Get current unix timestamp
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_secs() as i64;

    // Try to update existing record first
    let updated = conn.execute(
        "UPDATE station_stats 
         SET total_play_time = total_play_time + ?1, last_played = ?2 
         WHERE station_id = ?3",
        params![play_time, now, station_id],
    )?;

    // If no record was updated, insert a new one
    if updated == 0 {
        conn.execute(
            "INSERT INTO station_stats (station_id, total_play_time, last_played) 
             VALUES (?1, ?2, ?3)",
            params![station_id, play_time, now],
        )?;
    }

    Ok(())
}

pub fn get_station_stats(
    conn: &Connection,
    station_id: i32,
) -> Result<Option<StationStats>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT station_id, total_play_time, last_played 
         FROM station_stats 
         WHERE station_id = ?1",
    )?;

    let mut stats = stmt.query_map(params![station_id], |row| {
        Ok(StationStats {
            station_id: row.get(0)?,
            total_play_time: row.get(1)?,
            last_played: row.get(2)?,
        })
    })?;

    // Return the first (and only) result, or None if no stats exist
    if let Some(stat) = stats.next() {
        return Ok(Some(stat?));
    }

    Ok(None)
}

pub fn get_top_stations(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<(Station, i64)>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.url, s.favorite, s.description, st.total_play_time
         FROM stations s
         JOIN station_stats st ON s.id = st.station_id
         ORDER BY st.total_play_time DESC
         LIMIT ?1",
    )?;

    let results = stmt.query_map(params![limit as i64], |row| {
        Ok((
            Station {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                favorite: row.get::<_, i32>(3)? != 0,
                description: row.get(4)?,
            },
            row.get::<_, i64>(5)?,
        ))
    })?;

    let mut stations = Vec::new();
    for result in results {
        stations.push(result?);
    }

    Ok(stations)
}

pub fn format_play_time(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

// Function to find and remove duplicate URLs in the stations database
pub fn remove_duplicate_urls(conn: &Connection) -> Result<(), Box<dyn Error>> {
    // First find all duplicate URLs
    let mut find_stmt = conn.prepare(
        "SELECT url, COUNT(*) as count, MIN(id) as min_id 
         FROM stations 
         GROUP BY url 
         HAVING count > 1",
    )?;

    // Collect all duplicates first to avoid borrowing issues
    let mut duplicates_to_process = Vec::new();
    {
        let duplicate_rows = find_stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?, // URL
                row.get::<_, i64>(1)?,    // Count of occurrences
                row.get::<_, i32>(2)?,    // Minimum ID (the first occurrence)
            ))
        })?;

        // Use flatten to process only the Ok values
        for dup in duplicate_rows.flatten() {
            duplicates_to_process.push(dup);
        }
    }

    // Close the statement explicitly
    drop(find_stmt);

    // Process each duplicate URL
    for (url, count, min_id) in duplicates_to_process {
        // Log information about the duplicates (for debugging)
        eprintln!("Found {} duplicate entries for URL: {}", count, url);

        // Delete all occurrences of this URL except the one with the minimum ID
        let deleted = conn.execute(
            "DELETE FROM stations WHERE url = ?1 AND id != ?2",
            params![url, min_id],
        )?;

        eprintln!("Removed {} duplicate entries", deleted);
    }

    Ok(())
}
