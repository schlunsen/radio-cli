# RadioMac — macOS Menu Bar App Design

**Date:** 2026-04-05
**Status:** Approved

## Overview

RadioMac is a native macOS menu bar app that provides the same radio-playing experience
as RadioCLI without requiring a terminal. It sits in the system tray as an icon with two
UI surfaces: a compact popover for daily use and a full window for station management.

## Key Decisions

- **Audio backend:** AVFoundation/AVPlayer (native macOS, media key support, AirPlay, Now Playing integration)
- **Database:** Shared SQLite DB with the CLI at `~/.config/radio_cli/stations.db`
- **Interaction model:** Menu bar popover + separate full window for management
- **Menu bar icon:** Animated based on playback state (idle/playing/muted)
- **Dependencies:** Zero external — pure SwiftUI + AVFoundation + sqlite3

## Architecture

### App Layers

- **`RadioMacApp`** — Entry point. Configures `MenuBarExtra` with animated icon. No dock icon (`LSUIElement = true`).
- **`AudioManager`** — Observable wrapping AVPlayer. Handles stream playback, volume, mute, metadata parsing from ICY headers, and Now Playing integration (MPNowPlayingInfoCenter + MPRemoteCommandCenter for media keys).
- **`StationStore`** — Observable that reads/writes the shared SQLite DB. Uses the same schema the CLI uses. Polls for external changes every 5 seconds.
- **`RCastService`** — Async fetcher for RCast.net station discovery, same parsing logic ported to Swift.

### Shared Database Schema

Same tables as the CLI:

    stations (id INTEGER PRIMARY KEY, name TEXT NOT NULL, url TEXT NOT NULL, favorite INTEGER NOT NULL DEFAULT 0, description TEXT)
    station_stats (station_id INTEGER PRIMARY KEY, total_play_time INTEGER NOT NULL DEFAULT 0, last_played INTEGER, FOREIGN KEY (station_id) REFERENCES stations(id) ON DELETE CASCADE)

WAL mode for safe concurrent access with the CLI.

## Popover UI (~300px wide, ~450px tall)

### Now Playing (top)
- Station name (bold), current song/title (secondary)
- Audio format + bitrate label (e.g. "MP3 - 128kbps")
- Horizontal row: Play/Stop toggle, Mute button, Volume slider
- Empty state: "Select a station to start listening"

### Station List (middle, scrollable)
- Favorites pinned at top with star indicator
- Each row: station name, description in smaller text
- Click to play immediately (replaces current stream)
- Right-click context menu: Edit, Delete, Toggle Favorite

### Bottom Bar
- "Browse RCast..." button — opens full window to RCast tab
- "Manage Stations..." button — opens full window
- Gear icon for preferences

## Full Window (Tab-based)

### Stations Tab
- Table view: Name, URL, Description, Favorite, Total Play Time, Last Played
- Sortable columns
- Toolbar: Add (+), Delete (-), Edit (pencil)
- Add/Edit via sheet with text fields
- Search field in toolbar

### RCast Discovery Tab
- Loads stations from RCast.net on tab open
- List: station name, genre, bitrate, listener count, description
- Click to preview (plays), "Add to Library" to save to DB
- Refresh + search/filter

### Statistics Tab
- Top stations by play time
- Total listening time
- Last played timestamps
- Bar chart of top 10 most-played

### Preferences (Cmd+,)
- Launch at login toggle
- Global keyboard shortcut for play/pause
- Default volume level

## Menu Bar Icon States

- Idle: `radio` (static SF Symbol)
- Playing: Cycles `speaker.wave.1` / `.2` / `.3` (0.5s timer)
- Muted while playing: `speaker.slash`

## Stream Handling

- `AVPlayer(url:)` with station stream URL
- ICY metadata via KVO on `AVPlayerItem.timedMetadata` — parse `StreamTitle` for current song
- Network interruption handling with auto-reconnect after 3 seconds
- Play time tracking: timer on playback start, flush to `station_stats` every 30 seconds and on stop

## Build and Distribution

- Xcode project in `mac_app/radio-mac/`
- Minimum deployment: macOS 14 (Sonoma)
- No external dependencies
- Future distribution: DMG or Homebrew Cask

## File Structure

    radio_macApp.swift       — App entry point, MenuBarExtra setup, animated icon
    Models.swift             — Station, StationStats, RcastStation data models
    StationStore.swift       — SQLite wrapper, CRUD, stats, shared DB access
    AudioManager.swift       — AVPlayer wrapper, metadata, Now Playing, media keys
    RCastService.swift       — Async HTML fetcher/parser for RCast.net
    PopoverView.swift        — Main popover: now playing + station list + controls
    StationManagerView.swift — Full window: stations tab with add/edit/delete
    RCastView.swift          — Full window: RCast discovery tab
    StatsView.swift          — Full window: statistics tab
    SettingsView.swift       — Preferences window

## Xcode Project Changes Needed

- Disable App Sandbox (`ENABLE_APP_SANDBOX = NO`) for `~/.config/` file access
- Add `INFOPLIST_KEY_LSUIElement = YES` to hide from Dock
