#!/bin/bash
set -e

echo "=== Updating Radio CLI formula in Homebrew tap ==="

# Check if homebrew-radio-cli directory exists
if [ ! -d ~/homebrew-radio-cli ]; then
  echo "Error: ~/homebrew-radio-cli directory not found."
  echo "Please run ./homebrew-setup.sh first."
  exit 1
fi

# 1. Copy the formula to the tap directory
echo "1. Copying formula to tap directory..."
cp "$(pwd)/Formula/radio-cli.rb" ~/homebrew-radio-cli/Formula/

# 2. Commit and push changes
echo "2. Committing and pushing changes..."
cd ~/homebrew-radio-cli
git add Formula/radio-cli.rb
git commit -m "Update radio-cli formula to $(grep -o 'url ".*"' Formula/radio-cli.rb | grep -o 'v[0-9.]*')"
git push

echo ""
echo "=== Formula updated! ==="
echo "Users can update with:"
echo "  brew update"
echo "  brew upgrade radio-cli"
echo ""