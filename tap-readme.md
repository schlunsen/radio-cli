# Radio CLI Homebrew Tap

This repository contains the Homebrew formula for installing [Radio CLI](https://github.com/schlunsen/radio-cli), a terminal-based internet radio player with visualizations.

## Installation

```bash
# Add the tap
brew tap schlunsen/radio-cli

# Install radio-cli
brew install radio-cli
```

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

```bash
radio-cli
```

Or:

```bash
radio_cli
```

## Station Database

Your station database is stored at:
```
/opt/homebrew/var/radio_cli/stations.db
```

## Troubleshooting

If you encounter any issues, please report them at:
https://github.com/schlunsen/radio-cli/issues