import AVFoundation
import MediaPlayer

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

    init() {
        setupRemoteCommands()
        // Read default volume from UserDefaults
        let saved = UserDefaults.standard.float(forKey: "defaultVolume")
        if saved > 0 {
            volume = saved
        }
    }

    func setStationStore(_ store: StationStore) {
        self.stationStore = store
    }

    // MARK: - Playback

    func play(station: Station) {
        guard let url = URL(string: station.url) else {
            errorMessage = "Invalid URL: \(station.url)"
            return
        }

        stop()

        lastURL = url
        currentStationName = station.name
        currentStationId = station.id
        currentSong = nil
        errorMessage = nil

        let asset = AVURLAsset(url: url)
        playerItem = AVPlayerItem(asset: asset)
        player = AVPlayer(playerItem: playerItem)
        player?.volume = isMuted ? 0 : volume

        // Observe timed metadata for ICY stream titles
        metadataObserver = playerItem?.observe(\.timedMetadata, options: [.new]) { [weak self] item, _ in
            self?.handleMetadata(item.timedMetadata)
        }

        // Observe status for errors
        statusObserver = playerItem?.observe(\.status, options: [.new]) { [weak self] item, _ in
            switch item.status {
            case .failed:
                self?.errorMessage = item.error?.localizedDescription ?? "Playback failed"
                self?.isPlaying = false
                self?.scheduleReconnect()
            case .readyToPlay:
                self?.errorMessage = nil
            default:
                break
            }
        }

        player?.play()
        isPlaying = true
        playStartTime = Date()

        // Flush play time every 30 seconds
        statsFlushTimer = Timer.scheduledTimer(withTimeInterval: 30.0, repeats: true) { [weak self] _ in
            self?.flushPlayTime()
        }

        updateNowPlaying()
    }

    func stop() {
        flushPlayTime()
        statsFlushTimer?.invalidate()
        statsFlushTimer = nil
        reconnectTimer?.invalidate()
        reconnectTimer = nil

        player?.pause()
        metadataObserver?.invalidate()
        metadataObserver = nil
        statusObserver?.invalidate()
        statusObserver = nil
        player = nil
        playerItem = nil

        isPlaying = false
        currentStationName = nil
        currentStationId = nil
        currentSong = nil
        playStartTime = nil
        lastURL = nil

        MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
    }

    func togglePlayStop(station: Station?) {
        if isPlaying {
            stop()
        } else if let station {
            play(station: station)
        }
    }

    func toggleMute() {
        isMuted.toggle()
        player?.volume = isMuted ? 0 : volume
        updateNowPlaying()
    }

    func setVolume(_ newVolume: Float) {
        volume = max(0, min(1, newVolume))
        if !isMuted {
            player?.volume = volume
        }
    }

    // MARK: - Metadata

    private func handleMetadata(_ metadata: [AVMetadataItem]?) {
        guard let metadata else { return }
        for item in metadata {
            if let title = item.stringValue, !title.isEmpty {
                currentSong = title
                updateNowPlaying()
                return
            }
        }
    }

    // MARK: - Media Remote Commands

    private func setupRemoteCommands() {
        let center = MPRemoteCommandCenter.shared()

        center.playCommand.addTarget { [weak self] _ in
            // Resume isn't really applicable for live streams; no-op
            return .success
        }

        center.pauseCommand.addTarget { [weak self] _ in
            self?.stop()
            return .success
        }

        center.togglePlayPauseCommand.addTarget { [weak self] _ in
            if self?.isPlaying == true {
                self?.stop()
            }
            return .success
        }
    }

    private func updateNowPlaying() {
        var info: [String: Any] = [
            MPNowPlayingInfoPropertyIsLiveStream: true,
            MPNowPlayingInfoPropertyPlaybackRate: isPlaying ? 1.0 : 0.0,
        ]

        if let name = currentStationName {
            info[MPMediaItemPropertyTitle] = name
        }
        if let song = currentSong {
            info[MPMediaItemPropertyArtist] = song
        }

        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }

    // MARK: - Reconnect

    private func scheduleReconnect() {
        reconnectTimer?.invalidate()
        reconnectTimer = Timer.scheduledTimer(withTimeInterval: 3.0, repeats: false) { [weak self] _ in
            guard let self, let url = self.lastURL else { return }
            let asset = AVURLAsset(url: url)
            let newItem = AVPlayerItem(asset: asset)
            self.playerItem = newItem
            self.player?.replaceCurrentItem(with: newItem)

            // Re-observe
            self.metadataObserver = newItem.observe(\.timedMetadata, options: [.new]) { [weak self] item, _ in
                self?.handleMetadata(item.timedMetadata)
            }
            self.statusObserver = newItem.observe(\.status, options: [.new]) { [weak self] item, _ in
                switch item.status {
                case .failed:
                    self?.errorMessage = item.error?.localizedDescription ?? "Playback failed"
                    self?.isPlaying = false
                    self?.scheduleReconnect()
                case .readyToPlay:
                    self?.errorMessage = nil
                    self?.isPlaying = true
                default:
                    break
                }
            }

            self.player?.play()
        }
    }

    // MARK: - Play Time Tracking

    private func flushPlayTime() {
        guard let startTime = playStartTime, let stationId = currentStationId else { return }
        let elapsed = Int64(Date().timeIntervalSince(startTime))
        if elapsed > 0 {
            stationStore?.updateStats(stationId: stationId, playTime: elapsed)
        }
        playStartTime = Date()
    }
}
