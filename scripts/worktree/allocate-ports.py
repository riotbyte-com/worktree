#!/usr/bin/env python3
"""
Finds N consecutive free ports and records the allocation.
Usage: python3 allocate-ports.py <count> <worktree_name>
Returns: JSON with allocated ports
"""
import socket
import json
import sys
from pathlib import Path

ALLOCATIONS_FILE = Path.home() / ".claude" / "worktrees" / "port-allocations.json"
WORKTREES_DIR = Path.home() / ".claude" / "worktrees"
PORT_RANGE_START = 50000
PORT_RANGE_END = 60000


def is_port_free(port: int) -> bool:
    """Check if a port is available for binding."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.bind(('127.0.0.1', port))
            return True
        except OSError:
            return False


def load_allocations() -> dict:
    """Load existing port allocations from file."""
    if ALLOCATIONS_FILE.exists():
        return json.loads(ALLOCATIONS_FILE.read_text())
    return {}


def save_allocations(allocations: dict) -> None:
    """Save port allocations to file."""
    ALLOCATIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
    ALLOCATIONS_FILE.write_text(json.dumps(allocations, indent=2))


def cleanup_stale_allocations(allocations: dict) -> dict:
    """Remove allocations for worktrees that no longer exist."""
    stale = []
    for name in list(allocations.keys()):
        worktree_dir = WORKTREES_DIR / name
        state_file = worktree_dir / "state.json"
        # Consider stale if directory doesn't exist or has no state.json
        if not worktree_dir.exists() or not state_file.exists():
            stale.append(name)

    for name in stale:
        print(f"Cleaning up stale allocation: {name}", file=sys.stderr)
        del allocations[name]

    if stale:
        save_allocations(allocations)

    return allocations


def find_free_ports(count: int, existing_allocations: dict) -> list:
    """Find N consecutive free ports that aren't already allocated."""
    allocated = set()
    for ports in existing_allocations.values():
        allocated.update(ports)

    for start in range(PORT_RANGE_START, PORT_RANGE_END - count):
        candidate_ports = list(range(start, start + count))
        # Skip if any port is already allocated to another worktree
        if any(p in allocated for p in candidate_ports):
            continue
        # Check if all ports are actually free on the system
        if all(is_port_free(p) for p in candidate_ports):
            return candidate_ports

    raise RuntimeError(f"Could not find {count} consecutive free ports in range {PORT_RANGE_START}-{PORT_RANGE_END}")


def main():
    if len(sys.argv) != 3:
        print("Usage: python3 allocate-ports.py <count> <worktree_name>", file=sys.stderr)
        sys.exit(1)

    count = int(sys.argv[1])
    worktree_name = sys.argv[2]

    allocations = load_allocations()
    allocations = cleanup_stale_allocations(allocations)

    # Check if this worktree already has ports allocated
    if worktree_name in allocations:
        # Return existing allocation
        ports = allocations[worktree_name]
        print(json.dumps({"ports": ports, "existing": True}))
        return

    ports = find_free_ports(count, allocations)
    allocations[worktree_name] = ports
    save_allocations(allocations)

    print(json.dumps({"ports": ports, "existing": False}))


if __name__ == "__main__":
    main()
