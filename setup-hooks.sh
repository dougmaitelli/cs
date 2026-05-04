#!/usr/bin/env bash
set -e

echo "Setting up git hooks..."

# Set the hooks path to .githooks
git config core.hooksPath .githooks

echo "Git hooks configured successfully!"
echo "Hooks are now linked from .githooks/"
