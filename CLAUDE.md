# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This repository provides a Claude Code command (`/worktree`) for managing isolated git worktrees with automatic port allocation and lifecycle scripts. It is designed to be installed into the user's `~/.claude/` directory.

## Architecture

### Command System

The entry point is `commands/worktree.md`, a Claude Code command definition that routes subcommands (init, run, stop, close, list) or creates new worktrees by default.

### Helper Scripts (`scripts/worktree/`)

- **init.py** - Checks prerequisites (python3, git, claude CLI, bash) and detects if project is already initialized
- **allocate-ports.py** - Finds N consecutive free ports (default: 10 in range 50000-60000) and tracks allocations in `~/.claude/worktrees/port-allocations.json`
- **deallocate-ports.py** - Releases port allocations when worktrees are closed
- **list.py** - Lists all worktrees grouped by project, with port info and metadata
- **detect-worktree.sh** - Traverses directory hierarchy looking for `state.json` to identify if running within a worktree
- **open-terminal.sh** - Opens a new terminal window at the worktree path
- **run.sh/stop.sh/close.sh** - Wrapper scripts for project lifecycle commands

### Per-Project Configuration

When a user runs `/worktree init` in their project, these files are created in `.claude/worktree/`:

- `settings.json` - Team-shared settings (portCount, portRangeStart/End, branchPrefix, autoLaunchTerminal)
- `settings.local.json` - Personal settings like worktreeDir (gitignored)
- `SETUP.md` - Instructions for Claude on handling `/worktree <param>` (e.g., Linear issue ID)
- `setup.sh`, `run.sh`, `stop.sh`, `close.sh` - Project-specific lifecycle scripts

### Worktree State

Each created worktree has a `state.json` file containing:
- Worktree name and project name
- Original and worktree directories
- Branch name
- Allocated ports array
- Allocation key (used for port tracking)
- Creation timestamp

### Directory Structure

Default: `~/.claude/worktrees/<project>/<worktree>/`
Custom (via settings.local.json): `<customDir>/<worktree>/`

## Environment Variables for Scripts

Lifecycle scripts receive: `WORKTREE_NAME`, `WORKTREE_PROJECT`, `WORKTREE_DIR`, `WORKTREE_ORIGINAL_DIR`, `WORKTREE_PARAM`, `WORKTREE_PORT_0` through `WORKTREE_PORT_9`.
