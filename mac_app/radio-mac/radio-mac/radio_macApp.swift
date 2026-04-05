import SwiftUI
import Combine

@main
struct RadioMacApp: App {
    @State private var stationStore = StationStore()
    @State private var audioManager = AudioManager()

    var body: some Scene {
        MenuBarExtra {
            PopoverView()
                .environment(stationStore)
                .environment(audioManager)
                .onAppear {
                    audioManager.setStationStore(stationStore)
                }
        } label: {
            MenuBarLabel(audioManager: audioManager)
        }
        .menuBarExtraStyle(.window)

        Window("Station Manager", id: "manager") {
            TabView {
                StationManagerView()
                    .tabItem { Label("Stations", systemImage: "radio") }
                RCastView()
                    .tabItem { Label("Discover", systemImage: "globe") }
                StatsView()
                    .tabItem { Label("Statistics", systemImage: "chart.bar") }
            }
            .frame(minWidth: 700, minHeight: 500)
            .environment(stationStore)
            .environment(audioManager)
        }

        Settings {
            SettingsView()
        }
    }
}

/// Separate view for the menu bar label to isolate state changes from the scene body.
struct MenuBarLabel: View {
    let audioManager: AudioManager
    @State private var iconPhase = 0
    @State private var iconTimer: Timer?

    var body: some View {
        Image(systemName: iconName)
            .onChange(of: audioManager.isPlaying) { _, isPlaying in
                if isPlaying {
                    startAnimation()
                } else {
                    stopAnimation()
                }
            }
    }

    private var iconName: String {
        if audioManager.isMuted && audioManager.isPlaying {
            return "speaker.slash"
        } else if audioManager.isPlaying {
            let icons = ["speaker.wave.1", "speaker.wave.2", "speaker.wave.3"]
            return icons[iconPhase % icons.count]
        }
        return "radio"
    }

    private func startAnimation() {
        guard iconTimer == nil else { return }
        iconTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            iconPhase += 1
        }
    }

    private func stopAnimation() {
        iconTimer?.invalidate()
        iconTimer = nil
        iconPhase = 0
    }
}
