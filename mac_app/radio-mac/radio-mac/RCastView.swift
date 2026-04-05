import SwiftUI

struct RCastView: View {
    @Environment(StationStore.self) private var stationStore
    @Environment(AudioManager.self) private var audioManager

    @State private var stations: [RadioBrowserStation] = []
    @State private var isLoading = false
    @State private var errorMessage: String?
    @State private var searchText = ""
    @State private var selectedTag = "all"

    private let tagOptions = [
        "all", "pop", "rock", "jazz", "classical", "electronic",
        "hiphop", "ambient", "chillout", "news", "talk", "lounge"
    ]

    private var filteredStations: [RadioBrowserStation] {
        if searchText.isEmpty { return stations }
        let query = searchText.lowercased()
        return stations.filter {
            $0.name.lowercased().contains(query) ||
            ($0.tags?.lowercased().contains(query) ?? false) ||
            ($0.country?.lowercased().contains(query) ?? false)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Search + filter bar
            VStack(spacing: 6) {
                HStack {
                    Image(systemName: "magnifyingglass")
                        .foregroundStyle(.secondary)
                    TextField("Search stations...", text: $searchText)
                        .textFieldStyle(.plain)
                        .onSubmit { Task { await searchStations() } }
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

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        ForEach(tagOptions, id: \.self) { tag in
                            Button(tag.capitalized) {
                                selectedTag = tag
                                Task { await loadStations() }
                            }
                            .buttonStyle(.bordered)
                            .tint(selectedTag == tag ? .accentColor : .secondary)
                            .controlSize(.small)
                        }
                    }
                }
            }
            .padding(8)
            .background(.bar)

            if isLoading {
                Spacer()
                ProgressView("Loading stations...")
                Spacer()
            } else if let error = errorMessage {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                    Text(error)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                    Button("Retry") {
                        Task { await loadStations() }
                    }
                }
                .padding()
                Spacer()
            } else if filteredStations.isEmpty {
                Spacer()
                Text(searchText.isEmpty ? "No stations found" : "No matching stations")
                    .foregroundStyle(.secondary)
                Spacer()
            } else {
                List(filteredStations) { station in
                    BrowseStationRow(
                        station: station,
                        isInLibrary: stationStore.findStationByURL(station.url) != nil
                    ) {
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
            if stations.isEmpty {
                await loadStations()
            }
        }
    }

    private func loadStations() async {
        isLoading = true
        errorMessage = nil
        do {
            var filter = RadioBrowserFilter()
            if selectedTag != "all" {
                filter.tag = selectedTag
            }
            let result = try await fetchRadioBrowserStations(filter: filter)
            stations = result
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    private func searchStations() async {
        guard !searchText.isEmpty else {
            await loadStations()
            return
        }
        isLoading = true
        errorMessage = nil
        do {
            var filter = RadioBrowserFilter()
            filter.name = searchText
            if selectedTag != "all" {
                filter.tag = selectedTag
            }
            let result = try await fetchRadioBrowserStations(filter: filter)
            stations = result
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}

// MARK: - Browse Station Row

struct BrowseStationRow: View {
    let station: RadioBrowserStation
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

                if !station.description.isEmpty {
                    Text(station.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                HStack(spacing: 8) {
                    if let clicks = station.clickcount, clicks > 0 {
                        Label("\(clicks)", systemImage: "hand.tap")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                    if let votes = station.votes, votes > 0 {
                        Label("\(votes)", systemImage: "heart")
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
