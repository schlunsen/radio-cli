#!/bin/bash
set -e

# Check if a version is provided
if [ $# -ne 1 ]; then
  echo "Usage: $0 VERSION"
  echo "Example: $0 0.5"
  exit 1
fi

VERSION=$1
VERSION_NUM="0.0.$VERSION"
TAG="v0.$VERSION"

echo "=== Creating release for Radio CLI $TAG (version $VERSION_NUM) ==="

# 1. Update version in Cargo.toml
echo "1. Updating version in Cargo.toml to $VERSION_NUM..."
sed -i '' "s/version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$VERSION_NUM\"/" Cargo.toml

# 2. Commit the version change
echo "2. Committing version bump..."
git add Cargo.toml
git commit -m "Bump version to $VERSION_NUM"

# 3. Create a new tag
echo "3. Creating tag $TAG..."
git tag -a $TAG -m "Release version $TAG"

# 4. Update the Homebrew formula with the new tag (placeholder SHA256)
echo "4. Updating Homebrew formula..."
sed -i '' "s|url \"https://github.com/schlunsen/radio-cli/archive/refs/tags/v[0-9]*\\.[0-9]*.tar.gz\"|url \"https://github.com/schlunsen/radio-cli/archive/refs/tags/$TAG.tar.gz\"|" Formula/radio-cli.rb
sed -i '' "s|sha256 \"[a-z0-9]*\"|sha256 \"REPLACE_AFTER_PUSHING_TAG\"|" Formula/radio-cli.rb

# 5. Commit the formula update
echo "5. Committing formula update..."
git add Formula/radio-cli.rb
git commit -m "Update formula for $TAG"

# 6. Push the commits and tag
echo "6. Pushing commits and tag to GitHub..."
git push origin main
git push origin $TAG

# 7. Wait for GitHub to process the tag
echo "7. Waiting for GitHub to process the tag..."
sleep 5

# 8. Download the tarball and calculate SHA256
echo "8. Calculating SHA256 for the new release..."
mkdir -p /tmp/radio-cli-release
curl -sL "https://github.com/schlunsen/radio-cli/archive/refs/tags/$TAG.tar.gz" -o "/tmp/radio-cli-release/$TAG.tar.gz"
SHA256=$(shasum -a 256 "/tmp/radio-cli-release/$TAG.tar.gz" | cut -d ' ' -f 1)
echo "SHA256: $SHA256"

# 9. Update the formula with the actual SHA256
echo "9. Updating formula with SHA256..."
sed -i '' "s|sha256 \"REPLACE_AFTER_PUSHING_TAG\"|sha256 \"$SHA256\"|" Formula/radio-cli.rb
git add Formula/radio-cli.rb
git commit -m "Update SHA256 for $TAG"
git push origin main

# 10. Update the tap repository
echo "10. Updating the Homebrew tap repository..."
if [ -d ~/homebrew-radio-cli ]; then
  cp Formula/radio-cli.rb ~/homebrew-radio-cli/Formula/
  cd ~/homebrew-radio-cli
  git add Formula/radio-cli.rb
  git commit -m "Update radio-cli formula to $TAG"
  git push
  echo "✓ Homebrew tap updated"
else
  echo "! Homebrew tap directory not found"
  echo "  Please run ./homebrew-setup.sh to create it"
fi

echo ""
echo "=== Release $TAG completed! ==="
echo ""
echo "Users can install with:"
echo "  brew update"
echo "  brew install schlunsen/radio-cli/radio-cli"
echo ""
echo "Or upgrade with:"
echo "  brew update"
echo "  brew upgrade schlunsen/radio-cli/radio-cli"
echo ""