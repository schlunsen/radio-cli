#!/bin/bash

# Script to update the Homebrew formula for radio-cli
# Usage: ./update_homebrew.sh v1.3.5

set -e

# Check if a version argument was provided
if [ -z "$1" ]; then
  echo "Error: You must provide a version number (e.g., v1.3.5)"
  echo "Usage: $0 v1.3.5"
  exit 1
fi

VERSION=$1
FORMULA_PATH="Formula/radio-cli.rb"
GITHUB_REPO="schlunsen/radio-cli"
RELEASE_URL="https://github.com/$GITHUB_REPO/releases/tag/$VERSION"

echo "Updating Homebrew formula for Radio CLI $VERSION..."

# Function to calculate SHA256 from a URL
calculate_sha() {
  local url=$1
  echo "Downloading $url to calculate SHA256..."
  curl -sL "$url" | shasum -a 256 | cut -d ' ' -f 1
}

# Get the source tarball URL and SHA
SOURCE_URL="https://github.com/$GITHUB_REPO/archive/refs/tags/$VERSION.tar.gz"
SOURCE_SHA=$(calculate_sha "$SOURCE_URL")

# Get the macOS Intel binary URL and SHA
MACOS_INTEL_URL="https://github.com/$GITHUB_REPO/releases/download/$VERSION/radio_cli-macos-intel.tar.gz"
MACOS_INTEL_SHA=$(calculate_sha "$MACOS_INTEL_URL")

# Get the macOS Apple Silicon binary URL and SHA
MACOS_ARM_URL="https://github.com/$GITHUB_REPO/releases/download/$VERSION/radio_cli-macos-apple-silicon.tar.gz"
MACOS_ARM_SHA=$(calculate_sha "$MACOS_ARM_URL")

echo "Source tarball SHA: $SOURCE_SHA"
echo "macOS Intel binary SHA: $MACOS_INTEL_SHA"
echo "macOS Apple Silicon binary SHA: $MACOS_ARM_SHA"

# Check if formula file exists
if [ ! -f "$FORMULA_PATH" ]; then
  echo "Error: Formula file not found at $FORMULA_PATH"
  exit 1
fi

# Update the formula file with new version and SHAs
sed -i "" \
  -e "s|url \"https://github.com/$GITHUB_REPO/archive/refs/tags/v[0-9.]*\.tar\.gz\"|url \"$SOURCE_URL\"|" \
  -e "s|sha256 \"[a-f0-9]*\"|sha256 \"$SOURCE_SHA\"|" \
  -e "s|url \"https://github.com/$GITHUB_REPO/releases/download/v[0-9.]*/radio_cli-macos-intel.tar.gz\"|url \"$MACOS_INTEL_URL\"|" \
  -e "s|url \"https://github.com/$GITHUB_REPO/releases/download/v[0-9.]*/radio_cli-macos-apple-silicon.tar.gz\"|url \"$MACOS_ARM_URL\"|" \
  "$FORMULA_PATH"

# Update all the SHA256 hashes - Intel and Apple Silicon
# Get line numbers for Intel and Apple Silicon SHAs
INTEL_SHA_LINE=$(grep -n "Hardware::CPU.intel" "$FORMULA_PATH" | head -1 | cut -d ':' -f 1)
INTEL_SHA_LINE=$((INTEL_SHA_LINE + 1))
ARM_SHA_LINE=$(grep -n "Hardware::CPU.arm" "$FORMULA_PATH" | head -1 | cut -d ':' -f 1)
ARM_SHA_LINE=$((ARM_SHA_LINE + 1))

# Update the SHA lines
sed -i "" "${INTEL_SHA_LINE}s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_INTEL_SHA\"|" "$FORMULA_PATH"
sed -i "" "${ARM_SHA_LINE}s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_ARM_SHA\"|" "$FORMULA_PATH"

echo "Formula updated successfully with version $VERSION"
echo "URLs and SHA256 hashes have been updated."
echo "Please review the changes to $FORMULA_PATH"

# Show the diff
echo -e "\nChanges made to the formula:"
git diff "$FORMULA_PATH"

echo -e "\nTo test the formula locally, run:"
echo "  brew install --build-from-source ./Formula/radio-cli.rb"
echo "To commit and publish these changes, run:"
echo "  git add $FORMULA_PATH"
echo "  git commit -m \"Update Homebrew formula to $VERSION\""
echo "  git push origin main"