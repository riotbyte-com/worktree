#!/bin/bash
# Setup script for worktree: $WORKTREE_NAME
# This script runs after the worktree is created

set -e

echo "Setting up worktree: $WORKTREE_NAME"

echo "Worktree name: $WORKTREE_NAME" > worktree.info
echo "Allocated ports: $WORKTREE_PORT_0 - $WORKTREE_PORT_9" >> worktree.info

# TODO: Add your setup commands here
# Examples:
# - npm install
# - cp .env.example .env
# - Update .env with allocated ports
# - Run database migrations

echo "Setup complete!"
