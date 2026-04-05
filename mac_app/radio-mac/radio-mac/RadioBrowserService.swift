import Foundation

/// Station from the Radio Browser API
struct RadioBrowserStation: Identifiable, Sendable {
    let id = UUID()
    let name: String
    let url: String
    let codec: String?
    let bitrate: Int?
    let tags: String?
    let country: String?
    let clickcount: Int?
    let votes: Int?

    var description: String {
        var parts: [String] = []
        if let tags, !tags.isEmpty { parts.append(tags) }
        if let country, !country.isEmpty { parts.append(country) }
        if let codec, !codec.isEmpty { parts.append(codec) }
        if let br = bitrate, br > 0 { parts.append("\(br)kbps") }
        return parts.joined(separator: " | ")
    }
}

struct RadioBrowserFilter: Sendable {
    var limit: Int = 100
    var order: String = "clickcount"
    var reverse: Bool = true
    var hidebroken: Bool = true
    var tag: String?
    var country: String?
    var name: String?
}

func fetchRadioBrowserStations(filter: RadioBrowserFilter = RadioBrowserFilter()) async throws -> [RadioBrowserStation] {
    try await Task.detached {
        try await RadioBrowserFetcher.fetch(filter: filter)
    }.value
}

private enum RadioBrowserFetcher: Sendable {
    private struct ApiStation: Decodable, Sendable {
        let name: String
        let url_resolved: String
        let codec: String?
        let bitrate: Int?
        let tags: String?
        let country: String?
        let clickcount: Int?
        let votes: Int?
    }

    nonisolated static func fetch(filter: RadioBrowserFilter) async throws -> [RadioBrowserStation] {
        var components = URLComponents(string: "https://de1.api.radio-browser.info/json/stations/search")!
        var queryItems = [
            URLQueryItem(name: "limit", value: "\(filter.limit)"),
            URLQueryItem(name: "order", value: filter.order),
            URLQueryItem(name: "reverse", value: "\(filter.reverse)"),
            URLQueryItem(name: "hidebroken", value: "\(filter.hidebroken)"),
        ]
        if let tag = filter.tag, !tag.isEmpty {
            queryItems.append(URLQueryItem(name: "tag", value: tag))
        }
        if let country = filter.country, !country.isEmpty {
            queryItems.append(URLQueryItem(name: "country", value: country))
        }
        if let name = filter.name, !name.isEmpty {
            queryItems.append(URLQueryItem(name: "name", value: name))
        }
        components.queryItems = queryItems

        guard let url = components.url else {
            throw RCastError.networkError("Invalid URL")
        }

        var request = URLRequest(url: url)
        request.timeoutInterval = 15
        request.setValue("RadioMac/1.0 (github.com/schlunsen/radio-cli)", forHTTPHeaderField: "User-Agent")

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
            throw RCastError.networkError("HTTP request failed")
        }

        let decoder = JSONDecoder()
        let apiStations = try decoder.decode([ApiStation].self, from: data)

        return apiStations.map { s in
            RadioBrowserStation(
                name: s.name,
                url: s.url_resolved,
                codec: s.codec,
                bitrate: s.bitrate,
                tags: s.tags,
                country: s.country,
                clickcount: s.clickcount,
                votes: s.votes
            )
        }
    }
}
