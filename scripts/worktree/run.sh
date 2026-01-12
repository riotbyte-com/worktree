#!/bin/bash
#
# Runs the project's .claude/worktree/run.sh script with port environment variables.
# Usage: run.sh
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
import sys

state = json.loads('''$STATE_JSON''')

print(f'export WORKTREE_NAME=\"{state[\"name\"]}\"')
print(f'export WORKTREE_PROJECT=\"{state.get(\"projectName\", \"unknown\")}\"')
print(f'export WORKTREE_DIR=\"{state[\"worktreeDir\"]}\"')
print(f'export WORKTREE_ORIGINAL_DIR=\"{state[\"originalDir\"]}\"')

for i, port in enumerate(state.get('ports', [])):
    print(f'export WORKTREE_PORT_{i}=\"{port}\"')
")"

# Check if project run script exists
PROJECT_RUN_SCRIPT="$WORKTREE_DIR/.claude/worktree/run.sh"
if [ ! -f "$PROJECT_RUN_SCRIPT" ]; then
    echo "Error: Project run script not found: $PROJECT_RUN_SCRIPT" >&2
    echo "Create .claude/worktree/run.sh in your project to define how to start services." >&2
    exit 1
fi

if [ ! -x "$PROJECT_RUN_SCRIPT" ]; then
    echo "Error: Project run script is not executable: $PROJECT_RUN_SCRIPT" >&2
    echo "Run: chmod +x $PROJECT_RUN_SCRIPT" >&2
    exit 1
fi

echo "Running project script for $WORKTREE_PROJECT/$WORKTREE_NAME with ports: $WORKTREE_PORT_0-$WORKTREE_PORT_9"
exec "$PROJECT_RUN_SCRIPT"
