#!/bin/bash
set -e

# Check if a version is provided
if [ $# -ne 1 ]; then
  echo "Usage: $0 VERSION"
  echo "Example: $0 0.04"
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

# 6. Show next steps
echo ""
echo "=== Release $TAG prepared! ==="
echo ""
echo "Next steps:"
echo "1. Push the commits and tag:"
echo "   git push origin main"
echo "   git push origin $TAG"
echo ""
echo "2. After pushing, download the tarball and calculate SHA256:"
echo "   curl -sL https://github.com/schlunsen/radio-cli/archive/refs/tags/$TAG.tar.gz -o /tmp/$TAG.tar.gz"
echo "   shasum -a 256 /tmp/$TAG.tar.gz"
echo ""
echo "3. Update the formula with the actual SHA256 and push again"
echo ""
echo "4. Update your Homebrew tap:"
echo "   ./fix_homebrew.sh"
echo ""