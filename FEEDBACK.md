1. Add tmux support:
    - When the tmux terminal option is set, worktrees should be opened as new tmux sessions using the format: `<project>-<worktree>`
    - The close command should terminate the tmux session associated with the worktree being closed
2. Add a `worktree open` command:
    - This command should open the specified worktree using the configured terminal option
