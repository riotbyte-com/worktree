use anyhow::{bail, Result};
use colored::Colorize;
use std::process;
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};

pub fn execute(name: Option<String>) -> Result<()> {
    let worktree_state = match resolve_worktree(name)? {
        Some(state) => state,
        None => {
            eprintln!("{}", "Error: Not in a worktree directory".red());
            process::exit(1);
        }
    };

    display_status(&worktree_state);
    Ok(())
}

/// Resolve which worktree to show status for
fn resolve_worktree(name: Option<String>) -> Result<Option<WorktreeState>> {
    // If name provided, find by name
    if let Some(name) = name {
        let all_worktrees = find_all_worktrees()?;
        let matches: Vec<_> = all_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&name))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", name),
            1 => return Ok(Some(matches.into_iter().next().unwrap())),
            _ => bail!(
                "Multiple worktrees match '{}'. Please be more specific.",
                name
            ),
        }
    }

    // Try to detect current worktree
    crate::config::state::detect_worktree()
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

    Ok(worktrees)
}

/// Display the status of a worktree
fn display_status(state: &WorktreeState) {
    // Worktree name (show both display name and directory name if different)
    if state.has_custom_name() {
        println!(
            "{} {} ({})",
            "Worktree:".bold(),
            state.effective_name().green(),
            state.name.dimmed()
        );
    } else {
        println!("{} {}", "Worktree:".bold(), state.name.green());
    }

    println!("{} {}", "Branch:  ".bold(), state.branch.cyan());
    println!("{} {}", "Project: ".bold(), state.project_name.blue());

    println!();
    println!("{}", "Directories:".bold());
    println!(
        "  {} {}",
        "Original:".dimmed(),
        state.original_dir.display()
    );
    println!(
        "  {} {}",
        "Worktree:".dimmed(),
        state.worktree_dir.display()
    );

    println!();
    if state.ports.is_empty() {
        println!("{} {}", "Ports:".bold(), "none".dimmed());
    } else {
        let ports_str = state
            .ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("{} {}", "Ports:".bold(), ports_str);
    }

    println!();
    let created = state.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{} {}", "Created:".bold(), created);
}
