use anyhow::{bail, Result};
use colored::Colorize;
use std::io::{self, Write};
use walkdir::WalkDir;

use super::common::{self, RemoveOptions};
use crate::config::{paths, state::WorktreeState};
use crate::git;
use crate::terminal;

pub fn execute(name: Option<String>, force: bool, interactive: bool) -> Result<()> {
    // Determine which worktree to close
    let worktree_state = resolve_worktree(name, interactive)?;

    // Show display name with directory if custom name is set
    let name_display = if worktree_state.has_custom_name() {
        format!(
            "{} - {}",
            worktree_state.effective_name().green(),
            worktree_state.name.dimmed()
        )
    } else {
        worktree_state.name.green().to_string()
    };

    println!(
        "{} {}/{}",
        "Closing:".bold(),
        worktree_state.project_name.blue(),
        name_display
    );
    println!(
        "  {} {}",
        "Path:".dimmed(),
        worktree_state.worktree_dir.display()
    );

    // Confirm unless force flag is set
    if !force {
        print!(
            "\n{} ",
            "Are you sure you want to close this worktree? (y/N):".yellow()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    println!();

    // Kill tmux session if it exists (try effective name first, then directory name)
    println!("  Checking for tmux session...");
    let effective_name = worktree_state.effective_name();
    kill_tmux_session_by_name(
        &worktree_state.project_name,
        effective_name,
        &worktree_state.name,
    );

    // Change to home directory before removing worktree
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
    std::env::set_current_dir(&home)?;

    // Use shared worktree removal logic with verbose output
    let result = common::remove_worktree(&worktree_state, &RemoveOptions { verbose: true })?;

    // Print detailed results
    if let Some(success) = result.close_script_success {
        if success {
            println!("  {} Close script completed", "✓".green());
        } else {
            println!("  {} Close script failed (continuing anyway)", "⚠".yellow());
        }
    }

    match result.deallocated_ports {
        Some(ports) if !ports.is_empty() => {
            println!(
                "  {} Deallocated ports {}-{}",
                "✓".green(),
                ports.first().unwrap_or(&0),
                ports.last().unwrap_or(&0)
            );
        }
        Some(_) | None => {
            println!("  {} No ports were allocated", "⚠".yellow());
        }
    }

    if result.worktree_removed {
        println!("  {} Git worktree removed", "✓".green());
    } else {
        println!("  {} Failed to remove worktree", "✗".red());
    }

    println!();
    println!("{}", "Worktree closed successfully!".green().bold());
    println!();
    println!(
        "Return to original project:\n  cd {}",
        worktree_state.original_dir.display()
    );

    Ok(())
}

/// Resolve which worktree to close based on arguments
fn resolve_worktree(name: Option<String>, interactive: bool) -> Result<WorktreeState> {
    // If interactive mode, show selection (filtered by current project)
    if interactive {
        let worktrees = find_worktrees_for_current_project()?;
        if worktrees.is_empty() {
            bail!("No worktrees found for this project.");
        }
        return select_worktree(&worktrees);
    }

    // If name provided, find by name (search current project first, then all)
    if let Some(name) = name {
        // First try current project
        let project_worktrees = find_worktrees_for_current_project()?;
        let matches: Vec<_> = project_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&name))
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
            .filter(|wt| wt.matches_identifier(&name))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", name),
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
    println!("\n{}", "Select worktree to close:".bold());

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

/// Kill tmux session by trying effective name first, then falling back to directory name
fn kill_tmux_session_by_name(project_name: &str, effective_name: &str, directory_name: &str) {
    // Try to kill session with effective name first
    let effective_session = terminal::tmux_session_name(project_name, effective_name);
    if terminal::tmux_session_exists(&effective_session) {
        match terminal::kill_tmux_session(project_name, effective_name) {
            Ok(true) => {
                println!(
                    "  {} Terminated tmux session: {}",
                    "✓".green(),
                    effective_session
                );
                return;
            }
            Ok(false) => {}
            Err(e) => {
                println!("  {} Failed to kill tmux session: {}", "⚠".yellow(), e);
                return;
            }
        }
    }

    // Fall back to directory name (for sessions created before display name support)
    if effective_name != directory_name {
        let directory_session = terminal::tmux_session_name(project_name, directory_name);
        if terminal::tmux_session_exists(&directory_session) {
            match terminal::kill_tmux_session(project_name, directory_name) {
                Ok(true) => {
                    println!(
                        "  {} Terminated tmux session: {}",
                        "✓".green(),
                        directory_session
                    );
                    return;
                }
                Ok(false) => {}
                Err(e) => {
                    println!("  {} Failed to kill tmux session: {}", "⚠".yellow(), e);
                    return;
                }
            }
        }
    }

    println!("  {} No tmux session found", "·".dimmed());
}
