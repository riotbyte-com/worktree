# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Overview

This is a Rust CLI tool (`worktree`) for managing isolated git worktrees with automatic port allocation and lifecycle scripts.

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Check for errors without building
cargo test               # Run tests
cargo clippy             # Run linter
```

## Architecture

### Project Structure

```
src/
├── main.rs              # Entry point, CLI routing
├── cli.rs               # Clap CLI definitions
├── names.rs             # Random name generation
├── commands/            # Command implementations
│   ├── init.rs          # worktree init
│   ├── new.rs           # worktree [new] [param]
│   ├── run.rs           # worktree run
│   ├── stop.rs          # worktree stop
│   ├── close.rs         # worktree close
│   └── list.rs          # worktree list
├── config/              # Configuration management
│   ├── paths.rs         # Path utilities
│   ├── settings.rs      # Settings structs
│   └── state.rs         # Worktree state
├── ports/               # Port allocation
│   ├── allocator.rs     # Allocation logic
│   └── checker.rs       # Port availability
├── git/                 # Git operations
│   └── worktree.rs      # Git worktree commands
├── terminal/            # Terminal integration
│   └── launcher.rs      # Terminal detection/launch
└── scripts/             # Script handling
    ├── runner.rs        # Script execution
    └── generator.rs     # Claude CLI integration
```

### Key Data Files

- `~/.worktree/port-allocations.json` - Global port tracking
- `.worktree/settings.json` - Per-project team settings
- `.worktree/settings.local.json` - Per-project personal settings
- `state.json` - Per-worktree state (in worktree root)

### Configuration Schema

**settings.json** (team-shared):
```json
{
  "portCount": 10,
  "portRangeStart": 50000,
  "portRangeEnd": 60000,
  "branchPrefix": "worktree/",
  "autoLaunchTerminal": true
}
```

**state.json** (per-worktree):
```json
{
  "name": "swift-falcon-a3b2",
  "projectName": "my-project",
  "originalDir": "/path/to/project",
  "worktreeDir": "/path/to/worktree",
  "branch": "worktree/swift-falcon-a3b2",
  "ports": [50000, 50001, ...],
  "allocationKey": "my-project/swift-falcon-a3b2",
  "createdAt": "2026-01-13T10:00:00Z"
}
```

## Development Notes

- Uses `clap` for CLI parsing with derive macros
- Uses `serde` for JSON serialization
- Uses `anyhow` for error handling
- Uses `socket2` for port availability checking
- Cross-platform terminal detection (macOS + Linux)
- Optional Claude CLI integration for script generation
