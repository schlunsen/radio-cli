import SwiftUI

struct PopoverView: View {
    @Environment(StationStore.self) private var stationStore
    @Environment(AudioManager.self) private var audioManager
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        VStack(spacing: 0) {
            // MARK: - Now Playing
            nowPlayingSection
                .padding(.horizontal, 12)
                .padding(.top, 12)
                .padding(.bottom, 8)

            Divider()

            // MARK: - Station List
            stationList

            Divider()

            // MARK: - Bottom Bar
            bottomBar
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
        }
        .frame(width: 300, height: 420)
    }

    // MARK: - Now Playing Section

    @ViewBuilder
    private var nowPlayingSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            if audioManager.isPlaying, let name = audioManager.currentStationName {
                Text(name)
                    .font(.headline)
                    .lineLimit(1)
                if let song = audioManager.currentSong {
                    Text(song)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                if let error = audioManager.errorMessage {
                    Text(error)
                        .font(.caption2)
                        .foregroundStyle(.red)
                        .lineLimit(1)
                }
            } else {
                Text("Select a station")
                    .font(.headline)
                    .foregroundStyle(.secondary)
            }

            // Playback controls
            HStack(spacing: 8) {
                Button {
                    if audioManager.isPlaying {
                        audioManager.stop()
                    }
                } label: {
                    Image(systemName: audioManager.isPlaying ? "stop.fill" : "play.fill")
                        .frame(width: 20, height: 20)
                }
                .buttonStyle(.borderless)
                .disabled(!audioManager.isPlaying)

                Button {
                    audioManager.toggleMute()
                } label: {
                    Image(systemName: audioManager.isMuted ? "speaker.slash.fill" : "speaker.wave.2.fill")
                        .frame(width: 20, height: 20)
                }
                .buttonStyle(.borderless)

                @Bindable var am = audioManager
                Slider(value: Binding(
                    get: { am.volume },
                    set: { am.setVolume($0) }
                ), in: 0...1)
                .controlSize(.small)
            }
        }
    }

    // MARK: - Station List

    private var stationList: some View {
        List(stationStore.sortedStations) { station in
            StationRow(station: station, isActive: audioManager.currentStationId == station.id)
                .contentShape(Rectangle())
                .onTapGesture {
                    audioManager.play(station: station)
                }
                .contextMenu {
                    Button(station.favorite ? "Unfavorite" : "Favorite") {
                        stationStore.toggleFavorite(id: station.id, favorite: !station.favorite)
                    }
                    Button("Delete", role: .destructive) {
                        if audioManager.currentStationId == station.id {
                            audioManager.stop()
                        }
                        stationStore.deleteStation(id: station.id)
                    }
                }
        }
        .listStyle(.plain)
    }

    // MARK: - Bottom Bar

    private var bottomBar: some View {
        HStack {
            Button("Browse RCast...") {
                openWindow(id: "manager")
                NSApp.activate(ignoringOtherApps: true)
            }
            .buttonStyle(.borderless)
            .font(.caption)

            Spacer()

            Button {
                openWindow(id: "manager")
                NSApp.activate(ignoringOtherApps: true)
            } label: {
                Image(systemName: "list.bullet")
            }
            .buttonStyle(.borderless)
            .help("Manage Stations")

            Button {
                NSApp.sendAction(Selector(("showSettingsWindow:")), to: nil, from: nil)
                NSApp.activate(ignoringOtherApps: true)
            } label: {
                Image(systemName: "gear")
            }
            .buttonStyle(.borderless)
            .help("Settings")

            Button {
                NSApplication.shared.terminate(nil)
            } label: {
                Image(systemName: "power")
            }
            .buttonStyle(.borderless)
            .help("Quit RadioMac")
        }
    }
}

// MARK: - Station Row

struct StationRow: View {
    let station: Station
    let isActive: Bool

    var body: some View {
        HStack(spacing: 6) {
            if station.favorite {
                Image(systemName: "star.fill")
                    .foregroundStyle(.yellow)
                    .font(.caption2)
            }

            VStack(alignment: .leading, spacing: 1) {
                Text(station.name)
                    .font(.system(.body, weight: isActive ? .bold : .regular))
                    .lineLimit(1)
                if let desc = station.description, !desc.isEmpty {
                    Text(desc)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer()

            if isActive {
                Image(systemName: "speaker.wave.2.fill")
                    .foregroundColor(.accentColor)
                    .font(.caption)
            }
        }
        .padding(.vertical, 2)
    }
}
