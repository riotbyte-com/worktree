use anyhow::{bail, Result};
use colored::Colorize;
use std::io::{self, Write};
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};
use crate::git;
use crate::terminal;

pub fn execute(new_name: Option<String>, worktree: Option<String>, clear: bool) -> Result<()> {
    // Resolve which worktree to rename
    let mut state = resolve_worktree(worktree)?;

    // Handle --clear flag
    if clear {
        return clear_display_name(&mut state);
    }

    // Get new name (prompt if not provided)
    let new_name = match new_name {
        Some(name) => name,
        None => prompt_for_name(&state)?,
    };

    // Empty input means clear
    if new_name.is_empty() {
        return clear_display_name(&mut state);
    }

    // Validate and set the new name
    validate_name(&new_name)?;
    check_name_conflicts(&new_name, &state)?;

    let old_name = state.effective_name().to_string();
    state.display_name = Some(new_name.clone());
    state.save()?;

    println!(
        "{} Renamed worktree from '{}' to '{}'",
        "✓".green(),
        old_name.yellow(),
        new_name.green()
    );
    println!("  {} {}", "Directory:".dimmed(), state.name.dimmed());

    // Rename tmux session if it exists
    rename_tmux_session_if_exists(&state.project_name, &old_name, &new_name, &state.name)?;

    Ok(())
}

/// Clear the custom display name
fn clear_display_name(state: &mut WorktreeState) -> Result<()> {
    if state.display_name.is_none() {
        println!(
            "{} Worktree '{}' has no custom name to clear.",
            "ℹ".blue(),
            state.name.green()
        );
        return Ok(());
    }

    let old_name = state.effective_name().to_string();
    let new_name = state.name.clone();
    state.display_name = None;
    state.save()?;

    println!(
        "{} Cleared custom name '{}', reverted to '{}'",
        "✓".green(),
        old_name.yellow(),
        state.name.green()
    );

    // Rename tmux session if it exists
    rename_tmux_session_if_exists(&state.project_name, &old_name, &new_name, &state.name)?;

    Ok(())
}

/// Prompt user for new name
fn prompt_for_name(state: &WorktreeState) -> Result<String> {
    println!(
        "\n{} {}/{}",
        "Renaming:".bold(),
        state.project_name.blue(),
        if state.has_custom_name() {
            format!(
                "{} - {}",
                state.effective_name().green(),
                state.name.dimmed()
            )
        } else {
            state.name.green().to_string()
        }
    );

    print!("\n{} ", "Enter new name (empty to clear):".bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Validate the new name
fn validate_name(name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') {
        bail!("Name cannot contain path separators (/ or \\)");
    }
    if name.len() > 64 {
        bail!("Name is too long (max 64 characters)");
    }
    Ok(())
}

/// Check for name conflicts with other worktrees
fn check_name_conflicts(new_name: &str, current: &WorktreeState) -> Result<()> {
    let worktrees = find_all_worktrees()?;
    for wt in worktrees {
        // Skip the current worktree
        if wt.worktree_dir == current.worktree_dir {
            continue;
        }
        // Check for conflicts with name or display_name
        if wt.name == new_name || wt.display_name.as_deref() == Some(new_name) {
            bail!(
                "Name '{}' conflicts with existing worktree '{}' in project '{}'",
                new_name,
                wt.effective_name(),
                wt.project_name
            );
        }
    }
    Ok(())
}

/// Resolve which worktree to rename based on arguments
fn resolve_worktree(identifier: Option<String>) -> Result<WorktreeState> {
    // If identifier provided, find by name (search current project first, then all)
    if let Some(id) = identifier {
        // First try current project
        let project_worktrees = find_worktrees_for_current_project()?;
        let matches: Vec<_> = project_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&id))
            .collect();

        if matches.len() == 1 {
            return Ok(matches.into_iter().next().unwrap());
        }
        if matches.len() > 1 {
            println!("{}", "Multiple worktrees match that name:".yellow());
            return select_worktree(&matches);
        }

        // If not found in current project, search all worktrees
        let all_worktrees = find_all_worktrees()?;
        let matches: Vec<_> = all_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&id))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", id),
            1 => return Ok(matches.into_iter().next().unwrap()),
            _ => {
                println!("{}", "Multiple worktrees match that name:".yellow());
                return select_worktree(&matches);
            }
        }
    }

    // Try to detect current worktree
    if let Some(state) = crate::config::state::detect_worktree()? {
        return Ok(state);
    }

    // Not in a worktree - show interactive selection (filtered by current project)
    let worktrees = find_worktrees_for_current_project()?;
    if worktrees.is_empty() {
        bail!("No worktrees found for this project.");
    }
    select_worktree(&worktrees)
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

/// Find worktrees for the current project, or all if not in a project
fn find_worktrees_for_current_project() -> Result<Vec<WorktreeState>> {
    let mut worktrees = find_all_worktrees()?;

    if let Some(project) = get_current_project() {
        worktrees.retain(|wt| wt.project_name == project);
    }

    Ok(worktrees)
}

/// Find all worktrees in the global directory
fn find_all_worktrees() -> Result<Vec<WorktreeState>> {
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

    worktrees.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(worktrees)
}

/// Interactive worktree selection
fn select_worktree(worktrees: &[WorktreeState]) -> Result<WorktreeState> {
    println!("\n{}", "Select worktree to rename:".bold());

    for (i, wt) in worktrees.iter().enumerate() {
        let port_range = if wt.ports.is_empty() {
            "no ports".to_string()
        } else {
            format!("{}-{}", wt.ports.first().unwrap(), wt.ports.last().unwrap())
        };

        // Show display name with directory if custom name is set
        let name_display = if wt.has_custom_name() {
            format!("{} - {}", wt.effective_name().green(), wt.name.dimmed())
        } else {
            wt.name.green().to_string()
        };

        println!(
            "  {}) {}/{} {} {}",
            (i + 1).to_string().cyan(),
            wt.project_name.blue(),
            name_display,
            format!("(ports {})", port_range).dimmed(),
            format!("[{}]", wt.branch).dimmed()
        );
    }

    print!("\n{} ", "Enter number:".bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        bail!("No selection made.");
    }

    let idx: usize = input
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number: {}", input))?;

    if idx == 0 || idx > worktrees.len() {
        bail!("Invalid selection: {}. Choose 1-{}", idx, worktrees.len());
    }

    Ok(worktrees[idx - 1].clone())
}

/// Rename the tmux session if it exists
/// Tries to find session by old_effective_name first, then falls back to directory_name
fn rename_tmux_session_if_exists(
    project_name: &str,
    old_effective_name: &str,
    new_effective_name: &str,
    directory_name: &str,
) -> Result<()> {
    let old_session_name = terminal::tmux_session_name(project_name, old_effective_name);
    let new_session_name = terminal::tmux_session_name(project_name, new_effective_name);

    // Try to rename session with old effective name
    if terminal::tmux_session_exists(&old_session_name) {
        match terminal::rename_tmux_session(&old_session_name, &new_session_name) {
            Ok(true) => {
                println!(
                    "  {} Renamed tmux session to '{}'",
                    "✓".green(),
                    new_session_name
                );
            }
            Ok(false) => {}
            Err(e) => {
                println!("  {} Failed to rename tmux session: {}", "⚠".yellow(), e);
            }
        }
        return Ok(());
    }

    // Fall back to directory name (for sessions created before display name support)
    if old_effective_name != directory_name {
        let fallback_session_name = terminal::tmux_session_name(project_name, directory_name);
        if terminal::tmux_session_exists(&fallback_session_name) {
            match terminal::rename_tmux_session(&fallback_session_name, &new_session_name) {
                Ok(true) => {
                    println!(
                        "  {} Renamed tmux session to '{}'",
                        "✓".green(),
                        new_session_name
                    );
                }
                Ok(false) => {}
                Err(e) => {
                    println!("  {} Failed to rename tmux session: {}", "⚠".yellow(), e);
                }
            }
        }
    }

    Ok(())
}
