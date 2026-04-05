import Foundation

enum RCastError: LocalizedError, Sendable {
    case networkError(String)
    case parseError(String)

    var errorDescription: String? {
        switch self {
        case .networkError(let msg): return "Network error: \(msg)"
        case .parseError(let msg): return "Parse error: \(msg)"
        }
    }
}

/// Fetches RCast stations on a background thread to avoid blocking MainActor.
func fetchRCastStations() async throws -> [RcastStation] {
    try await Task.detached {
        try await RCastFetcher.fetch()
    }.value
}

/// All work runs off the MainActor.
private enum RCastFetcher: Sendable {
    nonisolated static func fetch() async throws -> [RcastStation] {
        let urlString = "https://www.rcast.net/dir?action=search&search=icecast&sortby=1"
        guard let url = URL(string: urlString) else {
            throw RCastError.networkError("Invalid URL")
        }

        var request = URLRequest(url: url)
        request.timeoutInterval = 15
        request.setValue(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            forHTTPHeaderField: "User-Agent"
        )

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
            throw RCastError.networkError("HTTP request failed")
        }

        guard let html = String(data: data, encoding: .utf8) else {
            throw RCastError.parseError("Could not decode response as UTF-8")
        }

        return parseStationsFromHTML(html)
    }

    nonisolated private static func parseStationsFromHTML(_ html: String) -> [RcastStation] {
        var stations: [RcastStation] = []
        var seenIds = Set<String>()

        let varPattern = "var stream"
        var searchPos = html.startIndex

        while let range = html.range(of: varPattern, range: searchPos..<html.endIndex) {
            let afterVar = range.upperBound

            guard let spaceRange = html.range(of: " ", range: afterVar..<html.endIndex) else {
                searchPos = afterVar
                continue
            }

            let stationId = String(html[afterVar..<spaceRange.lowerBound])

            guard stationId.allSatisfy(\.isNumber), !stationId.isEmpty else {
                searchPos = spaceRange.upperBound
                continue
            }

            guard !seenIds.contains(stationId) else {
                searchPos = spaceRange.upperBound
                continue
            }
            seenIds.insert(stationId)

            var name = "Icecast Station \(stationId)"
            let idMarker = "id=\"currentsong_\(stationId)"

            if let idRange = html.range(of: idMarker) {
                let lookbackStart = html.index(idRange.lowerBound, offsetBy: -2000, limitedBy: html.startIndex) ?? html.startIndex
                let nameContext = String(html[lookbackStart..<idRange.lowerBound])

                if let h4Range = nameContext.range(of: "<h4", options: .backwards) {
                    let afterH4 = h4Range.upperBound
                    if let tagEndRange = nameContext.range(of: ">", range: afterH4..<nameContext.endIndex) {
                        let contentStart = tagEndRange.upperBound
                        if let closeRange = nameContext.range(of: "</h4>", range: contentStart..<nameContext.endIndex) {
                            let rawName = String(nameContext[contentStart..<closeRange.lowerBound])
                            let cleaned = cleanHTML(rawName)
                            if !cleaned.isEmpty && cleaned.count < 100 {
                                name = cleaned
                            }
                        }
                    }
                }
            }

            let streamURL = "https://stream.rcast.net/\(stationId)"
            stations.append(RcastStation(
                name: name,
                url: streamURL,
                description: "Icecast Radio Station, ID: \(stationId)",
                bitrate: nil,
                genre: nil,
                listeners: nil
            ))

            searchPos = spaceRange.upperBound
        }

        return stations
    }

    nonisolated private static func cleanHTML(_ text: String) -> String {
        let result = text
            .replacingOccurrences(of: "&amp;", with: "&")
            .replacingOccurrences(of: "&lt;", with: "<")
            .replacingOccurrences(of: "&gt;", with: ">")
            .replacingOccurrences(of: "&quot;", with: "\"")
            .replacingOccurrences(of: "&apos;", with: "'")
            .replacingOccurrences(of: "&#39;", with: "'")
            .replacingOccurrences(of: "<br>", with: " ")
            .replacingOccurrences(of: "<br/>", with: " ")
            .replacingOccurrences(of: "<br />", with: " ")

        var cleaned = ""
        var inTag = false
        for ch in result {
            if ch == "<" { inTag = true; continue }
            if ch == ">" { inTag = false; continue }
            if !inTag { cleaned.append(ch) }
        }

        let components = cleaned.split(whereSeparator: \.isWhitespace)
        return components.joined(separator: " ")
    }
}
