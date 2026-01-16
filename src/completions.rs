use clap_complete::engine::CompletionCandidate;
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};
use crate::git;

/// Get worktree name completion candidates
/// Returns all worktree names, optionally filtered by current project
pub fn worktree_names() -> Vec<CompletionCandidate> {
    let worktrees = match find_all_worktrees() {
        Ok(wts) => wts,
        Err(_) => return vec![],
    };

    // Try to get the current project to filter results
    let current_project = get_current_project();

    worktrees
        .into_iter()
        .filter(|wt| {
            // If we know the current project, only show worktrees from that project
            current_project
                .as_ref()
                .map(|p| &wt.project_name == p)
                .unwrap_or(true)
        })
        .flat_map(|wt| {
            let mut candidates = vec![CompletionCandidate::new(&wt.name)
                .help(Some(format!("{} [{}]", wt.project_name, wt.branch).into()))];

            // Also add display name as a completion candidate if it differs
            if let Some(ref display_name) = wt.display_name {
                if display_name != &wt.name {
                    candidates.push(
                        CompletionCandidate::new(display_name)
                            .help(Some(format!("{} [{}]", wt.project_name, wt.branch).into())),
                    );
                }
            }

            candidates
        })
        .collect()
}

/// Find all worktrees across all projects
fn find_all_worktrees() -> anyhow::Result<Vec<WorktreeState>> {
    let mut worktrees = Vec::new();
    let base_dir = paths::global_worktrees_dir()?;

    if !base_dir.exists() {
        return Ok(worktrees);
    }

    for entry in WalkDir::new(&base_dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "state.json" {
            if let Ok(state) = WorktreeState::load(entry.path()) {
                worktrees.push(state);
            }
        }
    }

    Ok(worktrees)
}

/// Try to get the current project name from the git repo or worktree state
fn get_current_project() -> Option<String> {
    // First check if we're inside a worktree
    if let Ok(Some(state)) = crate::config::state::detect_worktree() {
        return Some(state.project_name);
    }

    // Otherwise try to get the project name from git
    if git::is_git_repo() {
        if let Ok(name) = git::get_main_project_name() {
            return Some(name);
        }
    }

    None
}
