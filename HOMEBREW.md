# RadioCLI Homebrew Guide

RadioCLI can be easily installed using Homebrew on macOS and Linux. This guide covers both installation and how to update the formula when releasing new versions.

## Installation Steps

### 1. Tap the Repository

First, add the RadioCLI tap to your Homebrew:

```bash
brew tap schlunsen/radio-cli https://github.com/schlunsen/radiocli
```

### 2. Install RadioCLI

Then install the package:

```bash
brew install radio-cli
```

### Dependencies

The formula will automatically install the required dependencies:
- `mpv` - Media player used for streaming audio

## Building from Source

If you prefer to build from the latest source code:

```bash
brew install --HEAD radio-cli
```

## Upgrading

To upgrade to the latest version:

```bash
brew update
brew upgrade radio-cli
```

## Troubleshooting

If you encounter any issues:

```bash
# Check for problems
brew doctor

# Force a reinstall
brew reinstall radio-cli
```

### Common Issues

- **Missing MPV**: If you get errors about missing mpv player, ensure it's installed with `brew install mpv`
- **Database Permissions**: The app creates a database file in the directory where it's run. Ensure you have write permissions.

## Uninstalling

If you need to uninstall:

```bash
brew uninstall radio-cli
brew untap schlunsen/radio-cli
```

## Updating the Formula for New Releases

When releasing a new version of RadioCLI, the Homebrew formula needs to be updated with new URLs and SHA256 checksums.

### Using the Automatic Update Script

1. Make sure you have created a new release on GitHub with proper version tag (e.g., `v1.3.5`) and uploaded all required assets:
   - Source code archive (automatically created by GitHub when you create a release)
   - MacOS Intel binary: `radio_cli-macos-intel.tar.gz`
   - MacOS Apple Silicon binary: `radio_cli-macos-apple-silicon.tar.gz`

2. Run the update script with the new version tag:
   ```bash
   ./update_homebrew.sh v1.3.5
   ```

3. The script will:
   - Download the required files to calculate their SHA256 checksums
   - Update the formula file (`Formula/radio-cli.rb`) with new URLs and SHAs
   - Show you a diff of the changes made

4. Review the changes to ensure everything looks correct

5. Test the updated formula locally:
   ```bash
   brew install --build-from-source ./Formula/radio-cli.rb
   ```

6. Commit the changes and push to GitHub:
   ```bash
   git add Formula/radio-cli.rb
   git commit -m "Update Homebrew formula to v1.3.5"
   git push origin main
   ```

### Manual Formula Update

If you need to update the formula manually:

1. Update the main URL and SHA256 in `Formula/radio-cli.rb`:
   ```ruby
   url "https://github.com/schlunsen/radio-cli/archive/refs/tags/v1.3.5.tar.gz"
   sha256 "abc123..." # Replace with actual SHA256 of the source tarball
   ```

2. Update the macOS Intel URL and SHA256:
   ```ruby
   url "https://github.com/schlunsen/radio-cli/releases/download/v1.3.5/radio_cli-macos-intel.tar.gz"
   sha256 "def456..." # Replace with actual SHA256 of the Intel binary
   ```

3. Update the macOS Apple Silicon URL and SHA256:
   ```ruby
   url "https://github.com/schlunsen/radio-cli/releases/download/v1.3.5/radio_cli-macos-apple-silicon.tar.gz" 
   sha256 "ghi789..." # Replace with actual SHA256 of the Apple Silicon binary
   ```

4. To calculate SHA256 checksums:
   ```bash
   curl -sL [URL] | shasum -a 256 | cut -d ' ' -f 1
   ```

5. Test, commit and push the changes as described above