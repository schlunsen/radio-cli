use rodio::{Decoder, Source};
use rust_embed::RustEmbed;
use std::io::Cursor;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Asset;

pub fn get_radio_static_sound() -> Result<impl Source<Item = i16> + Send, String> {
    // Get the embedded radio static sound
    let sound_data = Asset::get("sounds/radio-static.wav")
        .ok_or_else(|| "Failed to find embedded radio static sound".to_string())?;

    // Create a cursor from the static data
    let cursor = Cursor::new(sound_data.data.to_vec());

    // Decode the sound data
    let source =
        Decoder::new(cursor).map_err(|e| format!("Failed to decode embedded sound: {}", e))?;

    Ok(source)
}
