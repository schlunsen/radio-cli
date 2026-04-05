import Foundation

struct Station: Identifiable, Hashable {
    let id: Int
    var name: String
    var url: String
    var favorite: Bool
    var description: String?

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: Station, rhs: Station) -> Bool {
        lhs.id == rhs.id
    }
}

struct StationStats {
    let stationId: Int
    var totalPlayTime: Int64  // seconds
    var lastPlayed: Int64?    // unix timestamp
}

struct RcastStation: Identifiable {
    let id = UUID()
    let name: String
    let url: String
    let description: String?
    let bitrate: String?
    let genre: String?
    let listeners: Int?
}

// MARK: - Helpers

func formatPlayTime(_ seconds: Int64) -> String {
    if seconds < 60 {
        return "\(seconds)s"
    } else if seconds < 3600 {
        return "\(seconds / 60)m \(seconds % 60)s"
    } else {
        return "\(seconds / 3600)h \((seconds % 3600) / 60)m"
    }
}
