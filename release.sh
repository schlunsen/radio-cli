#!/bin/bash
set -e

# Release script for Radio CLI
# This script handles the entire release process including GitHub Actions binary builds

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to show colored output
echo_color() {
  local color=$1
  local message=$2
  echo -e "${color}${message}${NC}"
}

# Check if a version is provided
if [ $# -ne 1 ]; then
  echo_color $RED "Usage: $0 VERSION"
  echo_color $YELLOW "Examples:"
  echo_color $YELLOW "  $0 0.8      # Creates version 0.8.0"
  echo_color $YELLOW "  $0 0.8.1    # Creates version 0.8.1"
  echo_color $YELLOW "  $0 1.0.0    # Creates version 1.0.0"
  exit 1
fi

VERSION=$1
# Check if version already has dots
if [[ $VERSION == *"."* ]]; then
  # User provided a full version like 0.8.0
  VERSION_NUM="$VERSION"
  TAG="v$VERSION"
else
  # User provided a short version like 0.8
  VERSION_NUM="0.$VERSION.0"
  TAG="v0.$VERSION"
fi

echo_color $GREEN "=== Creating release for Radio CLI $TAG (version $VERSION_NUM) ==="

# 1. Update version in Cargo.toml - only package version, not dependencies
echo_color $YELLOW "1. Updating package version in Cargo.toml to $VERSION_NUM..."
# This will match only the first occurrence of version in [package] section
sed -i '' "0,/version = \"[0-9.]*\"/s//version = \"$VERSION_NUM\"/" Cargo.toml

# 1b. Update only the radio_cli package version in Cargo.lock
echo_color $YELLOW "1b. Updating only radio_cli version in Cargo.lock to $VERSION_NUM..."
if [ -f "Cargo.lock" ]; then
  # Update only the version of the main package without changing dependency versions
  sed -i '' "/name = \"radio_cli\"/,/version = \"[0-9.]*\"/s/version = \"[0-9.]*\"/version = \"$VERSION_NUM\"/" Cargo.lock
else
  echo_color $RED "Warning: Cargo.lock not found!"
fi

# 2. Commit the version change
echo_color $YELLOW "2. Committing version bump..."
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $VERSION_NUM"

# 3. Create a new tag
echo_color $YELLOW "3. Creating tag $TAG..."
git tag -a $TAG -m "Release version $TAG"

# 4. Update the Homebrew formula with the new tag (placeholder SHA256)
echo_color $YELLOW "4. Updating Homebrew formula..."
sed -i '' "s|url \"https://github.com/schlunsen/radio-cli/archive/refs/tags/v[0-9.]*\.tar\.gz\"|url \"https://github.com/schlunsen/radio-cli/archive/refs/tags/$TAG.tar.gz\"|" Formula/radio-cli.rb
sed -i '' "s|sha256 \"[a-z0-9]*\"|sha256 \"REPLACE_AFTER_PUSHING_TAG\"|" Formula/radio-cli.rb

# Update the macOS Intel binary URL and set placeholder SHA256
sed -i '' "s|url \"https://github.com/schlunsen/radio-cli/releases/download/v[0-9.]*/radio_cli-macos-intel\.tar\.gz\"|url \"https://github.com/schlunsen/radio-cli/releases/download/$TAG/radio_cli-macos-intel.tar.gz\"|" Formula/radio-cli.rb
sed -i '' "/macos-intel-binary/,/end/ s|sha256 \"[a-f0-9]*\"|sha256 \"REPLACE_AFTER_BUILDING\"|" Formula/radio-cli.rb

# Update the macOS Apple Silicon binary URL and set placeholder SHA256
sed -i '' "s|url \"https://github.com/schlunsen/radio-cli/releases/download/v[0-9.]*/radio_cli-macos-apple-silicon\.tar\.gz\"|url \"https://github.com/schlunsen/radio-cli/releases/download/$TAG/radio_cli-macos-apple-silicon.tar.gz\"|" Formula/radio-cli.rb
sed -i '' "/macos-arm-binary/,/end/ s|sha256 \"[a-f0-9]*\"|sha256 \"REPLACE_AFTER_BUILDING\"|" Formula/radio-cli.rb

# 5. Commit the formula update
echo_color $YELLOW "5. Committing formula update..."
git add Formula/radio-cli.rb
git commit -m "Update formula for $TAG"

# 6. Push the commits and tag
echo_color $YELLOW "6. Pushing commits and tag to GitHub..."
git push origin main
git push origin $TAG

# 7. Wait for GitHub to process the tag
echo_color $YELLOW "7. Waiting for GitHub to process the tag..."
sleep 5

# 8. Download the tarball and calculate SHA256
echo_color $YELLOW "8. Calculating SHA256 for the source tarball..."
mkdir -p /tmp/radio-cli-release
curl -sL "https://github.com/schlunsen/radio-cli/archive/refs/tags/$TAG.tar.gz" -o "/tmp/radio-cli-release/$TAG.tar.gz"
SOURCE_SHA256=$(shasum -a 256 "/tmp/radio-cli-release/$TAG.tar.gz" | cut -d ' ' -f 1)
echo_color $GREEN "Source SHA256: $SOURCE_SHA256"

# 9. Update the formula with the actual source SHA256
echo_color $YELLOW "9. Updating formula with source SHA256..."
sed -i '' "s|sha256 \"REPLACE_AFTER_PUSHING_TAG\"|sha256 \"$SOURCE_SHA256\"|" Formula/radio-cli.rb
git add Formula/radio-cli.rb
git commit -m "Update source SHA256 for $TAG"
git push origin main

# 10. Wait for GitHub Actions to build the binaries
echo_color $YELLOW "10. Waiting for GitHub Actions to build the binaries..."
echo_color $YELLOW "    This process can take several minutes. Please check the Actions tab on GitHub:"
echo_color $GREEN "    https://github.com/schlunsen/radio-cli/actions"
echo_color $YELLOW "    You need to wait until the workflow completes and binaries are available."
read -p "Press Enter once the GitHub Actions workflow has completed..." 

# 11. Download the macOS binaries and calculate SHA256
echo_color $YELLOW "11. Calculating SHA256 for the macOS Intel binary..."
curl -sL "https://github.com/schlunsen/radio-cli/releases/download/$TAG/radio_cli-macos-intel.tar.gz" -o "/tmp/radio-cli-release/radio_cli-macos-intel.tar.gz"
MACOS_INTEL_SHA256=$(shasum -a 256 "/tmp/radio-cli-release/radio_cli-macos-intel.tar.gz" | cut -d ' ' -f 1)
echo_color $GREEN "macOS Intel binary SHA256: $MACOS_INTEL_SHA256"

echo_color $YELLOW "11b. Calculating SHA256 for the macOS Apple Silicon binary..."
echo_color $YELLOW "Downloading from: https://github.com/schlunsen/radio-cli/releases/download/$TAG/radio_cli-macos-apple-silicon.tar.gz"
curl -sL "https://github.com/schlunsen/radio-cli/releases/download/$TAG/radio_cli-macos-apple-silicon.tar.gz" -o "/tmp/radio-cli-release/radio_cli-macos-apple-silicon.tar.gz" --retry 3
MACSHA="/tmp/radio-cli-release/radio_cli-macos-apple-silicon.tar.gz"
echo_color $GREEN "Download complete, calculating checksum for file: $MACSHA"
if [ -f "$MACSHA" ]; then
  MACOS_ARM_SHA256=$(shasum -a 256 "$MACSHA" | cut -d ' ' -f 1)
  echo_color $GREEN "macOS Apple Silicon binary SHA256: $MACOS_ARM_SHA256"
else
  echo_color $RED "ERROR: Apple Silicon binary download failed!"
  exit 1
fi

# 12. Update the formula with the macOS binary SHA256 values
echo_color $YELLOW "12. Updating formula with macOS binary SHA256 values..."
# Update Intel binary SHA256
sed -i '' "/Hardware::CPU.intel?/,/end/ s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_INTEL_SHA256\"|" Formula/radio-cli.rb
# Update Apple Silicon binary SHA256
sed -i '' "/Hardware::CPU.arm?/,/end/ s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_ARM_SHA256\"|" Formula/radio-cli.rb
git add Formula/radio-cli.rb
git commit -m "Update macOS binary SHA256 values for $TAG"
git push origin main

# 13. Update the tap repository
echo_color $YELLOW "13. Updating the Homebrew tap repository..."
if [ -d ~/homebrew-radio-cli ]; then
  cp Formula/radio-cli.rb ~/homebrew-radio-cli/Formula/
  cd ~/homebrew-radio-cli
  git add Formula/radio-cli.rb
  git commit -m "Update radio-cli formula to $TAG"
  git push
  echo_color $GREEN "✓ Homebrew tap updated"
else
  echo_color $RED "! Homebrew tap directory not found"
  echo_color $YELLOW "  Please run ./setup_homebrew_tap.sh to create it"
fi

echo ""
echo_color $GREEN "=== Release $TAG completed! ==="
echo ""
echo_color $GREEN "Users can install with:"
echo_color $YELLOW "  brew update"
echo_color $YELLOW "  brew install schlunsen/radio-cli/radio-cli"
echo ""
echo_color $GREEN "Or upgrade with:"
echo_color $YELLOW "  brew update"
echo_color $YELLOW "  brew upgrade schlunsen/radio-cli/radio-cli"
echo ""
echo_color $GREEN "macOS Intel users will get the prebuilt binary automatically!"
echo ""