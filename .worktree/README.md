# Worktree Configuration

This directory contains the worktree configuration for this project.

## Files

- `settings.json` - Team-shared settings (committed to repo)
- `settings.local.json` - Personal settings like custom worktree directory (gitignored)
- `setup.sh` - Runs after a new worktree is created
- `run.sh` - Starts the development environment
- `stop.sh` - Stops running services
- `close.sh` - Cleanup before worktree deletion
- `SETUP.md` - Instructions for handling parameters passed to `worktree new`

## Environment Variables

All lifecycle scripts receive these environment variables:

| Variable | Description |
|----------|-------------|
| `WORKTREE_NAME` | Unique worktree name (e.g., `swift-falcon-a3b2`) |
| `WORKTREE_PROJECT` | Project name |
| `WORKTREE_DIR` | Full path to worktree directory |
| `WORKTREE_ORIGINAL_DIR` | Full path to original project |
| `WORKTREE_PORT_0` - `WORKTREE_PORT_9` | Allocated ports |
| `WORKTREE_PARAM` | Parameter passed to `worktree new` |

## Usage

```bash
# Create a new worktree
worktree

# Create with a parameter (e.g., issue ID)
worktree new ISSUE-123

# Start development
worktree run

# Stop services
worktree stop

# Close and cleanup
worktree close

# List all worktrees
worktree list
```
