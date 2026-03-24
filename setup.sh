#!/usr/bin/env bash
# Install FreeSynergy git hooks for this repo.
# Run once after cloning: bash setup.sh
set -e
git config core.hooksPath .githooks
echo "Git hooks installed. Pre-commit checks are now active."
