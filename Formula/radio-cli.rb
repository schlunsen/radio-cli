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
    # Build and install the binary using a more direct approach
    cd "radio_cli" do
      # First, build the release version
      system "cargo", "build", "--release"
      
      # Then, install the binary to the bin directory
      bin.install "target/release/radio_cli"
      
      # Install the stations database to the share directory
      share.install "stations.db"
      
      # Create a wrapper script to run radio_cli with the correct database path
      wrapper_script = <<~EOS
        #!/bin/bash
        DB_PATH="#{share}/stations.db"
        
        # Check if local stations.db exists in the current directory
        if [ -f "./stations.db" ]; then
          # Use the local database
          exec "#{bin}/radio_cli" "$@" 
        else
          # Copy the shared database to the user's home directory if it doesn't exist
          USER_DB="$HOME/.radio_cli/stations.db"
          mkdir -p "$HOME/.radio_cli"
          
          if [ ! -f "$USER_DB" ]; then
            cp "#{share}/stations.db" "$USER_DB"
          fi
          
          # Change to the home directory and run the program
          cd "$HOME/.radio_cli"
          exec "#{bin}/radio_cli" "$@"
        fi
      EOS
      
      # Write the wrapper script to bin directory
      (bin/"radio-cli").write wrapper_script
      chmod 0755, bin/"radio-cli"
    end

    # Install shell completions if you add them later
    # bash_completion.install "radio_cli/completions/radio_cli.bash"
    # zsh_completion.install "radio_cli/completions/radio_cli.zsh"
    # fish_completion.install "radio_cli/completions/radio_cli.fish"
  end

  test do
    # Test that the binary runs and outputs the expected version
    assert_match "RadioCLI", shell_output("#{bin}/radio_cli --version")
  end

  # Show a message after installation to inform users about MPV
  def caveats
    <<~EOS
      Radio CLI uses mpv for audio playback, which has been automatically installed as a dependency.
      
      To start listening to radio stations, simply run:
        radio-cli
      
      The station database is stored in $HOME/.radio_cli/stations.db
      Any stations you add will be saved there.
    EOS
  end
end