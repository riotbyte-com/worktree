#!/usr/bin/env python3
"""
Removes port allocation for a worktree.
Usage: python3 deallocate-ports.py <worktree_name>
"""
import json
import sys
from pathlib import Path

ALLOCATIONS_FILE = Path.home() / ".claude" / "worktrees" / "port-allocations.json"


def load_allocations() -> dict:
    """Load existing port allocations from file."""
    if ALLOCATIONS_FILE.exists():
        return json.loads(ALLOCATIONS_FILE.read_text())
    return {}


def save_allocations(allocations: dict) -> None:
    """Save port allocations to file."""
    ALLOCATIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
    ALLOCATIONS_FILE.write_text(json.dumps(allocations, indent=2))


def main():
    if len(sys.argv) != 2:
        print("Usage: python3 deallocate-ports.py <worktree_name>", file=sys.stderr)
        sys.exit(1)

    worktree_name = sys.argv[1]

    allocations = load_allocations()

    if worktree_name not in allocations:
        print(f"No allocation found for worktree: {worktree_name}", file=sys.stderr)
        sys.exit(0)

    ports = allocations[worktree_name]
    del allocations[worktree_name]
    save_allocations(allocations)

    print(json.dumps({"deallocated": True, "ports": ports}))


if __name__ == "__main__":
    main()
