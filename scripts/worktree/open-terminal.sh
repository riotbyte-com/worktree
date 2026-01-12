#!/bin/bash
#
# Opens a new terminal tab/window and runs Claude in the specified worktree directory.
# Usage: open-terminal.sh <worktree_dir>
#
# Supports:
# - macOS: Terminal.app, iTerm2, VS Code, Warp, Ghostty
# - Linux: gnome-terminal, konsole, xfce4-terminal, kitty, alacritty
#

set -e

WORKTREE_DIR="$1"

if [ -z "$WORKTREE_DIR" ]; then
    echo "Usage: open-terminal.sh <worktree_dir>" >&2
    exit 1
fi

# Expand ~ if present
WORKTREE_DIR="${WORKTREE_DIR/#\~/$HOME}"

if [ ! -d "$WORKTREE_DIR" ]; then
    echo "Error: Directory does not exist: $WORKTREE_DIR" >&2
    exit 1
fi

CMD="cd \"$WORKTREE_DIR\" && claude"

# macOS terminals (check TERM_PROGRAM)
case "$TERM_PROGRAM" in
    "Apple_Terminal")
        osascript -e "tell application \"Terminal\"
            activate
            do script \"$CMD\"
        end tell"
        echo "Opened new Terminal.app window"
        exit 0
        ;;

    "iTerm.app")
        osascript <<EOF
tell application "iTerm"
    activate
    tell current window
        create tab with default profile
        tell current session
            write text "$CMD"
        end tell
    end tell
end tell
EOF
        echo "Opened new iTerm2 tab"
        exit 0
        ;;

    "vscode")
        # VS Code: open the folder, user will need to run claude manually
        code "$WORKTREE_DIR"
        echo "Opened VS Code at $WORKTREE_DIR"
        echo "Run 'claude' in the integrated terminal."
        exit 0
        ;;

    "WarpTerminal")
        osascript -e "tell application \"Warp\"
            activate
            do script \"$CMD\"
        end tell"
        echo "Opened new Warp window"
        exit 0
        ;;

    "Ghostty")
        # Ghostty supports new window via CLI
        ghostty -e bash -c "$CMD" &
        echo "Opened new Ghostty window"
        exit 0
        ;;
esac

# Linux terminals (check common terminal emulators)
if [ "$(uname)" = "Linux" ]; then
    if command -v gnome-terminal &>/dev/null; then
        gnome-terminal --tab --working-directory="$WORKTREE_DIR" -- bash -c "claude; exec bash" &
        echo "Opened new gnome-terminal tab"
        exit 0
    elif command -v konsole &>/dev/null; then
        konsole --new-tab --workdir "$WORKTREE_DIR" -e bash -c "claude; exec bash" &
        echo "Opened new Konsole tab"
        exit 0
    elif command -v xfce4-terminal &>/dev/null; then
        xfce4-terminal --tab --working-directory="$WORKTREE_DIR" -e "bash -c 'claude; exec bash'" &
        echo "Opened new XFCE terminal tab"
        exit 0
    elif command -v kitty &>/dev/null; then
        kitty --directory="$WORKTREE_DIR" bash -c "claude; exec bash" &
        echo "Opened new Kitty window"
        exit 0
    elif command -v alacritty &>/dev/null; then
        alacritty --working-directory "$WORKTREE_DIR" -e bash -c "claude; exec bash" &
        echo "Opened new Alacritty window"
        exit 0
    fi
fi

# Fallback: output command for user to copy
echo "=========================================="
echo "Could not auto-launch terminal."
echo "Run this command in a new terminal:"
echo ""
echo "  cd $WORKTREE_DIR && claude"
echo "=========================================="
exit 0
