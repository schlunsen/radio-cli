import SwiftUI

struct RCastView: View {
    @Environment(StationStore.self) private var stationStore
    @Environment(AudioManager.self) private var audioManager

    @State private var rcastStations: [RcastStation] = []
    @State private var isLoading = false
    @State private var errorMessage: String?
    @State private var searchText = ""

    private var filteredStations: [RcastStation] {
        if searchText.isEmpty { return rcastStations }
        let query = searchText.lowercased()
        return rcastStations.filter {
            $0.name.lowercased().contains(query) ||
            ($0.description?.lowercased().contains(query) ?? false)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Search bar
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Search RCast stations...", text: $searchText)
                    .textFieldStyle(.plain)
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                    }
                    .buttonStyle(.borderless)
                }
            }
            .padding(8)
            .background(.bar)

            if isLoading {
                Spacer()
                ProgressView("Loading stations from RCast...")
                Spacer()
            } else if let error = errorMessage {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                    Text(error)
                        .foregroundStyle(.secondary)
                    Button("Retry") {
                        Task { await loadStations() }
                    }
                }
                Spacer()
            } else if filteredStations.isEmpty {
                Spacer()
                Text(searchText.isEmpty ? "No stations found" : "No matching stations")
                    .foregroundStyle(.secondary)
                Spacer()
            } else {
                List(filteredStations) { station in
                    RCastStationRow(station: station, isInLibrary: stationStore.findStationByURL(station.url) != nil) {
                        audioManager.play(station: Station(
                            id: -1, name: station.name, url: station.url,
                            favorite: false, description: station.description
                        ))
                    } onAdd: {
                        if stationStore.findStationByURL(station.url) == nil {
                            stationStore.addStation(
                                name: station.name,
                                url: station.url,
                                description: station.description
                            )
                        }
                    }
                }
                .listStyle(.plain)
            }
        }
        .toolbar {
            ToolbarItem {
                Button {
                    Task { await loadStations() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .help("Refresh")
                .disabled(isLoading)
            }
        }
        .task {
            if rcastStations.isEmpty {
                await loadStations()
            }
        }
    }

    private func loadStations() async {
        isLoading = true
        errorMessage = nil
        do {
            let stations = try await fetchRCastStations()
            rcastStations = stations
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}

// MARK: - RCast Station Row

struct RCastStationRow: View {
    let station: RcastStation
    let isInLibrary: Bool
    let onPlay: () -> Void
    let onAdd: () -> Void

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(station.name)
                    .font(.body)
                    .fontWeight(.medium)
                    .lineLimit(1)
                if let desc = station.description {
                    Text(desc)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                HStack(spacing: 8) {
                    if let bitrate = station.bitrate {
                        Text(bitrate)
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                    if let listeners = station.listeners {
                        Text("\(listeners) listeners")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                }
            }

            Spacer()

            Button {
                onPlay()
            } label: {
                Image(systemName: "play.circle")
            }
            .buttonStyle(.borderless)
            .help("Preview")

            Button {
                onAdd()
            } label: {
                Image(systemName: isInLibrary ? "checkmark.circle.fill" : "plus.circle")
            }
            .buttonStyle(.borderless)
            .disabled(isInLibrary)
            .help(isInLibrary ? "Already in library" : "Add to library")
        }
        .padding(.vertical, 2)
    }
}
