class RadioCli < Formula
  desc "Terminal-based internet radio player with visualizations"
  homepage "https://github.com/schlunsen/radio-cli"
  url "https://github.com/schlunsen/radio-cli/archive/refs/tags/v0.02.tar.gz"
  sha256 "a579ba27eeb5f35085311083539cab035c468542000fafa27ac6f0eff27aa2a4"
  license "MIT"
  head "https://github.com/schlunsen/radio-cli.git", branch: "master"

  depends_on "rust" => :build
  depends_on "mpv" # Required dependency for audio playback

  def install
    # Find the radio_cli directory - we need to handle different structures
    # Homebrew unpacks GitHub releases to directories like "radio-cli-0.02"
    radio_cli_dir = if Dir.exist?("radio_cli")
                      "radio_cli"
                    else
                      # We're directly in the radio_cli directory
                      "."
                    end

    # Use a persistent location for the database
    db_path = var/"radio_cli/stations.db"
    
    # Create the var directory to store the database
    (var/"radio_cli").mkpath
    
    # Find and copy the stations.db file
    db_source = if File.exist?("#{radio_cli_dir}/stations.db")
                  "#{radio_cli_dir}/stations.db"
                elsif File.exist?("stations.db")
                  "stations.db"
                else
                  raise "Could not find stations.db"
                end
    
    # Copy the database file to the var directory
    (var/"radio_cli").install db_source
    
    # Find and patch the app/mod.rs file
    app_mod_path = if File.exist?("#{radio_cli_dir}/src/app/mod.rs")
                     "#{radio_cli_dir}/src/app/mod.rs"
                   elsif File.exist?("src/app/mod.rs")
                     "src/app/mod.rs"
                   else
                     raise "Could not find src/app/mod.rs"
                   end
    
    # Patch the source code to use the correct database path
    inreplace app_mod_path, 
              'let conn = Connection::open("stations.db")?;', 
              "let conn = Connection::open(\"#{db_path}\")?"
    
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
                      raise "Could not find compiled binary"
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