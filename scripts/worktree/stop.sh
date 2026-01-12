#!/bin/bash
#
# Runs the project's .claude/worktree/stop.sh script.
# Usage: stop.sh
#
# Must be run from within a Claude worktree.
#

set -e

SCRIPTS_DIR="$(dirname "$0")"

# Detect worktree and get state as JSON
STATE_JSON=$("$SCRIPTS_DIR/detect-worktree.sh")
if [ $? -ne 0 ]; then
    exit 1
fi

# Parse state.json using Python
eval "$(python3 -c "
import json

state = json.loads('''$STATE_JSON''')

print(f'export WORKTREE_NAME=\"{state[\"name\"]}\"')
print(f'export WORKTREE_PROJECT=\"{state.get(\"projectName\", \"unknown\")}\"')
print(f'export WORKTREE_DIR=\"{state[\"worktreeDir\"]}\"')
print(f'export WORKTREE_ORIGINAL_DIR=\"{state[\"originalDir\"]}\"')

for i, port in enumerate(state.get('ports', [])):
    print(f'export WORKTREE_PORT_{i}=\"{port}\"')
")"

# Check if project stop script exists
PROJECT_STOP_SCRIPT="$WORKTREE_DIR/.claude/worktree/stop.sh"
if [ ! -f "$PROJECT_STOP_SCRIPT" ]; then
    echo "Error: Project stop script not found: $PROJECT_STOP_SCRIPT" >&2
    echo "Create .claude/worktree/stop.sh in your project to define how to stop services." >&2
    exit 1
fi

if [ ! -x "$PROJECT_STOP_SCRIPT" ]; then
    echo "Error: Project stop script is not executable: $PROJECT_STOP_SCRIPT" >&2
    echo "Run: chmod +x $PROJECT_STOP_SCRIPT" >&2
    exit 1
fi

echo "Stopping services for $WORKTREE_PROJECT/$WORKTREE_NAME..."
exec "$PROJECT_STOP_SCRIPT"
