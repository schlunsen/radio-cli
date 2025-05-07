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

# Remove potential 'v' prefix if present
VERSION=${1#v}
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
SOURCE_URL="https://github.com/$GITHUB_REPO/archive/refs/tags/v$VERSION.tar.gz"
SOURCE_SHA=$(calculate_sha "$SOURCE_URL")

# Get the macOS Intel binary URL and SHA
MACOS_INTEL_URL="https://github.com/$GITHUB_REPO/releases/download/v$VERSION/radio_cli-macos-intel.tar.gz"
MACOS_INTEL_SHA=$(calculate_sha "$MACOS_INTEL_URL")

# Get the macOS Apple Silicon binary URL and SHA
MACOS_ARM_URL="https://github.com/$GITHUB_REPO/releases/download/v$VERSION/radio_cli-macos-apple-silicon.tar.gz"
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
echo "Updating formula file with new version and SHAs..."

# Update the main formula URL
sed -i "" -e "s|url \"https://github.com/$GITHUB_REPO/archive/refs/tags/v[0-9.]*\.tar\.gz\"|url \"$SOURCE_URL\"|" "$FORMULA_PATH"

# Update the Intel URL 
sed -i "" -e "s|url \"https://github.com/$GITHUB_REPO/releases/download/v[0-9.]*/radio_cli-macos-intel.tar.gz\"|url \"$MACOS_INTEL_URL\"|" "$FORMULA_PATH"

# Update the Apple Silicon URL
sed -i "" -e "s|url \"https://github.com/$GITHUB_REPO/releases/download/v[0-9.]*/radio_cli-macos-apple-silicon.tar.gz\"|url \"$MACOS_ARM_URL\"|" "$FORMULA_PATH"

# Update all the SHA256 hashes
echo "Updating SHA256 hashes..."

# Update main SHA first (this is the source code tarball)
awk -v sha="$SOURCE_SHA" '
  /^  desc / { print; getline; 
    if ($1 == "homepage") { print; getline; 
      if ($1 == "url") { print; getline; 
        if ($1 == "sha256") { print "  sha256 \"" sha "\""; next; }
      }
    }
  }
  { print }
' "$FORMULA_PATH" > "$FORMULA_PATH.tmp" && mv "$FORMULA_PATH.tmp" "$FORMULA_PATH"

# Update the Intel SHA
INTEL_SECTION=$(grep -n -A 3 "if Hardware::CPU.intel" "$FORMULA_PATH" | head -3)
INTEL_SHA_LINE=$(echo "$INTEL_SECTION" | grep -n "sha256" | cut -d ':' -f 1)
if [ -n "$INTEL_SHA_LINE" ]; then
  INTEL_SHA_LINE=$(echo "$INTEL_SECTION" | grep -n "sha256" | cut -d ':' -f 3 | cut -d '-' -f 1)
  sed -i "" "${INTEL_SHA_LINE}s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_INTEL_SHA\"|" "$FORMULA_PATH"
  echo "Updated Intel SHA256"
fi

# Update the ARM SHA
ARM_SECTION=$(grep -n -A 3 "if Hardware::CPU.arm" "$FORMULA_PATH" | head -3)
ARM_SHA_LINE=$(echo "$ARM_SECTION" | grep -n "sha256" | cut -d ':' -f 1)
if [ -n "$ARM_SHA_LINE" ]; then
  ARM_SHA_LINE=$(echo "$ARM_SECTION" | grep -n "sha256" | cut -d ':' -f 3 | cut -d '-' -f 1)
  sed -i "" "${ARM_SHA_LINE}s|sha256 \"[a-f0-9]*\"|sha256 \"$MACOS_ARM_SHA\"|" "$FORMULA_PATH"
  echo "Updated ARM SHA256"
fi

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