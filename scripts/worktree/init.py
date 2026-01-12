#!/usr/bin/env python3
"""
Checks prerequisites and initializes worktree configuration for a project.
Usage: python3 init.py [--check-only]

Options:
  --check-only    Only check prerequisites, don't initialize

Exit codes:
  0 - Success
  1 - Prerequisites missing
  2 - Already initialized (when not --check-only)
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def check_command(cmd: str) -> tuple[bool, str]:
    """Check if a command is available."""
    path = shutil.which(cmd)
    if path:
        return True, path
    return False, ""


def get_version(cmd: str, version_flag: str = "--version") -> str:
    """Get version string for a command."""
    try:
        result = subprocess.run(
            [cmd, version_flag],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.stdout.strip().split('\n')[0] or result.stderr.strip().split('\n')[0]
    except Exception:
        return "unknown version"


def check_prerequisites() -> tuple[bool, list[dict]]:
    """Check all prerequisites and return status."""
    prerequisites = []
    all_ok = True

    # Python 3
    py_ok, py_path = check_command("python3")
    prerequisites.append({
        "name": "Python 3",
        "command": "python3",
        "ok": py_ok,
        "path": py_path,
        "version": get_version("python3") if py_ok else None,
        "required": True,
        "reason": "Required for port allocation and helper scripts"
    })
    if not py_ok:
        all_ok = False

    # Git
    git_ok, git_path = check_command("git")
    prerequisites.append({
        "name": "Git",
        "command": "git",
        "ok": git_ok,
        "path": git_path,
        "version": get_version("git") if git_ok else None,
        "required": True,
        "reason": "Required for git worktree management"
    })
    if not git_ok:
        all_ok = False

    # Claude CLI
    claude_ok, claude_path = check_command("claude")
    prerequisites.append({
        "name": "Claude CLI",
        "command": "claude",
        "ok": claude_ok,
        "path": claude_path,
        "version": get_version("claude", "--version") if claude_ok else None,
        "required": True,
        "reason": "Required to run Claude in new worktree"
    })
    if not claude_ok:
        all_ok = False

    # Bash
    bash_ok, bash_path = check_command("bash")
    prerequisites.append({
        "name": "Bash",
        "command": "bash",
        "ok": bash_ok,
        "path": bash_path,
        "version": get_version("bash") if bash_ok else None,
        "required": True,
        "reason": "Required for shell scripts"
    })
    if not bash_ok:
        all_ok = False

    return all_ok, prerequisites


def print_prerequisites(prerequisites: list[dict]) -> None:
    """Print prerequisites status."""
    print("\nPrerequisites check:")
    print("-" * 60)

    for prereq in prerequisites:
        status = "✓" if prereq["ok"] else "✗"
        required = "(required)" if prereq["required"] else "(optional)"

        if prereq["ok"]:
            print(f"  {status} {prereq['name']} {required}")
            print(f"      {prereq['version']}")
            print(f"      Path: {prereq['path']}")
        else:
            print(f"  {status} {prereq['name']} {required} - NOT FOUND")
            print(f"      {prereq['reason']}")
        print()


def is_initialized(project_dir: Path) -> bool:
    """Check if worktree is already initialized for this project."""
    config_dir = project_dir / ".claude" / "worktree"
    settings_file = config_dir / "settings.json"
    return settings_file.exists()


def get_existing_settings(project_dir: Path) -> dict:
    """Load existing settings if they exist."""
    settings_file = project_dir / ".claude" / "worktree" / "settings.json"
    local_settings_file = project_dir / ".claude" / "worktree" / "settings.local.json"

    settings = {}
    if settings_file.exists():
        settings = json.loads(settings_file.read_text())
    if local_settings_file.exists():
        local = json.loads(local_settings_file.read_text())
        settings.update(local)

    return settings


def main():
    check_only = "--check-only" in sys.argv

    # Check prerequisites
    all_ok, prerequisites = check_prerequisites()
    print_prerequisites(prerequisites)

    if not all_ok:
        print("ERROR: Some required prerequisites are missing.")
        print("Please install the missing tools and try again.")
        sys.exit(1)

    print("All prerequisites satisfied!")

    if check_only:
        sys.exit(0)

    # Check if we're in a git repository
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=True
        )
        project_dir = Path(result.stdout.strip())
    except subprocess.CalledProcessError:
        print("\nERROR: Not in a git repository.")
        print("Please run this command from within a git repository.")
        sys.exit(1)

    print(f"\nProject root: {project_dir}")

    # Check if already initialized
    if is_initialized(project_dir):
        existing = get_existing_settings(project_dir)
        print("\nWARNING: Worktree is already initialized for this project!")
        print(f"  Config dir: {project_dir / '.claude' / 'worktree'}")
        if existing:
            print(f"  Current settings: {json.dumps(existing, indent=2)}")
        print("\nTo reinitialize, delete the .claude/worktree directory first.")
        sys.exit(2)

    # Output info for Claude to use when prompting the user
    default_worktree_dir = Path.home() / ".claude" / "worktrees"

    output = {
        "status": "ready_to_initialize",
        "project_dir": str(project_dir),
        "config_dir": str(project_dir / ".claude" / "worktree"),
        "defaults": {
            "worktree_dir": str(default_worktree_dir),
            "port_count": 10,
            "port_range_start": 50000,
            "port_range_end": 60000,
            "branch_prefix": "worktree/",
            "auto_launch_terminal": True
        }
    }

    print("\n" + json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
