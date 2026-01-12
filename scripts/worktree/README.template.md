# Worktree Configuration

This directory contains configuration and scripts for the `/worktree` Claude command, which manages isolated git worktrees for parallel development.

## Overview

The `/worktree` command creates git worktrees in a separate directory, allocates dedicated ports, and runs lifecycle scripts to set up your development environment. This allows you to work on multiple features or issues simultaneously without conflicts.

## Directory Structure

Worktrees are organized by project to make it easy to see which worktrees belong to which projects:

**Default structure** (when no custom directory is configured):
```
~/.claude/worktrees/
├── my-project/                   # Project name (from original directory)
│   ├── swift-falcon-a3b2/        # Worktree 1
│   │   ├── state.json
│   │   └── ... (git worktree files)
│   └── happy-tiger-c4d5/         # Worktree 2
│       ├── state.json
│       └── ...
├── another-project/
│   └── cool-wolf-e6f7/
│       └── ...
└── port-allocations.json         # Central port tracking
```

**Custom directory structure** (when `worktreeDir` is set in settings.local.json):
```
/path/to/custom/dir/
├── swift-falcon-a3b2/            # Worktrees directly in custom dir
│   ├── state.json                # (no project subdirectory)
│   └── ...
└── happy-tiger-c4d5/
    └── ...
```

When using a custom directory, worktrees are created directly in that directory without a project subdirectory, since custom directories are typically already project-specific.

## Commands

| Command | Description |
|---------|-------------|
| `/worktree` | Create a new worktree with allocated ports |
| `/worktree <param>` | Create worktree with parameter (see SETUP.md for handling) |
| `/worktree init` | Initialize worktree configuration for this project |
| `/worktree run` | Start the development environment |
| `/worktree stop` | Stop running services |
| `/worktree close` | Clean up and delete the worktree |
| `/worktree list` | Show all active worktrees |

### Example: `/worktree list` Output

The list command groups worktrees by project:

```
Active worktrees:
================================================================================

## my-project (2 worktree(s))
----------------------------------------

  swift-falcon-a3b2
    ports 50000-50009
    dir:     /home/user/.claude/worktrees/my-project/swift-falcon-a3b2
    from:    /home/user/projects/my-project
    branch:  feature/issue-33
    created: 2026-01-12T10:00:00Z

  happy-tiger-c4d5
    ports 50010-50019
    dir:     /home/user/.claude/worktrees/my-project/happy-tiger-c4d5
    from:    /home/user/projects/my-project
    branch:  feature/issue-45
    created: 2026-01-12T11:30:00Z

## another-project (1 worktree(s))
----------------------------------------

  cool-wolf-e6f7
    ports 50020-50029
    dir:     /home/user/.claude/worktrees/another-project/cool-wolf-e6f7
    from:    /home/user/projects/another-project
    branch:  main
    created: 2026-01-11T14:00:00Z

================================================================================
Total: 3 worktree(s) across 2 project(s)
```

## Configuration Files

### `settings.json` (team-shared)

Shared settings that should be committed to the repository:

```json
{
  "portCount": 10,
  "portRangeStart": 50000,
  "portRangeEnd": 60000,
  "branchPrefix": "worktree/",
  "autoLaunchTerminal": true
}
```

| Setting | Description |
|---------|-------------|
| `portCount` | Number of consecutive ports to allocate (default: 10) |
| `portRangeStart` | Start of port range to allocate from (default: 50000) |
| `portRangeEnd` | End of port range (default: 60000) |
| `branchPrefix` | Prefix for auto-generated branch names (default: "worktree/") |
| `autoLaunchTerminal` | Whether to auto-launch a new terminal (default: true) |

### `settings.local.json` (personal, gitignored)

User-specific settings that override team settings:

```json
{
  "worktreeDir": "/path/to/your/worktrees"
}
```

| Setting | Description |
|---------|-------------|
| `worktreeDir` | Custom directory where worktrees are created (default: `~/.claude/worktrees/<project>/`) |

**Note about custom directories**: When you set a custom `worktreeDir`, worktrees are created directly in that directory without a project subdirectory. This is because custom directories are typically already project-specific. The port allocation key will use just the worktree name instead of `project/name`.

### `SETUP.md`

Instructions for Claude on how to handle parameters passed to `/worktree <param>`. For example, you can configure it to fetch Linear issue details and use the issue's branch name.

## Lifecycle Scripts

All scripts receive environment variables with worktree information and allocated ports.

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `WORKTREE_NAME` | Unique name of the worktree | `swift-falcon-a3b2` |
| `WORKTREE_PROJECT` | Project name (from original directory) | `my-project` |
| `WORKTREE_DIR` | Full path to the worktree | `/home/user/.claude/worktrees/my-project/swift-falcon-a3b2` |
| `WORKTREE_ORIGINAL_DIR` | Path to the original project | `/home/user/projects/my-project` |
| `WORKTREE_ALLOCATION_KEY` | Key used for port allocation | `my-project/swift-falcon-a3b2` |
| `WORKTREE_PARAM` | Parameter passed to /worktree | `CHR-123` |
| `WORKTREE_PORT_0` | First allocated port | `50000` |
| `WORKTREE_PORT_1` | Second allocated port | `50001` |
| ... | ... | ... |
| `WORKTREE_PORT_9` | Tenth allocated port | `50009` |

### `setup.sh`

Runs automatically after the worktree is created. Use this for:
- Installing dependencies
- Creating/copying environment files
- Setting up databases
- Running migrations
- Any other initialization

**Example (Node.js/Bun project):**
```bash
#!/bin/bash
set -e

echo "Setting up worktree: $WORKTREE_NAME"

# Install dependencies
bun install

# Create .env from template
if [ -f .env.example ]; then
  cp .env.example .env
  # Update port in .env
  sed -i '' "s/^PORT=.*/PORT=$WORKTREE_PORT_0/" .env
fi

echo "Setup complete!"
```

**Example (Python project):**
```bash
#!/bin/bash
set -e

echo "Setting up worktree: $WORKTREE_NAME"

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Create .env from template
if [ -f .env.example ]; then
  cp .env.example .env
  sed -i '' "s/^PORT=.*/PORT=$WORKTREE_PORT_0/" .env
fi

echo "Setup complete!"
```

**Example (PHP/Laravel project):**
```bash
#!/bin/bash
set -e

echo "Setting up worktree: $WORKTREE_NAME"

# Install dependencies
composer install

# Create .env from template
cp .env.example .env
php artisan key:generate

# Update ports in .env
sed -i '' "s/^APP_PORT=.*/APP_PORT=$WORKTREE_PORT_0/" .env
sed -i '' "s/^VITE_PORT=.*/VITE_PORT=$WORKTREE_PORT_1/" .env

# Run migrations (using sqlite for isolation)
sed -i '' "s/^DB_CONNECTION=.*/DB_CONNECTION=sqlite/" .env
touch database/database.sqlite
php artisan migrate

echo "Setup complete!"
```

### `run.sh`

Starts the development environment. Use this for:
- Starting the dev server
- Starting databases or other services
- Running background workers

**Example (Node.js/Bun project):**
```bash
#!/bin/bash
set -e

echo "Starting dev server on port $WORKTREE_PORT_0..."
PORT=$WORKTREE_PORT_0 bun run dev
```

**Example (Python/Django project):**
```bash
#!/bin/bash
set -e

source .venv/bin/activate
echo "Starting Django on port $WORKTREE_PORT_0..."
python manage.py runserver 0.0.0.0:$WORKTREE_PORT_0
```

**Example (PHP/Laravel project):**
```bash
#!/bin/bash
set -e

echo "Starting Laravel on port $WORKTREE_PORT_0..."
php artisan serve --port=$WORKTREE_PORT_0
```

**Example (with multiple services):**
```bash
#!/bin/bash
set -e

# Start database on port 1
echo "Starting PostgreSQL on port $WORKTREE_PORT_1..."
docker run -d --name "${WORKTREE_NAME}-db" \
  -p $WORKTREE_PORT_1:5432 \
  -e POSTGRES_PASSWORD=secret \
  postgres:15

# Start Redis on port 2
echo "Starting Redis on port $WORKTREE_PORT_2..."
docker run -d --name "${WORKTREE_NAME}-redis" \
  -p $WORKTREE_PORT_2:6379 \
  redis:7

# Wait for services
sleep 3

# Start app on port 0
echo "Starting app on port $WORKTREE_PORT_0..."
bun run dev
```

### `stop.sh`

Stops running services. Use this for:
- Stopping the dev server
- Stopping databases or containers
- Killing processes on allocated ports

**Example (simple):**
```bash
#!/bin/bash

echo "Stopping services..."

# Kill process on dev server port
lsof -ti:$WORKTREE_PORT_0 | xargs kill -9 2>/dev/null || true

echo "Services stopped."
```

**Example (with Docker):**
```bash
#!/bin/bash

echo "Stopping services..."

# Stop Docker containers
docker stop "${WORKTREE_NAME}-db" 2>/dev/null || true
docker stop "${WORKTREE_NAME}-redis" 2>/dev/null || true
docker rm "${WORKTREE_NAME}-db" 2>/dev/null || true
docker rm "${WORKTREE_NAME}-redis" 2>/dev/null || true

# Kill any remaining processes on allocated ports
for i in {0..9}; do
  port_var="WORKTREE_PORT_$i"
  port="${!port_var}"
  if [ -n "$port" ]; then
    lsof -ti:$port | xargs kill -9 2>/dev/null || true
  fi
done

echo "Services stopped."
```

### `close.sh`

Runs before the worktree is deleted. Use this for:
- Final cleanup
- Removing temporary files
- Stopping any remaining services

**Example:**
```bash
#!/bin/bash

echo "Cleaning up worktree: $WORKTREE_NAME"

# Stop services first
.claude/worktree/stop.sh 2>/dev/null || true

# Remove generated files (optional)
# rm -rf node_modules
# rm -rf .venv
# rm -rf vendor

echo "Cleanup complete."
```

## Port Allocation

Each worktree gets 10 consecutive ports by default. Common usage patterns:

| Port | Common Use |
|------|------------|
| `WORKTREE_PORT_0` | Main application / dev server |
| `WORKTREE_PORT_1` | Database (PostgreSQL, MySQL, etc.) |
| `WORKTREE_PORT_2` | Cache (Redis, Memcached, etc.) |
| `WORKTREE_PORT_3` | Queue worker / message broker |
| `WORKTREE_PORT_4` | Frontend dev server (if separate) |
| `WORKTREE_PORT_5` - `WORKTREE_PORT_9` | Additional services as needed |

Ports are tracked centrally to avoid conflicts between worktrees. When a worktree is closed, its ports are released for reuse.

## Tips

1. **Keep scripts idempotent**: Scripts may be run multiple times, so make sure they handle that gracefully.

2. **Use environment variables**: Always use `WORKTREE_PORT_*` variables instead of hardcoding ports.

3. **Document port usage**: If your project uses multiple ports, document which service uses which port in your scripts.

4. **Handle failures gracefully**: Use `|| true` for commands that might fail during cleanup.

5. **Test locally first**: Test your scripts manually before relying on them in worktrees.

## Troubleshooting

### Worktree creation fails
- Check if the branch already exists: `git branch -a | grep <branch-name>`
- Ensure you have enough disk space
- Verify the worktree directory is writable

### Port conflicts
- Run `/worktree list` to see all allocated ports
- Check for stale allocations: manually deleted worktrees are cleaned up automatically on next allocation

### Scripts not running
- Ensure scripts are executable: `chmod +x .claude/worktree/*.sh`
- Check script syntax: `bash -n .claude/worktree/setup.sh`
- Look for error messages in the output

### Services not stopping
- Check for orphaned processes: `lsof -i :<port>`
- Kill manually if needed: `kill -9 <pid>`
