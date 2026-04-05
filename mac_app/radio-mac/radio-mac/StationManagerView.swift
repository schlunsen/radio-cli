import SwiftUI

struct StationManagerView: View {
    @Environment(StationStore.self) private var stationStore
    @Environment(AudioManager.self) private var audioManager

    @State private var searchText = ""
    @State private var selection: Station.ID?
    @State private var showAddSheet = false
    @State private var showEditSheet = false
    @State private var showDeleteAlert = false

    // Sheet fields
    @State private var editName = ""
    @State private var editURL = ""
    @State private var editDescription = ""

    private var filteredStations: [Station] {
        if searchText.isEmpty {
            return stationStore.sortedStations
        }
        let query = searchText.lowercased()
        return stationStore.sortedStations.filter {
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
                TextField("Search stations...", text: $searchText)
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

            // Station table
            Table(filteredStations, selection: $selection) {
                TableColumn("") { station in
                    if station.favorite {
                        Image(systemName: "star.fill")
                            .foregroundStyle(.yellow)
                            .font(.caption)
                    }
                }
                .width(20)

                TableColumn("Name") { station in
                    Text(station.name)
                        .fontWeight(audioManager.currentStationId == station.id ? .bold : .regular)
                }
                .width(min: 150, ideal: 200)

                TableColumn("URL") { station in
                    Text(station.url)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .width(min: 150, ideal: 200)

                TableColumn("Description") { station in
                    Text(station.description ?? "")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .width(min: 100, ideal: 150)

                TableColumn("Play Time") { station in
                    let stats = stationStore.getStats(stationId: station.id)
                    Text(stats.map { formatPlayTime($0.totalPlayTime) } ?? "-")
                        .font(.caption)
                        .monospacedDigit()
                }
                .width(70)
            }
            .contextMenu(forSelectionType: Station.ID.self) { ids in
                if let id = ids.first {
                    Button("Play") {
                        if let station = stationStore.stations.first(where: { $0.id == id }) {
                            audioManager.play(station: station)
                        }
                    }
                    Button("Edit") {
                        if let station = stationStore.stations.first(where: { $0.id == id }) {
                            prepareEdit(station)
                        }
                    }
                    Divider()
                    Button("Delete", role: .destructive) {
                        selection = id
                        showDeleteAlert = true
                    }
                }
            }
        }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    editName = ""
                    editURL = ""
                    editDescription = ""
                    showAddSheet = true
                } label: {
                    Image(systemName: "plus")
                }
                .help("Add Station")

                Button {
                    if let id = selection,
                       let station = stationStore.stations.first(where: { $0.id == id }) {
                        prepareEdit(station)
                    }
                } label: {
                    Image(systemName: "pencil")
                }
                .help("Edit Station")
                .disabled(selection == nil)

                Button {
                    showDeleteAlert = true
                } label: {
                    Image(systemName: "minus")
                }
                .help("Delete Station")
                .disabled(selection == nil)
            }
        }
        .sheet(isPresented: $showAddSheet) {
            stationFormSheet(title: "Add Station") {
                stationStore.addStation(name: editName, url: editURL, description: editDescription.isEmpty ? nil : editDescription)
            }
        }
        .sheet(isPresented: $showEditSheet) {
            stationFormSheet(title: "Edit Station") {
                if let id = selection {
                    stationStore.updateStation(id: id, name: editName, url: editURL, description: editDescription.isEmpty ? nil : editDescription)
                }
            }
        }
        .alert("Delete Station?", isPresented: $showDeleteAlert) {
            Button("Delete", role: .destructive) {
                if let id = selection {
                    if audioManager.currentStationId == id {
                        audioManager.stop()
                    }
                    stationStore.deleteStation(id: id)
                    selection = nil
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This action cannot be undone.")
        }
    }

    private func prepareEdit(_ station: Station) {
        editName = station.name
        editURL = station.url
        editDescription = station.description ?? ""
        selection = station.id
        showEditSheet = true
    }

    @ViewBuilder
    private func stationFormSheet(title: String, action: @escaping () -> Void) -> some View {
        VStack(spacing: 16) {
            Text(title)
                .font(.headline)

            Form {
                TextField("Name:", text: $editName)
                TextField("URL:", text: $editURL)
                TextField("Description:", text: $editDescription)
            }
            .formStyle(.grouped)

            HStack {
                Button("Cancel") {
                    showAddSheet = false
                    showEditSheet = false
                }
                .keyboardShortcut(.cancelAction)

                Spacer()

                Button("Save") {
                    action()
                    showAddSheet = false
                    showEditSheet = false
                }
                .keyboardShortcut(.defaultAction)
                .disabled(editName.isEmpty || editURL.isEmpty)
            }
        }
        .padding()
        .frame(width: 400)
    }
}
