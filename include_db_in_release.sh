#!/bin/bash
set -e

echo "Ensuring stations.db is properly packaged in the release:"

# Make sure the stations.db file is at the root for the Homebrew formula
if [ -f "stations.db" ]; then
  echo "✓ stations.db found at root level"
else
  echo "! stations.db not found at root level, copying from the original location..."
  cp stations.db .
  echo "✓ stations.db copied to root level"
fi

# Make sure stations.db is committed 
if git ls-files --error-unmatch stations.db &>/dev/null; then
  echo "✓ stations.db is already committed to git"
else
  echo "! stations.db is not committed to git, adding it..."
  git add stations.db
  git commit -m "Include stations.db in repository for Homebrew formula"
  echo "✓ stations.db added to git"
fi

echo "Done! The stations.db file is now properly packaged for Homebrew."