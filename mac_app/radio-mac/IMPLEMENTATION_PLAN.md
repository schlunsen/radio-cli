# RadioMac Implementation Plan

## Context

RadioMac is a native macOS menu bar app (SwiftUI) that replicates RadioCLI's functionality.
The Xcode project already exists at `mac_app/radio-mac/` with `PBXFileSystemSynchronizedRootGroup`
(new Swift files in `radio-mac/` are auto-discovered — no pbxproj editing needed for sources).

Currently written: `Models.swift`, `StationStore.swift` (both complete).
Still boilerplate: `radio_macApp.swift`, `ContentView.swift`.

The CLI source at `src/` has the reference implementations for DB schema (`src/db/mod.rs`),
audio playback (`src/audio/mod.rs`), and RCast parsing (`src/rcast/mod.rs`).

## Xcode Project Changes (do first)

Edit `mac_app/radio-mac/radio-mac.xcodeproj/project.pbxproj`:

1. In BOTH Debug and Release target-level configs (the ones with `ASSETCATALOG_COMPILER_APPICON_NAME`):
   - Change `ENABLE_APP_SANDBOX = YES` to `ENABLE_APP_SANDBOX = NO`
     (needed to read/write `~/.config/radio_cli/stations.db`)
   - Add `INFOPLIST_KEY_LSUIElement = YES;` (hides app from Dock — menu bar only)

## Files to Create/Modify

All Swift files go in: `mac_app/radio-mac/radio-mac/`

### 1. `AudioManager.swift` (NEW — ~200 lines)

An `@Observable` class wrapping AVPlayer for streaming radio.

```
@Observable
final class AudioManager {
    var isPlaying = false
    var isMuted = false
    var volume: Float = 0.5
    var currentStationName: String?
    var currentStationId: Int?
    var currentSong: String?
    var errorMessage: String?

    private var player: AVPlayer?
    private var playerItem: AVPlayerItem?
    private var metadataObserver: NSKeyValueObservation?
    private var statusObserver: NSKeyValueObservation?
    private var playStartTime: Date?
    private var statsFlushTimer: Timer?
    private var stationStore: StationStore?
    private var reconnectTimer: Timer?
    private var lastURL: URL?
}
```

Key methods:
- `func setStationStore(_ store: StationStore)` — inject store for play-time tracking
- `func play(station: Station)` — stop current, create AVPlayer(url:), observe `.timedMetadata`
  for ICY stream title via KVO, observe `.status` for errors, start play-time timer (flush every 30s)
- `func stop()` — flush play time, pause player, nil everything out, clear Now Playing
- `func toggleMute()` — toggle `isMuted`, set `player?.volume` to 0 or volume
- `func setVolume(_ newVolume: Float)` — clamp 0...1, update player if not muted
- `private func handleMetadata(_ metadata: [AVMetadataItem]?)` — extract `stringValue` for song title
- `private func setupRemoteCommands()` — `MPRemoteCommandCenter.shared()` for play/pause/togglePlayPause
- `private func updateNowPlaying()` — set `MPNowPlayingInfoCenter.default().nowPlayingInfo` with
  title, artist, isLiveStream=true, playbackRate
- `private func scheduleReconnect()` — on failure, wait 3s then `player?.replaceCurrentItem(with:)` retry
- `private func flushPlayTime()` — calculate elapsed since `playStartTime`, call
  `stationStore?.updateStats(stationId:playTime:)`, reset `playStartTime`

Import: `AVFoundation`, `MediaPlayer`

### 2. `RCastService.swift` (NEW — ~120 lines)

Ports `src/rcast/mod.rs` to Swift. Async function using URLSession.

```
enum RCastError: Error {
    case networkError(String)
    case parseError(String)
}

func fetchRCastStations() async throws -> [RcastStation]
```

Implementation:
- URL: `https://www.rcast.net/dir?action=search&search=icecast&sortby=1`
- Use `URLSession.shared.data(for:)` with a custom User-Agent header
- Parse HTML looking for `var stream` + numeric ID pattern (same logic as Rust version)
- For each station ID, look backwards from `id="currentsong_{ID}"` for `<h4>` tag to get name
- Clean HTML entities (`&amp;` -> `&`, etc.) and strip tags
- Stream URL: `https://stream.rcast.net/{ID}`
- Return `[RcastStation]`

### 3. `PopoverView.swift` (NEW — ~200 lines)

The main menu bar popover. ~300px wide, ~450px tall.

Structure (top to bottom):
- **Now Playing section**: if `audioManager.isPlaying`, show station name (bold), current song
  (secondary), else "Select a station". HStack with play/stop button, mute button, volume Slider.
- **Station list**: `List` of `stationStore.sortedStations`. Each row shows star if favorite,
  name bold, description smaller. `onTapGesture { audioManager.play(station:) }`.
  `.contextMenu` with Toggle Favorite, Edit (not implemented in popover — opens manager), Delete.
- **Bottom bar**: HStack with "Browse RCast..." button, "Manage Stations..." button (both open
  the full window via `openWindow(id:)`), gear icon for Settings.

Needs `@Environment(\.openWindow)` to launch full window and settings.

### 4. `StationManagerView.swift` (NEW — ~250 lines)

Full window for station CRUD. Opened via `Window(id: "manager")`.

- `@State` search text for filtering
- `Table` with columns: Name, URL, Description, Favorite (star toggle), Play Time, Last Played
- Toolbar: Add (+) button, Edit (pencil) button, Delete (-) button
- Add/Edit shows a `.sheet` with TextFields for name, url, description and Save/Cancel buttons
- Delete shows confirmation alert
- Search field in toolbar filters `stationStore.stations` by name/description

### 5. `RCastView.swift` (NEW — ~150 lines)

Tab or section in the manager window for RCast discovery.

- `@State var rcastStations: [RcastStation] = []`
- `@State var isLoading = false`
- `.task { }` to fetch on appear
- List showing each RcastStation: name, description, bitrate, listeners
- Click to preview (play via audioManager)
- "Add to Library" button — calls `stationStore.addStation(name:url:description:)`
  only if `stationStore.findStationByURL()` returns nil (avoid duplicates)
- Refresh button in toolbar
- Search/filter field

### 6. `StatsView.swift` (NEW — ~100 lines)

Statistics display tab/section.

- Show `stationStore.topStations` as a list with rank number, station name, formatted play time
- Total listening time (sum of all play times)
- Last played timestamps formatted as relative dates
- Simple bar visualization using `Rectangle()` with proportional widths

### 7. `SettingsView.swift` (NEW — ~60 lines)

Preferences window via `Settings { }` scene.

- Toggle: "Launch at Login" — use `SMAppService.mainApp` (ServiceManagement framework)
- Slider: "Default Volume" — stored in `@AppStorage("defaultVolume")`
- Info: database path display

### 8. `radio_macApp.swift` (REWRITE — ~80 lines)

Replace current boilerplate with MenuBarExtra app:

```
import SwiftUI

@main
struct RadioMacApp: App {
    @State private var stationStore = StationStore()
    @State private var audioManager = AudioManager()
    @State private var iconPhase = 0
    @State private var iconTimer: Timer?

    var body: some Scene {
        MenuBarExtra {
            PopoverView()
                .environment(stationStore)
                .environment(audioManager)
        } label: {
            Image(systemName: menuBarIconName)
        }
        .menuBarExtraStyle(.window)

        Window("Station Manager", id: "manager") {
            TabView {
                StationManagerView().tabItem { Label("Stations", systemImage: "radio") }
                RCastView().tabItem { Label("Discover", systemImage: "globe") }
                StatsView().tabItem { Label("Statistics", systemImage: "chart.bar") }
            }
            .frame(minWidth: 700, minHeight: 500)
            .environment(stationStore)
            .environment(audioManager)
        }

        Settings {
            SettingsView()
        }
    }

    var menuBarIconName: String {
        if audioManager.isMuted && audioManager.isPlaying {
            return "speaker.slash"
        } else if audioManager.isPlaying {
            let icons = ["speaker.wave.1", "speaker.wave.2", "speaker.wave.3"]
            return icons[iconPhase % icons.count]
        }
        return "radio"
    }
}
```

On `audioManager.isPlaying` change: start/stop a 0.5s timer that increments `iconPhase`.
Wire up `audioManager.setStationStore(stationStore)` in init or `.onAppear`.

### 9. `ContentView.swift` (DELETE or leave empty)

No longer needed — PopoverView replaces it. Can delete the file or leave it unused.

## Key Technical Notes

- The project uses `SWIFT_DEFAULT_ACTOR_ISOLATION = MainActor` (Swift 6 strict concurrency).
  All `@Observable` classes are implicitly `@MainActor`. Use `nonisolated` or `@Sendable`
  for background work. `async` functions in RCastService need proper isolation.
- `MACOSX_DEPLOYMENT_TARGET = 26.2` in pbxproj — this is macOS Tahoe. Fine to keep or
  lower to 14.0 (Sonoma) for broader compatibility.
- `MenuBarExtra` with `.menuBarExtraStyle(.window)` gives the popover panel style.
- `@Observable` (Observation framework) is used instead of `ObservableObject`/`@Published`.
  Pass via `.environment()` and read with `@Environment`.
- SQLite3 is a system library — just `import SQLite3`, no package dependency needed.
- App Sandbox MUST be disabled for `~/.config/` access.
- `LSUIElement = YES` hides from Dock (menu bar only app).

## Build Order

1. Xcode project settings (sandbox, LSUIElement)
2. AudioManager.swift
3. RCastService.swift
4. PopoverView.swift
5. radio_macApp.swift (rewrite)
6. StationManagerView.swift
7. RCastView.swift
8. StatsView.swift
9. SettingsView.swift
10. Delete/clean ContentView.swift
11. Build & test in Xcode
