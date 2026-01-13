# Worktree Setup Instructions

When a parameter is passed to `worktree new <param>`, it's available as `$WORKTREE_PARAM` in the setup script.

## Common Use Cases

### Linear/Jira Issue IDs

If your team uses issue IDs like `PROJ-123`:
- Use the ID to name the branch: `feature/PROJ-123`
- Fetch issue details via API for commit messages
- Link the worktree to the issue in your tracking system

### Feature Branch Names

If passing a feature name directly:
- Create a branch like `feature/<param>`
- Set up any feature-specific configuration

### Custom Parameters

Define your own parameter conventions:
- Environment names (dev, staging)
- Customer-specific configurations
- Experiment identifiers

## Example setup.sh Usage

```bash
#!/bin/bash
if [ -n "$WORKTREE_PARAM" ]; then
    echo "Setting up for: $WORKTREE_PARAM"
    # Parse issue ID, fetch details, etc.
fi
```
