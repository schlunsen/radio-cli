#!/bin/bash
set -e

echo "=== Setting up Homebrew tap for Radio CLI ==="

# 1. Remove any existing tap
echo "1. Removing any existing Radio CLI tap..."
brew untap schlunsen/radio-cli 2>/dev/null || true

# 2. Tap from the GitHub repository
echo "2. Setting up tap from GitHub repository..."
brew tap schlunsen/radio-cli https://github.com/schlunsen/homebrew-radio-cli.git

# 3. Copy the formula to a local directory for inspection
echo "3. Copying formula to ~/homebrew-radio-cli ..."
mkdir -p ~/homebrew-radio-cli/Formula
cp "$(pwd)/Formula/radio-cli.rb" ~/homebrew-radio-cli/Formula/

echo ""
echo "=== Instructions ==="
echo "1. Create a GitHub repository called 'homebrew-radio-cli'"
echo "2. Push the formula to that repository:"
echo "   cd ~/homebrew-radio-cli"
echo "   git init"
echo "   git add ."
echo "   git commit -m \"Initial commit\""
echo "   git branch -M main"
echo "   git remote add origin git@github.com:schlunsen/homebrew-radio-cli.git"
echo "   git push -u origin main"
echo ""
echo "3. Then users can install with:"
echo "   brew install schlunsen/radio-cli/radio-cli"
echo ""
echo "NOTE: Homebrew tap repositories must follow the 'homebrew-xxx' naming convention"
echo "      The tap name will be 'schlunsen/radio-cli' (without the 'homebrew-' prefix)"
echo ""