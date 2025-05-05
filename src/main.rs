mod app;
mod audio;
mod db;
mod ui;

use std::env;
use std::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    // Check for command-line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("RadioCLI v{}", VERSION);
                return Ok(());
            },
            "--help" | "-h" => {
                println!("RadioCLI - Terminal-based internet radio player with visualizations\n");
                println!("Usage: radio_cli [OPTIONS]");
                println!("\nOptions:");
                println!("  -v, --version    Print version information");
                println!("  -h, --help       Print this help message");
                return Ok(());
            },
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Try 'radio_cli --help' for more information.");
                return Ok(());
            }
        }
    }
    
    // Create and run the application
    let mut app = app::App::new()?;
    app.run()
}