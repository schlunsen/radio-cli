class RadioCli < Formula
  desc "Terminal-based internet radio player with visualizations"
  homepage "https://github.com/schlunsen/radio-cli"
  url "https://github.com/schlunsen/radio-cli/archive/refs/tags/v0.5.tar.gz"
  sha256 "99f4122bd30941a33f00becb64789208b6870098a329865863be6094529de4f7"
  license "MIT"
  head "https://github.com/schlunsen/radio-cli.git", branch: "master"

  depends_on "rust" => :build
  depends_on "mpv" # Required dependency for audio playback
  depends_on "sqlite" # For database access

  def install
    # Find the radio_cli directory - we need to handle different structures
    # Homebrew unpacks GitHub releases to directories like "radio-cli-0.02"
    radio_cli_dir = if Dir.exist?("radio_cli")
                      "radio_cli"
                    else
                      # We're directly in the unpacked directory
                      "."
                    end

    # The database is now managed by the application itself
    
    # Database path is now managed by the application itself
    
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
      
      Your station database will be automatically created in a platform-specific location:
        macOS:   ~/Library/Application Support/radio_cli/stations.db
        Linux:   ~/.local/share/radio_cli/stations.db
        Windows: %APPDATA%/radio_cli/stations.db
    EOS
  end
  
  # Database persistence is now handled by the application
end