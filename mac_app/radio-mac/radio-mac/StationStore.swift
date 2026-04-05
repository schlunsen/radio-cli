import Foundation
import SQLite3

@Observable
final class StationStore {
    var stations: [Station] = []
    var topStations: [(Station, Int64)] = []

    private var db: OpaquePointer?
    private var pollTimer: Timer?
    private let dbPath: String

    init() {
        // Shared DB path with CLI: ~/.config/radio_cli/stations.db
        let configDir = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/radio_cli")
        self.dbPath = configDir.appendingPathComponent("stations.db").path

        // Ensure directory exists
        try? FileManager.default.createDirectory(
            at: configDir,
            withIntermediateDirectories: true
        )

        openDatabase()
        initSchema()
        loadStations()

        // Poll for external changes every 5 seconds
        pollTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            self?.loadStations()
        }
    }

    deinit {
        pollTimer?.invalidate()
        if let db {
            sqlite3_close(db)
        }
    }

    // MARK: - Database Setup

    private func openDatabase() {
        if sqlite3_open(dbPath, &db) != SQLITE_OK {
            print("Failed to open database at \(dbPath)")
            return
        }
        // Enable WAL mode for concurrent access with CLI
        exec("PRAGMA journal_mode=WAL")
    }

    private func initSchema() {
        exec("""
            CREATE TABLE IF NOT EXISTS stations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                favorite INTEGER NOT NULL DEFAULT 0,
                description TEXT
            )
        """)
        exec("""
            CREATE TABLE IF NOT EXISTS station_stats (
                station_id INTEGER PRIMARY KEY,
                total_play_time INTEGER NOT NULL DEFAULT 0,
                last_played INTEGER,
                FOREIGN KEY (station_id) REFERENCES stations(id) ON DELETE CASCADE
            )
        """)

        // Seed default stations if empty
        let count = queryInt("SELECT COUNT(*) FROM stations")
        if count == 0 {
            let defaults: [(String, String, String)] = [
                ("Groove Salad (SomaFM)", "http://ice1.somafm.com/groovesalad-128-mp3",
                 "Chilled electronic and downtempo beats"),
                ("Secret Agent (SomaFM)", "http://ice4.somafm.com/secretagent-128-mp3",
                 "The soundtrack for your stylish, mysterious, dangerous life"),
                ("BBC Radio 1", "http://icecast.omroep.nl/radio1-bb-mp3",
                 "BBC's flagship radio station for new music and entertainment"),
                ("FluxFM Chillhop", "https://streams.fluxfm.de/Chillhop/mp3-320/streams.fluxfm.de/",
                 "High-quality Chillhop stream from FluxFM"),
            ]
            for (name, url, desc) in defaults {
                addStation(name: name, url: url, description: desc)
            }
        }
    }

    // MARK: - CRUD Operations

    func loadStations() {
        guard let db else { return }

        var stmt: OpaquePointer?
        let sql = "SELECT id, name, url, favorite, description FROM stations ORDER BY name"

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(stmt) }

        var result: [Station] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            let id = Int(sqlite3_column_int(stmt, 0))
            let name = String(cString: sqlite3_column_text(stmt, 1))
            let url = String(cString: sqlite3_column_text(stmt, 2))
            let favorite = sqlite3_column_int(stmt, 3) != 0
            let description: String? = sqlite3_column_text(stmt, 4).map { String(cString: $0) }

            result.append(Station(
                id: id, name: name, url: url,
                favorite: favorite, description: description
            ))
        }

        stations = result
        loadTopStations()
    }

    @discardableResult
    func addStation(name: String, url: String, description: String?) -> Int {
        guard let db else { return -1 }

        var stmt: OpaquePointer?
        let sql = "INSERT INTO stations (name, url, description) VALUES (?, ?, ?)"

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return -1 }
        defer { sqlite3_finalize(stmt) }

        sqlite3_bind_text(stmt, 1, (name as NSString).utf8String, -1, nil)
        sqlite3_bind_text(stmt, 2, (url as NSString).utf8String, -1, nil)
        if let description {
            sqlite3_bind_text(stmt, 3, (description as NSString).utf8String, -1, nil)
        } else {
            sqlite3_bind_null(stmt, 3)
        }

        sqlite3_step(stmt)
        let newId = Int(sqlite3_last_insert_rowid(db))
        loadStations()
        return newId
    }

    func updateStation(id: Int, name: String, url: String, description: String?) {
        guard let db else { return }

        var stmt: OpaquePointer?
        let sql = "UPDATE stations SET name = ?, url = ?, description = ? WHERE id = ?"

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(stmt) }

        sqlite3_bind_text(stmt, 1, (name as NSString).utf8String, -1, nil)
        sqlite3_bind_text(stmt, 2, (url as NSString).utf8String, -1, nil)
        if let description {
            sqlite3_bind_text(stmt, 3, (description as NSString).utf8String, -1, nil)
        } else {
            sqlite3_bind_null(stmt, 3)
        }
        sqlite3_bind_int(stmt, 4, Int32(id))

        sqlite3_step(stmt)
        loadStations()
    }

    func deleteStation(id: Int) {
        exec("DELETE FROM station_stats WHERE station_id = \(id)")
        exec("DELETE FROM stations WHERE id = \(id)")
        loadStations()
    }

    func toggleFavorite(id: Int, favorite: Bool) {
        exec("UPDATE stations SET favorite = \(favorite ? 1 : 0) WHERE id = \(id)")
        loadStations()
    }

    // MARK: - Stats

    func updateStats(stationId: Int, playTime: Int64) {
        guard let db else { return }
        let now = Int64(Date().timeIntervalSince1970)

        var stmt: OpaquePointer?
        let updateSQL = "UPDATE station_stats SET total_play_time = total_play_time + ?, last_played = ? WHERE station_id = ?"
        guard sqlite3_prepare_v2(db, updateSQL, -1, &stmt, nil) == SQLITE_OK else { return }
        sqlite3_bind_int64(stmt, 1, playTime)
        sqlite3_bind_int64(stmt, 2, now)
        sqlite3_bind_int(stmt, 3, Int32(stationId))
        sqlite3_step(stmt)
        let changes = sqlite3_changes(db)
        sqlite3_finalize(stmt)

        if changes == 0 {
            var insertStmt: OpaquePointer?
            let insertSQL = "INSERT INTO station_stats (station_id, total_play_time, last_played) VALUES (?, ?, ?)"
            guard sqlite3_prepare_v2(db, insertSQL, -1, &insertStmt, nil) == SQLITE_OK else { return }
            sqlite3_bind_int(insertStmt, 1, Int32(stationId))
            sqlite3_bind_int64(insertStmt, 2, playTime)
            sqlite3_bind_int64(insertStmt, 3, now)
            sqlite3_step(insertStmt)
            sqlite3_finalize(insertStmt)
        }
    }

    func getStats(stationId: Int) -> StationStats? {
        guard let db else { return nil }

        var stmt: OpaquePointer?
        let sql = "SELECT station_id, total_play_time, last_played FROM station_stats WHERE station_id = ?"

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return nil }
        defer { sqlite3_finalize(stmt) }

        sqlite3_bind_int(stmt, 1, Int32(stationId))
        guard sqlite3_step(stmt) == SQLITE_ROW else { return nil }

        let lastPlayed: Int64? = sqlite3_column_type(stmt, 2) != SQLITE_NULL
            ? sqlite3_column_int64(stmt, 2)
            : nil

        return StationStats(
            stationId: Int(sqlite3_column_int(stmt, 0)),
            totalPlayTime: sqlite3_column_int64(stmt, 1),
            lastPlayed: lastPlayed
        )
    }

    private func loadTopStations() {
        guard let db else { return }

        var stmt: OpaquePointer?
        let sql = """
            SELECT s.id, s.name, s.url, s.favorite, s.description, st.total_play_time
            FROM stations s
            JOIN station_stats st ON s.id = st.station_id
            ORDER BY st.total_play_time DESC
            LIMIT 10
        """

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(stmt) }

        var result: [(Station, Int64)] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            let station = Station(
                id: Int(sqlite3_column_int(stmt, 0)),
                name: String(cString: sqlite3_column_text(stmt, 1)),
                url: String(cString: sqlite3_column_text(stmt, 2)),
                favorite: sqlite3_column_int(stmt, 3) != 0,
                description: sqlite3_column_text(stmt, 4).map { String(cString: $0) }
            )
            let playTime = sqlite3_column_int64(stmt, 5)
            result.append((station, playTime))
        }

        topStations = result
    }

    func findStationByURL(_ url: String) -> Int? {
        guard let db else { return nil }

        var stmt: OpaquePointer?
        let sql = "SELECT id FROM stations WHERE url = ?"

        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return nil }
        defer { sqlite3_finalize(stmt) }

        sqlite3_bind_text(stmt, 1, (url as NSString).utf8String, -1, nil)
        guard sqlite3_step(stmt) == SQLITE_ROW else { return nil }
        return Int(sqlite3_column_int(stmt, 0))
    }

    // MARK: - Helpers

    private func exec(_ sql: String) {
        guard let db else { return }
        sqlite3_exec(db, sql, nil, nil, nil)
    }

    private func queryInt(_ sql: String) -> Int {
        guard let db else { return 0 }

        var stmt: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return 0 }
        defer { sqlite3_finalize(stmt) }

        guard sqlite3_step(stmt) == SQLITE_ROW else { return 0 }
        return Int(sqlite3_column_int(stmt, 0))
    }

    /// Sorted stations: favorites first, then alphabetical
    var sortedStations: [Station] {
        stations.sorted { a, b in
            if a.favorite != b.favorite { return a.favorite }
            return a.name.localizedCaseInsensitiveCompare(b.name) == .orderedAscending
        }
    }
}
