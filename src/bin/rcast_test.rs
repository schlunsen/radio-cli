use std::error::Error;

// Import the main crate's modules
use radio_cli::rcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing RCast station fetching...");

    // Try to fetch stations
    match rcast::fetch_stations().await {
        Ok(stations) => {
            println!("Successfully fetched {} stations:", stations.len());

            // Print the first 5 stations or all if less than 5
            let display_count = std::cmp::min(5, stations.len());
            for (i, station) in stations.iter().take(display_count).enumerate() {
                println!("\nStation {}:", i + 1);
                println!("  Name: {}", station.name);
                println!("  URL: {}", station.url);
                if let Some(desc) = &station.description {
                    println!("  Description: {}", desc);
                }
                if let Some(genre) = &station.genre {
                    println!("  Genre: {}", genre);
                }
                if let Some(bitrate) = &station.bitrate {
                    println!("  Bitrate: {}", bitrate);
                }
                if let Some(listeners) = station.listeners {
                    println!("  Listeners: {}", listeners);
                }
            }

            if stations.len() > display_count {
                println!("\n... and {} more stations", stations.len() - display_count);
            }
        }
        Err(e) => {
            println!("Error fetching stations: {}", e);
        }
    }

    Ok(())
}
