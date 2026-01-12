#!/usr/bin/env python3
"""
Lists all active worktrees with their port allocations.
Usage: python3 list.py
"""
import json
import sys
from pathlib import Path

ALLOCATIONS_FILE = Path.home() / ".claude" / "worktrees" / "port-allocations.json"
WORKTREES_DIR = Path.home() / ".claude" / "worktrees"


def load_allocations() -> dict:
    """Load existing port allocations from file."""
    if ALLOCATIONS_FILE.exists():
        return json.loads(ALLOCATIONS_FILE.read_text())
    return {}


def save_allocations(allocations: dict) -> None:
    """Save port allocations to file."""
    ALLOCATIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
    ALLOCATIONS_FILE.write_text(json.dumps(allocations, indent=2))


def find_worktree_dir(allocation_key: str) -> Path | None:
    """Find the worktree directory for an allocation key.

    Allocation keys can be:
    - "project/worktree" for default directory structure
    - "worktree" for custom directories (need to search state.json files)
    """
    # Check if it's a project/worktree key
    if "/" in allocation_key:
        project, name = allocation_key.split("/", 1)
        worktree_dir = WORKTREES_DIR / project / name
        if worktree_dir.exists():
            return worktree_dir

    # Check direct path (for backwards compatibility or custom dirs)
    direct_path = WORKTREES_DIR / allocation_key
    if direct_path.exists():
        return direct_path

    # Search all state.json files for matching allocationKey
    for state_file in WORKTREES_DIR.rglob("state.json"):
        try:
            state = json.loads(state_file.read_text())
            if state.get("allocationKey") == allocation_key:
                return state_file.parent
        except (json.JSONDecodeError, IOError):
            continue

    return None


def cleanup_stale_allocations(allocations: dict) -> dict:
    """Remove allocations for worktrees that no longer exist."""
    stale = []
    for key in list(allocations.keys()):
        worktree_dir = find_worktree_dir(key)
        if worktree_dir is None:
            stale.append(key)
        else:
            state_file = worktree_dir / "state.json"
            if not state_file.exists():
                stale.append(key)

    for key in stale:
        print(f"Cleaned up stale allocation: {key}", file=sys.stderr)
        del allocations[key]

    if stale:
        save_allocations(allocations)

    return allocations


def find_all_worktrees() -> list[dict]:
    """Find all worktrees by scanning for state.json files."""
    worktrees = []

    if not WORKTREES_DIR.exists():
        return worktrees

    # Scan for state.json files (handles both old and new directory structures)
    for state_file in WORKTREES_DIR.rglob("state.json"):
        # Skip the allocations file directory
        if state_file.parent == WORKTREES_DIR:
            continue

        try:
            state = json.loads(state_file.read_text())
            state["_dir"] = state_file.parent
            worktrees.append(state)
        except (json.JSONDecodeError, IOError) as e:
            print(f"Warning: Could not read {state_file}: {e}", file=sys.stderr)

    return worktrees


def main():
    allocations = load_allocations()
    allocations = cleanup_stale_allocations(allocations)

    worktrees = find_all_worktrees()

    if not worktrees:
        print("No active worktrees found.")
        return

    # Group worktrees by project
    by_project: dict[str, list[dict]] = {}
    for wt in worktrees:
        project = wt.get("projectName", "unknown")
        if project not in by_project:
            by_project[project] = []
        by_project[project].append(wt)

    print("\nActive worktrees:")
    print("=" * 80)

    for project in sorted(by_project.keys()):
        project_worktrees = by_project[project]
        print(f"\n## {project} ({len(project_worktrees)} worktree(s))")
        print("-" * 40)

        for wt in sorted(project_worktrees, key=lambda x: x.get("name", "")):
            name = wt.get("name", "unknown")
            ports = wt.get("ports", [])
            port_range = f"ports {ports[0]}-{ports[-1]}" if ports else "no ports"
            worktree_dir = wt.get("worktreeDir", wt.get("_dir", "unknown"))
            original_dir = wt.get("originalDir", "unknown")
            branch = wt.get("branch", "unknown")
            created = wt.get("createdAt", "unknown")

            print(f"\n  {name}")
            print(f"    {port_range}")
            print(f"    dir:     {worktree_dir}")
            print(f"    from:    {original_dir}")
            print(f"    branch:  {branch}")
            print(f"    created: {created}")

    total = sum(len(wts) for wts in by_project.values())
    print("\n" + "=" * 80)
    print(f"Total: {total} worktree(s) across {len(by_project)} project(s)\n")


if __name__ == "__main__":
    main()
