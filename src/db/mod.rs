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

pub fn find_station_by_url(conn: &Connection, url: &str) -> Result<Option<i32>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT id FROM stations WHERE url = ?1")?;
    let mut rows = stmt.query_map(params![url], |row| row.get::<_, i32>(0))?;
    if let Some(row) = rows.next() {
        return Ok(Some(row?));
    }
    Ok(None)
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

/// Function to find and remove duplicate URLs in the stations database
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        init_db(&conn).expect("Failed to initialize database");
        // Clear default stations for clean tests
        conn.execute("DELETE FROM stations", []).unwrap();
        conn.execute("DELETE FROM station_stats", []).unwrap();
        conn
    }

    #[test]
    fn test_add_and_load_stations() {
        let conn = setup_test_db();

        let id1 = add_station(&conn, "Test Station 1", "http://test1.com", Some("Desc 1")).unwrap();
        let id2 = add_station(&conn, "Test Station 2", "http://test2.com", None).unwrap();

        assert!(id1 > 0);
        assert!(id2 > 0);
        assert_ne!(id1, id2);

        let stations = load_stations(&conn).unwrap();
        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].name, "Test Station 1");
        assert_eq!(stations[1].url, "http://test2.com");
        assert_eq!(stations[0].description, Some("Desc 1".to_string()));
        assert_eq!(stations[1].description, None);
    }

    #[test]
    fn test_delete_station() {
        let conn = setup_test_db();

        let id = add_station(&conn, "To Delete", "http://delete.me", None).unwrap();
        assert_eq!(load_stations(&conn).unwrap().len(), 1);

        delete_station(&conn, id).unwrap();
        assert_eq!(load_stations(&conn).unwrap().len(), 0);
    }

    #[test]
    fn test_update_station() {
        let conn = setup_test_db();

        let id = add_station(&conn, "Original", "http://original.com", None).unwrap();
        update_station(&conn, id, "Updated", "http://updated.com", Some("New desc")).unwrap();

        let stations = load_stations(&conn).unwrap();
        assert_eq!(stations[0].name, "Updated");
        assert_eq!(stations[0].url, "http://updated.com");
        assert_eq!(stations[0].description, Some("New desc".to_string()));
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = setup_test_db();

        let id = add_station(&conn, "Fav Test", "http://fav.com", None).unwrap();

        let stations = load_stations(&conn).unwrap();
        assert!(!stations[0].favorite);

        toggle_favorite(&conn, id, true).unwrap();
        let stations = load_stations(&conn).unwrap();
        assert!(stations[0].favorite);

        toggle_favorite(&conn, id, false).unwrap();
        let stations = load_stations(&conn).unwrap();
        assert!(!stations[0].favorite);
    }

    #[test]
    fn test_remove_duplicate_urls() {
        let conn = setup_test_db();

        add_station(&conn, "Station 1", "http://test1.com", Some("Test 1")).unwrap();
        add_station(&conn, "Station 2", "http://test2.com", Some("Test 2")).unwrap();

        // Insert duplicates directly via SQL to bypass normal checks
        conn.execute(
            "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
            params![
                "Station 1 Duplicate",
                "http://test1.com",
                "Duplicate of Test 1"
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
            params![
                "Station 2 Duplicate",
                "http://test2.com",
                "Duplicate of Test 2"
            ],
        )
        .unwrap();

        let count_before: i32 = conn
            .query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_before, 4);

        remove_duplicate_urls(&conn).unwrap();

        let count_after: i32 = conn
            .query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_after, 2);
    }

    #[test]
    fn test_station_stats() {
        let conn = setup_test_db();

        let id = add_station(&conn, "Stats Test", "http://stats.com", None).unwrap();

        // No stats initially
        let stats = get_station_stats(&conn, id).unwrap();
        assert!(stats.is_none());

        // Add play time
        update_station_stats(&conn, id, 60).unwrap();
        let stats = get_station_stats(&conn, id).unwrap().unwrap();
        assert_eq!(stats.total_play_time, 60);
        assert!(stats.last_played.is_some());

        // Accumulate play time
        update_station_stats(&conn, id, 30).unwrap();
        let stats = get_station_stats(&conn, id).unwrap().unwrap();
        assert_eq!(stats.total_play_time, 90);
    }

    #[test]
    fn test_get_top_stations() {
        let conn = setup_test_db();

        let id1 = add_station(&conn, "Low Play", "http://low.com", None).unwrap();
        let id2 = add_station(&conn, "High Play", "http://high.com", None).unwrap();
        let id3 = add_station(&conn, "Mid Play", "http://mid.com", None).unwrap();

        update_station_stats(&conn, id1, 10).unwrap();
        update_station_stats(&conn, id2, 100).unwrap();
        update_station_stats(&conn, id3, 50).unwrap();

        let top = get_top_stations(&conn, 2).unwrap();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0.name, "High Play");
        assert_eq!(top[0].1, 100);
        assert_eq!(top[1].0.name, "Mid Play");
        assert_eq!(top[1].1, 50);
    }

    #[test]
    fn test_find_station_by_url() {
        let conn = setup_test_db();

        let id = add_station(&conn, "Find Me", "http://findme.com", None).unwrap();

        assert_eq!(
            find_station_by_url(&conn, "http://findme.com").unwrap(),
            Some(id)
        );
        assert_eq!(
            find_station_by_url(&conn, "http://notfound.com").unwrap(),
            None
        );
    }

    #[test]
    fn test_format_play_time() {
        assert_eq!(format_play_time(0), "0s");
        assert_eq!(format_play_time(30), "30s");
        assert_eq!(format_play_time(59), "59s");
        assert_eq!(format_play_time(60), "1m 0s");
        assert_eq!(format_play_time(90), "1m 30s");
        assert_eq!(format_play_time(3599), "59m 59s");
        assert_eq!(format_play_time(3600), "1h 0m");
        assert_eq!(format_play_time(7260), "2h 1m");
    }
}
