# Installing RadioCLI with Homebrew

RadioCLI can be easily installed using Homebrew on macOS and Linux.

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