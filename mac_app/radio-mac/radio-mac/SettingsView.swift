import SwiftUI
import ServiceManagement

struct SettingsView: View {
    @AppStorage("defaultVolume") private var defaultVolume: Double = 0.5
    @State private var launchAtLogin = false

    private let dbPath: String = {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/radio_cli/stations.db")
            .path
    }()

    var body: some View {
        Form {
            Section("General") {
                Toggle("Launch at Login", isOn: $launchAtLogin)
                    .onChange(of: launchAtLogin) { _, newValue in
                        do {
                            if newValue {
                                try SMAppService.mainApp.register()
                            } else {
                                try SMAppService.mainApp.unregister()
                            }
                        } catch {
                            print("Failed to update login item: \(error)")
                            launchAtLogin = !newValue
                        }
                    }
            }

            Section("Audio") {
                HStack {
                    Text("Default Volume")
                    Slider(value: $defaultVolume, in: 0...1)
                    Text("\(Int(defaultVolume * 100))%")
                        .monospacedDigit()
                        .frame(width: 40)
                }
            }

            Section("Data") {
                LabeledContent("Database Path") {
                    Text(dbPath)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 400)
        .onAppear {
            launchAtLogin = (SMAppService.mainApp.status == .enabled)
        }
    }
}
