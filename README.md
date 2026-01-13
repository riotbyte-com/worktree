# worktree

A standalone CLI tool for managing isolated git worktrees with automatic port allocation and lifecycle scripts.

## Features

- Create isolated git worktrees for parallel development
- Automatic port allocation to avoid conflicts between worktrees
- Lifecycle scripts (setup, run, stop, close) for each project
- Per-project configuration with team-shared and personal settings
- Automatic terminal launching for new worktrees
- Optional Claude CLI integration for intelligent script generation

## Installation

### Quick Install (Recommended)

Install the latest version:

```bash
curl -fsSL https://raw.githubusercontent.com/dbrekelmans/claude-worktree/main/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/dbrekelmans/claude-worktree/main/install.sh | bash -s v1.0.0
```

The binary is installed to `~/.local/bin` by default. Set `INSTALL_DIR` to customize:

```bash
curl -fsSL https://raw.githubusercontent.com/dbrekelmans/claude-worktree/main/install.sh | INSTALL_DIR=/usr/local/bin bash
```

### From Source

```bash
cargo install --path .
```

### Build Release Binary

```bash
cargo build --release
# Binary is at ./target/release/worktree
```

## Usage

```
worktree [COMMAND] [OPTIONS]

Commands:
  new [param]   Create a new worktree (default if no command given)
  init          Initialize worktree configuration for your project
  run           Start the development environment
  stop          Stop running services
  close         Clean up and delete the worktree
  list          Show all active worktrees
```

## Getting Started

### 1. Initialize your project (first time only)

```bash
worktree init
```

This creates a `.worktree/` directory with:
- `settings.json` - Team-shared settings (commit to repo)
- `settings.local.json` - Personal settings (gitignored)
- `setup.sh`, `run.sh`, `stop.sh`, `close.sh` - Lifecycle scripts

If Claude CLI is installed, it can optionally generate intelligent scripts based on your project.

### 2. Create a worktree

```bash
worktree
```

Or with a parameter (e.g., issue ID, branch name):

```bash
worktree new ISSUE-123
```

This will:
- Create a git worktree with a random name (e.g., `swift-falcon-a3b2`)
- Allocate ports for the worktree
- Run the setup script
- Launch a new terminal (if configured)

### 3. Work in the worktree

```bash
worktree run    # Start development environment
worktree stop   # Stop services
```

### 4. Close when done

```bash
worktree close
```

This deallocates ports and removes the git worktree.

## Configuration

### Settings (`settings.json`)

```json
{
  "portCount": 10,
  "portRangeStart": 50000,
  "portRangeEnd": 60000,
  "branchPrefix": "worktree/",
  "autoLaunchTerminal": true,
  "terminal": "iterm2"
}
```

| Setting | Description | Default |
|---------|-------------|---------|
| `portCount` | Number of ports to allocate per worktree | `10` |
| `portRangeStart` | Start of port allocation range | `50000` |
| `portRangeEnd` | End of port allocation range | `60000` |
| `branchPrefix` | Prefix for worktree branch names | `worktree/` |
| `autoLaunchTerminal` | Automatically open terminal on worktree creation | `true` |
| `terminal` | Terminal emulator to use (see below) | Auto-detect |

### Terminal Options

When `terminal` is not set, the tool auto-detects your terminal. You can explicitly set it to one of:

**Cross-platform:**
| Value | Terminal |
|-------|----------|
| `tmux` | tmux (creates a named session) |

**macOS:**
| Value | Terminal |
|-------|----------|
| `terminal`, `terminal.app`, `apple_terminal` | Terminal.app |
| `iterm`, `iterm2` | iTerm2 |
| `warp` | Warp |
| `ghostty` | Ghostty |
| `vscode`, `code` | VS Code |

**Linux:**
| Value | Terminal |
|-------|----------|
| `gnome-terminal`, `gnome` | GNOME Terminal |
| `konsole` | Konsole |
| `xfce4-terminal`, `xfce` | Xfce Terminal |
| `kitty` | Kitty |
| `alacritty` | Alacritty |

### Local Settings (`settings.local.json`)

```json
{
  "worktreeDir": "/custom/path/to/worktrees"
}
```

## Environment Variables

Lifecycle scripts receive these environment variables:

| Variable | Description |
|----------|-------------|
| `WORKTREE_NAME` | Unique worktree name (e.g., `swift-falcon-a3b2`) |
| `WORKTREE_PROJECT` | Project name |
| `WORKTREE_DIR` | Full path to worktree directory |
| `WORKTREE_ORIGINAL_DIR` | Full path to original project |
| `WORKTREE_PORT_0` - `WORKTREE_PORT_9` | Allocated ports |
| `WORKTREE_PARAM` | Parameter passed to `worktree new` |

## Directory Structure

```
~/.worktree/
├── worktrees/
│   └── my-project/
│       ├── swift-falcon-a3b2/
│       │   ├── state.json
│       │   └── (git worktree files)
│       └── happy-tiger-c4d5/
└── port-allocations.json

your-project/
└── .worktree/
    ├── settings.json
    ├── settings.local.json
    ├── setup.sh
    ├── run.sh
    ├── stop.sh
    └── close.sh
```

## License

MIT License
