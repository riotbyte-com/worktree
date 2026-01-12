#!/bin/bash
#
# Detects if the current directory is within a Claude worktree.
# Usage: detect-worktree.sh
#
# Outputs JSON with worktree info if found, exits with error if not in a worktree.
#
# Handles both directory structures:
# - Default: ~/.claude/worktrees/<project>/<worktree>/
# - Custom: <customDir>/<worktree>/
#

CURRENT_DIR="$(pwd)"

# Look for state.json in current directory or any parent
find_state_json() {
    local dir="$1"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/state.json" ]; then
            echo "$dir/state.json"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

STATE_FILE=$(find_state_json "$CURRENT_DIR")

if [ -z "$STATE_FILE" ]; then
    echo "Error: Not in a Claude worktree" >&2
    echo "Current directory: $CURRENT_DIR" >&2
    echo "No state.json found in directory hierarchy" >&2
    exit 1
fi

WORKTREE_DIR="$(dirname "$STATE_FILE")"

# Output the worktree info
cat "$STATE_FILE"
