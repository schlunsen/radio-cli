# Release Process for RadioCLI

This document outlines the steps to create a new release of RadioCLI and update the Homebrew formula.

## Creating a Release

1. Update version in `radio_cli/Cargo.toml`
2. Update CHANGELOG.md if you have one
3. Commit changes: `git commit -am "Bump version to X.Y.Z"`
4. Create a git tag: `git tag -a vX.Y.Z -m "Version X.Y.Z"`
5. Push to GitHub: `git push && git push --tags`
6. Create a GitHub release from the tag

## Updating the Homebrew Formula

After releasing a new version, update the Homebrew formula:

1. Download the tarball from GitHub: `curl -L https://github.com/schlunsen/radiocli/archive/refs/tags/vX.Y.Z.tar.gz -o vX.Y.Z.tar.gz`
2. Calculate the SHA256: `shasum -a 256 vX.Y.Z.tar.gz`
3. Update the formula in `Formula/radio-cli.rb`:
   - Update the version in the URL
   - Update the SHA256 hash

Example:
```ruby
class RadioCli < Formula
  # ...
  url "https://github.com/schlunsen/radiocli/archive/refs/tags/vX.Y.Z.tar.gz"
  sha256 "CALCULATED_SHA256"
  # ...
end
```

4. Commit and push the formula changes: `git commit -am "Update formula to vX.Y.Z" && git push`

## Testing the Formula Locally

Before releasing, test the formula locally:

```bash
brew install --build-from-source ./Formula/radio-cli.rb
```

## Versioning Guidelines

Follow semantic versioning (semver.org):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes