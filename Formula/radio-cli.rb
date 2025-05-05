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

    # Debug: Print the directory structure
    system "ls", "-la", radio_cli_dir
    system "find", ".", "-name", "*.db"

    # Use a persistent location for the database
    db_path = var/"radio_cli/stations.db"
    
    # Create the var directory to store the database
    (var/"radio_cli").mkpath
    
    # Try to find and copy the stations.db file
    db_source = nil
    [
      "#{radio_cli_dir}/stations.db",
      "stations.db",
      "../radio_cli/stations.db"
    ].each do |path|
      if File.exist?(path)
        db_source = path
        break
      end
    end

    if db_source
      # Copy the database file to the var directory
      (var/"radio_cli").install db_source
    else
      # Create a default database if no source is found
      system "sqlite3", "#{var}/radio_cli/stations.db", <<~SQL
        CREATE TABLE stations (
          id INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          url TEXT NOT NULL,
          favorite INTEGER NOT NULL DEFAULT 0,
          description TEXT
        );
        INSERT INTO stations (name, url, description) VALUES 
          ('Groove Salad (SomaFM)', 'http://ice1.somafm.com/groovesalad-128-mp3', 'Chilled electronic and downtempo beats'),
          ('Secret Agent (SomaFM)', 'http://ice4.somafm.com/secretagent-128-mp3', 'The soundtrack for your stylish, mysterious, dangerous life'),
          ('BBC Radio 1', 'http://icecast.omroep.nl/radio1-bb-mp3', 'BBC''s flagship radio station for new music and entertainment'),
          ('FluxFM Chillhop', 'https://streams.fluxfm.de/Chillhop/mp3-320/streams.fluxfm.de/', 'High-quality Chillhop stream from FluxFM - relaxed beats at 320kbps');
      SQL
    end
    
    # Find and patch the app/mod.rs file
    app_mod_path = nil
    [
      "#{radio_cli_dir}/src/app/mod.rs",
      "src/app/mod.rs"
    ].each do |path|
      if File.exist?(path)
        app_mod_path = path
        break
      end
    end

    if app_mod_path.nil?
      # Handle the case when app/mod.rs can't be found
      ohai "Could not find src/app/mod.rs"
      system "find", ".", "-type", "f", "-name", "*.rs"
      # Try another approach - look for any file containing the database connection
      files = Utils.popen_read("grep", "-l", "Connection::open", "--include=*.rs", "-r", ".").split("\n")
      app_mod_path = files.first unless files.empty?
    end

    # Patch the source code to use the correct database path
    if app_mod_path
      inreplace app_mod_path, 
                'let conn = Connection::open("stations.db")?;', 
                "let conn = Connection::open(\"#{db_path}\")?;"
    else
      odie "Could not find any file with database connection"
    end
    
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
      
      Your station database is stored at:
        #{var}/radio_cli/stations.db
    EOS
  end
  
  # Ensure the var directory persists across upgrades
  def plist_name
    "com.schlunsen.radio-cli"
  end
end