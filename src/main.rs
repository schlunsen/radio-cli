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
                return Ok(());
            }
            "--vis" => {
                show_visualizations = true;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                eprintln!("Try 'radio_cli --help' for more information.");
                return Ok(());
            }
        }
        i += 1;
    }

    // Create and run the application
    let mut app = app::App::new(show_visualizations)?;
    app.run()
}
