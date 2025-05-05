use rusqlite::{params, Connection, Result};
use std::error::Error;

pub struct Station {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub favorite: bool,
    pub description: Option<String>,
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
