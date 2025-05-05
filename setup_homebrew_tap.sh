#!/bin/bash
set -e

echo "=== Setting up Homebrew Tap Repository for Radio CLI ==="

# Define variables
GITHUB_USER="schlunsen"
CODE_REPO="radio-cli"
TAP_REPO="homebrew-radio-cli"
LOCAL_TAP_DIR="$HOME/$TAP_REPO"

# Step 1: Create the local directory for the tap repository
echo "Step 1: Creating local tap repository at $LOCAL_TAP_DIR"
mkdir -p "$LOCAL_TAP_DIR/Formula"

# Step 2: Copy the formula and README to the tap directory
echo "Step 2: Copying formula and README"
cp "$(pwd)/Formula/radio-cli.rb" "$LOCAL_TAP_DIR/Formula/"
if [ -f "$(pwd)/tap-readme.md" ]; then
  cp "$(pwd)/tap-readme.md" "$LOCAL_TAP_DIR/README.md"
else
  # Create a default README if one doesn't exist
  cat > "$LOCAL_TAP_DIR/README.md" << EOF
# Radio CLI Homebrew Tap

This repository contains the Homebrew formula for installing [Radio CLI](https://github.com/$GITHUB_USER/$CODE_REPO), a terminal-based internet radio player with visualizations.

## Installation

\`\`\`bash
# Add the tap
brew tap $GITHUB_USER/radio-cli

# Install radio-cli
brew install radio-cli
\`\`\`

## Features

- Play internet radio streams directly in your terminal
- Beautiful 90s-style starfield visualization
- Manage favorite stations
- Add custom radio stations
- Database storage for station information

## Dependencies

The following dependencies will be installed automatically:

- mpv - For audio playback
- sqlite - For database management
- Rust - For building (build-time only)

## Usage

After installation, you can run the application with:

\`\`\`bash
radio-cli
\`\`\`

Or:

\`\`\`bash
radio_cli
\`\`\`

## Station Database

Your station database will be automatically created in one of these locations (in priority order):
1. stations.db in the current directory (if it exists)
2. The location specified in the XDG_DATA_HOME environment variable
3. Platform-specific data directory:
   - macOS: ~/Library/Application Support/radio_cli/stations.db
   - Linux: ~/.local/share/radio_cli/stations.db
   - Windows: %APPDATA%/radio_cli/stations.db
EOF
fi

# Step 3: Initialize git repository
echo "Step 3: Initializing git repository for Homebrew tap"
cd "$LOCAL_TAP_DIR"
git init --quiet
git add .
git commit -m "Initial commit of Radio CLI homebrew formula"
git branch -M main

# Step 4: Check if the remote repository exists or needs to be created
echo "Step 4: Checking remote repository status"
if gh repo view "$GITHUB_USER/$TAP_REPO" &>/dev/null; then
  echo "Remote repository $GITHUB_USER/$TAP_REPO already exists"
else
  echo "Creating remote repository $GITHUB_USER/$TAP_REPO"
  gh repo create "$TAP_REPO" --public --description "Homebrew tap for Radio CLI" --confirm || {
    echo ""
    echo "WARNING: Could not automatically create the GitHub repository."
    echo "Please create it manually at https://github.com/new"
    echo "Repository name: $TAP_REPO"
    echo "Description: Homebrew tap for Radio CLI"
    echo "Make it Public"
    echo ""
    echo "After creating the repository, come back here and press Enter to continue..."
    read -r
  }
fi

# Step 5: Set the remote URL and push
echo "Step 5: Setting remote URL and pushing to GitHub"
if git remote -v | grep origin >/dev/null; then
  git remote set-url origin "git@github.com:$GITHUB_USER/$TAP_REPO.git" || {
    git remote add origin "git@github.com:$GITHUB_USER/$TAP_REPO.git"
  }
else
  git remote add origin "git@github.com:$GITHUB_USER/$TAP_REPO.git"
fi

echo "Pushing to remote repository..."
git push -u origin main --force

# Step 6: Tap the repository
echo "Step 6: Tapping the repository in Homebrew"
brew untap "$GITHUB_USER/radio-cli" 2>/dev/null || true
brew tap "$GITHUB_USER/radio-cli"

echo ""
echo "=== Homebrew Tap Setup Complete! ==="
echo ""
echo "You can now install Radio CLI with:"
echo "  brew install $GITHUB_USER/radio-cli/radio-cli"
echo ""
echo "Future updates to the formula can be made by:"
echo "1. Editing the formula at $LOCAL_TAP_DIR/Formula/radio-cli.rb"
echo "2. Committing and pushing changes:"
echo "   cd $LOCAL_TAP_DIR"
echo "   git add Formula/radio-cli.rb"
echo "   git commit -m \"Update formula\""
echo "   git push"
echo ""
echo "Users can then update with:"
echo "  brew update"
echo "  brew upgrade radio-cli"
echo ""