use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if the current directory is inside a git repository
pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the root directory of the current git repository (or worktree)
pub fn get_repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        bail!("Not in a git repository");
    }

    let path = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(path))
}

/// Get the root directory of the main repository (not a worktree).
/// If currently in a worktree, this returns the main repository root.
/// If in the main repository, returns that root.
pub fn get_main_repo_root() -> Result<PathBuf> {
    // git worktree list outputs lines like:
    // /path/to/main/repo  abc1234 [main]
    // /path/to/worktree   def5678 [feature-branch]
    // The first entry is always the main working tree
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .context("Failed to execute git worktree list")?;

    if !output.status.success() {
        // Fallback to regular repo root if worktree list fails
        return get_repo_root();
    }

    let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;

    // Porcelain format: first line starts with "worktree " followed by path
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            return Ok(PathBuf::from(path));
        }
    }

    // Fallback if parsing fails
    get_repo_root()
}

/// Get the project name from the main repository root directory name
pub fn get_main_project_name() -> Result<String> {
    let root = get_main_repo_root()?;
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .context("Could not determine project name from repository root")?;
    Ok(name)
}

/// Check if a branch exists locally
pub fn branch_exists(branch: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a new git worktree
pub fn create_worktree(path: &Path, branch: &str) -> Result<()> {
    let path_str = path.to_str().context("Invalid path for worktree")?;

    let output = Command::new("git")
        .args(["worktree", "add", path_str, "-b", branch])
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree add failed: {}", stderr);
    }

    Ok(())
}

/// Remove a git worktree
pub fn remove_worktree(original_dir: &Path, worktree_dir: &Path, force: bool) -> Result<()> {
    let worktree_str = worktree_dir.to_str().context("Invalid worktree path")?;

    let mut args = vec!["worktree", "remove", worktree_str];
    if force {
        args.push("--force");
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(original_dir)
        .output()
        .context("Failed to execute git worktree remove")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If git remove fails, try manual removal
        if force {
            std::fs::remove_dir_all(worktree_dir).with_context(|| {
                format!("Failed to remove directory {}", worktree_dir.display())
            })?;
        } else {
            bail!("git worktree remove failed: {}", stderr);
        }
    }

    // Prune stale worktree entries
    let _ = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(original_dir)
        .output();

    Ok(())
}

/// Get the latest commit date in a worktree directory
pub fn get_latest_commit_date(worktree_dir: &Path) -> Result<DateTime<Utc>> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%aI"])
        .current_dir(worktree_dir)
        .output()
        .context("Failed to get latest commit date")?;

    if !output.status.success() {
        bail!("Failed to get latest commit date");
    }

    let date_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    DateTime::parse_from_rfc3339(&date_str)
        .map(|d| d.with_timezone(&Utc))
        .context("Failed to parse commit date")
}
