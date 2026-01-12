#!/bin/bash
#
# Closes and cleans up a Claude worktree.
# Usage: close.sh
#
# Must be run from within a Claude worktree.
# This script will:
# 1. Run the project's close.sh if it exists
# 2. Deallocate ports
# 3. Remove the git worktree
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
print(f'export WORKTREE_ALLOCATION_KEY=\"{state.get(\"allocationKey\", state[\"name\"])}\"')

for i, port in enumerate(state.get('ports', [])):
    print(f'export WORKTREE_PORT_{i}=\"{port}\"')
")"

echo "Closing worktree: $WORKTREE_PROJECT/$WORKTREE_NAME"
echo "Original directory: $WORKTREE_ORIGINAL_DIR"

# Run project close script if it exists
PROJECT_CLOSE_SCRIPT="$WORKTREE_DIR/.claude/worktree/close.sh"
if [ -x "$PROJECT_CLOSE_SCRIPT" ]; then
    echo "Running project close script..."
    (
        "$PROJECT_CLOSE_SCRIPT"
    ) || echo "Warning: Project close script failed (continuing anyway)"
fi

# Deallocate ports using the allocation key
echo "Deallocating ports..."
python3 "$SCRIPTS_DIR/deallocate-ports.py" "$WORKTREE_ALLOCATION_KEY" || echo "Warning: Port deallocation failed"

# Change to home directory before removing worktree (can't be in the directory we're removing)
cd "$HOME"

# Remove git worktree
echo "Removing git worktree..."
if [ -d "$WORKTREE_ORIGINAL_DIR" ]; then
    git -C "$WORKTREE_ORIGINAL_DIR" worktree remove "$WORKTREE_DIR" --force 2>/dev/null || {
        echo "Warning: git worktree remove failed, removing directory manually..."
        rm -rf "$WORKTREE_DIR"
        git -C "$WORKTREE_ORIGINAL_DIR" worktree prune 2>/dev/null || true
    }
else
    echo "Warning: Original directory not found, removing worktree directory manually..."
    rm -rf "$WORKTREE_DIR"
fi

echo ""
echo "=========================================="
echo "Worktree '$WORKTREE_PROJECT/$WORKTREE_NAME' has been closed."
echo ""
echo "To return to the original project:"
echo "  cd $WORKTREE_ORIGINAL_DIR && claude"
echo "=========================================="
