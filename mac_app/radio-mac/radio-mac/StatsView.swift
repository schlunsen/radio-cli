import SwiftUI

struct StatsView: View {
    @Environment(StationStore.self) private var stationStore

    private var totalListeningTime: Int64 {
        stationStore.topStations.reduce(0) { $0 + $1.1 }
    }

    private var maxPlayTime: Int64 {
        stationStore.topStations.first?.1 ?? 1
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Summary
            HStack {
                VStack(alignment: .leading) {
                    Text("Total Listening Time")
                        .font(.headline)
                    Text(formatPlayTime(totalListeningTime))
                        .font(.title)
                        .monospacedDigit()
                        .foregroundColor(.accentColor)
                }
                Spacer()
                VStack(alignment: .trailing) {
                    Text("Stations Played")
                        .font(.headline)
                    Text("\(stationStore.topStations.count)")
                        .font(.title)
                        .monospacedDigit()
                        .foregroundColor(.accentColor)
                }
            }
            .padding()
            .background(.bar, in: RoundedRectangle(cornerRadius: 8))

            // Top Stations
            if stationStore.topStations.isEmpty {
                Spacer()
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "chart.bar")
                            .font(.largeTitle)
                            .foregroundStyle(.secondary)
                        Text("No listening history yet")
                            .foregroundStyle(.secondary)
                        Text("Play some stations to see your stats!")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                    }
                    Spacer()
                }
                Spacer()
            } else {
                Text("Top Stations")
                    .font(.headline)

                List {
                    ForEach(Array(stationStore.topStations.enumerated()), id: \.offset) { index, entry in
                        let (station, playTime) = entry
                        HStack(spacing: 12) {
                            Text("#\(index + 1)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .frame(width: 24, alignment: .trailing)
                                .monospacedDigit()

                            VStack(alignment: .leading, spacing: 4) {
                                HStack {
                                    Text(station.name)
                                        .fontWeight(.medium)
                                    Spacer()
                                    Text(formatPlayTime(playTime))
                                        .font(.caption)
                                        .monospacedDigit()
                                        .foregroundStyle(.secondary)
                                }

                                // Bar visualization
                                GeometryReader { geo in
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(Color.accentColor.opacity(0.6))
                                        .frame(
                                            width: geo.size.width * CGFloat(playTime) / CGFloat(maxPlayTime),
                                            height: 6
                                        )
                                }
                                .frame(height: 6)

                                // Last played
                                if let stats = stationStore.getStats(stationId: station.id),
                                   let lastPlayed = stats.lastPlayed {
                                    Text("Last played: \(relativeDate(from: lastPlayed))")
                                        .font(.caption2)
                                        .foregroundStyle(.tertiary)
                                }
                            }
                        }
                        .padding(.vertical, 4)
                    }
                }
                .listStyle(.plain)
            }
        }
        .padding()
    }
}

private func relativeDate(from timestamp: Int64) -> String {
    let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
    let formatter = RelativeDateTimeFormatter()
    formatter.unitsStyle = .abbreviated
    return formatter.localizedString(for: date, relativeTo: Date())
}
