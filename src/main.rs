mod app;
mod audio;
mod db;
mod rcast;
mod ui;
mod visualizations;

use std::env;
use std::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    // Check for command-line arguments
    let args: Vec<String> = env::args().collect();

    // Default setting for visualizations (disabled by default)
    let mut show_visualizations = false;
    let mut test_duplicate_removal = false;

    // Check for args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-v" => {
                println!("RadioCLI v{}", VERSION);
                return Ok(());
            }
            "--help" | "-h" => {
                println!("RadioCLI - Terminal-based internet radio player with visualizations\n");
                println!("Usage: radio_cli [OPTIONS]");
                println!("\nOptions:");
                println!("  -v, --version    Print version information");
                println!("  -h, --help       Print this help message");
                println!("  --vis            Enable visualizations (disabled by default)");
                println!("  --test-dupes     Run a test to verify duplicate URL removal");
                return Ok(());
            }
            "--vis" => {
                show_visualizations = true;
            }
            "--test-dupes" => {
                test_duplicate_removal = true;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                eprintln!("Try 'radio_cli --help' for more information.");
                return Ok(());
            }
        }
        i += 1;
    }

    // Test duplicate removal if requested
    if test_duplicate_removal {
        return test_duplicate_url_removal();
    }

    // Create and run the application
    let mut app = app::App::new(show_visualizations)?;
    app.run()
}

// Function to test the duplicate URL removal functionality
fn test_duplicate_url_removal() -> Result<(), Box<dyn Error>> {
    use rusqlite::Connection;

    println!("Running duplicate URL removal test...");

    // Create an in-memory database for testing
    let conn = Connection::open_in_memory()?;

    // Initialize the database schema
    db::init_db(&conn)?;

    // Add some test stations with duplicate URLs
    println!("Adding test stations with duplicate URLs...");

    // Add original stations
    let _station1_id = db::add_station(&conn, "Station 1", "http://test1.com", Some("Test 1"))?;
    let _station2_id = db::add_station(&conn, "Station 2", "http://test2.com", Some("Test 2"))?;

    // Add duplicate URLs with different names
    println!("Adding duplicate URLs...");

    // These should be direct SQL inserts to bypass the normal checks
    conn.execute(
        "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            "Station 1 Duplicate",
            "http://test1.com",
            "Duplicate of Test 1"
        ],
    )?;

    conn.execute(
        "INSERT INTO stations (name, url, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            "Station 2 Duplicate",
            "http://test2.com",
            "Duplicate of Test 2"
        ],
    )?;

    // Count stations before deduplication
    let count_before: i32 =
        conn.query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))?;

    println!("Before deduplication: {} stations", count_before);
    println!("Stations before deduplication:");

    // List all stations before deduplication
    let stations_before = db::load_stations(&conn)?;
    for station in &stations_before {
        println!(
            "ID: {}, Name: {}, URL: {}",
            station.id, station.name, station.url
        );
    }

    // Now remove duplicates
    println!("\nRemoving duplicates...");
    db::remove_duplicate_urls(&conn)?;

    // Count stations after deduplication
    let count_after: i32 = conn.query_row("SELECT COUNT(*) FROM stations", [], |row| row.get(0))?;

    println!("After deduplication: {} stations", count_after);
    println!("Stations after deduplication:");

    // List all stations after deduplication
    let stations_after = db::load_stations(&conn)?;
    for station in &stations_after {
        println!(
            "ID: {}, Name: {}, URL: {}",
            station.id, station.name, station.url
        );
    }

    // Verify that duplicates were removed
    if count_before == 4 && count_after == 2 {
        println!("\nDuplicate removal test PASSED ✓");
        println!("The remove_duplicate_urls function works correctly.");
    } else {
        println!("\nDuplicate removal test FAILED ✗");
        println!(
            "Expected 4 stations before and 2 after, but got {} before and {} after.",
            count_before, count_after
        );
    }

    Ok(())
}
