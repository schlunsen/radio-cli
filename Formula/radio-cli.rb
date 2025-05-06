class RadioCli < Formula
  desc "Terminal-based internet radio player with visualizations"
  homepage "https://github.com/schlunsen/radio-cli"
  url "https://github.com/schlunsen/radio-cli/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "939571d4f239d562400f73424d3584a684321a46daab3b8d9363dbc5c38fedfc"
  license "MIT"
  head "https://github.com/schlunsen/radio-cli.git", branch: "master"
  
  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/schlunsen/radio-cli/releases/download/v1.0.0/radio_cli-macos-intel.tar.gz"
      sha256 "32775f5eb4a395f08b9bb3e60bf96e0bb53b2501fe9baebe47e777fa04f08c08"
    end
    if Hardware::CPU.arm?
      url "https://github.com/schlunsen/radio-cli/releases/download/v1.0.0/radio_cli-macos-apple-silicon.tar.gz"
      sha256 "a43dbd8604034c5d3a5929bb172e289634f7f984dce2012938bf0efeb1ae013f"
    end
  end

  depends_on "rust" => :build unless OS.mac? # Only needed for Linux builds now
  depends_on "mpv" # Required dependency for audio playback
  depends_on "sqlite" # For database access

  def install
    if OS.mac?
      if Hardware::CPU.intel?
        # Install prebuilt binary for Intel Mac
        system "tar", "-xzf", cached_download
        bin.install "radio_cli" => "radio_cli"
      elsif Hardware::CPU.arm?
        # Install prebuilt binary for Apple Silicon (M1/M2/M3/M4)
        system "tar", "-xzf", cached_download
        bin.install "radio_cli" => "radio_cli"
      end
    else
      # Build from source for other platforms
      # Find the radio_cli directory - we need to handle different structures
      # Homebrew unpacks GitHub releases to directories like "radio-cli-0.02"
      radio_cli_dir = if Dir.exist?("radio_cli")
                        "radio_cli"
                      else
                        # We're directly in the unpacked directory
                        "."
                      end
      
      # The database is now managed by the application itself
      
      # Build and install
      cargo_dir = if File.exist?("#{radio_cli_dir}/Cargo.toml")
                    radio_cli_dir
                  else
                    "."
                  end
      
      cd cargo_dir do
        system "cargo", "build", "--release"
        
        # Find the compiled binary
        binary_path = if File.exist?("target/release/radio_cli")
                        "target/release/radio_cli"
                      else
                        odie "Could not find compiled binary"
                      end
        
        bin.install binary_path
      end
    end
    
    # Create a symlink with a hyphenated name
    bin.install_symlink "radio_cli" => "radio-cli"
  end

  test do
    system bin/"radio_cli", "--version"
  end

  def caveats
    <<~EOS
      Radio CLI uses mpv for audio playback, which has been automatically installed as a dependency.
      
      To start listening to radio stations, run either:
        radio_cli
      or:
        radio-cli
      
      Your station database will be automatically created in one of these locations (in priority order):
        1. stations.db in the current directory (if it exists)
        2. The location specified in the XDG_DATA_HOME environment variable
        3. Platform-specific data directory:
           macOS:   ~/Library/Application Support/radio_cli/stations.db
           Linux:   ~/.local/share/radio_cli/stations.db
           Windows: %APPDATA%/radio_cli/stations.db
    EOS
  end
  
  # Database persistence is now handled by the application
end