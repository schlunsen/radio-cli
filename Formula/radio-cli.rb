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
    # The main code is in the radio_cli directory
    cd "radio_cli" do
      system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    end

    # Install shell completions if you add them later
    # bash_completion.install "completions/radio_cli.bash"
    # zsh_completion.install "completions/radio_cli.zsh"
    # fish_completion.install "completions/radio_cli.fish"
  end

  test do
    # Add a test to make sure the binary runs at the basic level
    assert_match "RadioCLI", shell_output("#{bin}/radio_cli --version")
  end

  # Show a message after installation to inform users about MPV
  def caveats
    <<~EOS
      Radio CLI uses mpv for audio playback, which has been automatically installed as a dependency.
      
      To start listening to radio stations, simply run:
        radio_cli
    EOS
  end
end