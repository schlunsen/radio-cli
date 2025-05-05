class RadioCli < Formula
  desc "Terminal-based internet radio player with visualizations"
  homepage "https://github.com/schlunsen/radiocli"
  url "https://github.com/schlunsen/radiocli/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"
  head "https://github.com/schlunsen/radiocli.git", branch: "master"

  depends_on "rust" => :build
  depends_on "mpv" # Required dependency for audio playback

  def install
    # Modify the app/mod.rs file to use a fixed database path
    db_path = var/"radio_cli/stations.db"
    
    cd "radio_cli" do
      # Create the var directory to store the database
      (var/"radio_cli").mkpath
      
      # Copy the database file to the var directory
      (var/"radio_cli").install "stations.db"
      
      # Patch the source code to use the correct database path
      inreplace "src/app/mod.rs", 
                "let conn = Connection::open(\"stations.db\")?;", 
                "let conn = Connection::open(\"#{db_path}\")?"
      
      # Build and install
      system "cargo", "build", "--release"
      bin.install "target/release/radio_cli"
      
      # Create a symlink with a hyphenated name (optional)
      bin.install_symlink "radio_cli" => "radio-cli"
    end
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
      
      Your station database is stored at:
        #{var}/radio_cli/stations.db
    EOS
  end
  
  # Ensure the var directory persists across upgrades
  def plist_name
    "com.schlunsen.radio-cli"
  end
end