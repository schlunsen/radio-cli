mod app;
mod audio;
mod db;
mod ui;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Print the current working directory for debugging
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    println!("Current directory: {:?}", current_dir);
    
    // Create and run the application
    let mut app = app::App::new()?;
    app.run()
}