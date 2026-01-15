use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};
use crate::git;
use crate::ports::PortAllocations;

pub fn execute(json: bool, all: bool) -> Result<()> {
    // Clean up stale allocations
    let mut allocations = PortAllocations::load()?;
    let stale = allocations.cleanup_stale();
    if !stale.is_empty() {
        allocations.save()?;
    }

    // Find all worktrees
    let mut worktrees = find_all_worktrees()?;

    // Filter by current project unless --all is specified
    let current_project = if !all { get_current_project() } else { None };

    if let Some(ref project) = current_project {
        worktrees.retain(|wt| &wt.project_name == project);
    }

    if worktrees.is_empty() {
        if json {
            println!("[]");
        } else if current_project.is_some() {
            println!(
                "{}",
                "No worktrees found for this project. Use --all to see all worktrees.".dimmed()
            );
        } else {
            println!("{}", "No active worktrees found.".dimmed());
        }
        return Ok(());
    }

    if json {
        display_json(&worktrees)?;
    } else {
        display_table(&worktrees, current_project.is_some());
    }

    Ok(())
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

fn find_all_worktrees() -> Result<Vec<WorktreeState>> {
    let mut worktrees = Vec::new();
    let base_dir = paths::global_worktrees_dir()?;

    if !base_dir.exists() {
        return Ok(worktrees);
    }

    // Walk through all directories looking for state.json files
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

    // Sort by creation time (newest first)
    worktrees.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(worktrees)
}

fn display_table(worktrees: &[WorktreeState], filtered_by_project: bool) {
    // Group by project
    let mut by_project: HashMap<String, Vec<&WorktreeState>> = HashMap::new();
    for wt in worktrees {
        by_project
            .entry(wt.project_name.clone())
            .or_default()
            .push(wt);
    }

    let mut project_names: Vec<_> = by_project.keys().collect();
    project_names.sort();

    for project_name in project_names {
        println!("\n{}", project_name.bold().blue());
        println!("{}", "─".repeat(60).dimmed());

        if let Some(project_worktrees) = by_project.get(project_name) {
            for wt in project_worktrees {
                let port_range = if wt.ports.is_empty() {
                    "no ports".to_string()
                } else if wt.ports.len() == 1 {
                    format!("port {}", wt.ports[0])
                } else {
                    format!(
                        "ports {}-{}",
                        wt.ports.first().unwrap(),
                        wt.ports.last().unwrap()
                    )
                };

                // Show display name with directory if custom name is set
                let name_display = if wt.has_custom_name() {
                    format!("{} - {}", wt.effective_name().green(), wt.name.dimmed())
                } else {
                    wt.name.green().to_string()
                };

                println!(
                    "  {} {} {}",
                    name_display,
                    format!("({})", port_range).dimmed(),
                    format!("[{}]", wt.branch).cyan()
                );

                println!("    {} {}", "dir:".dimmed(), wt.worktree_dir.display());

                let created = wt.created_at.format("%Y-%m-%d %H:%M").to_string();
                println!("    {} {}", "created:".dimmed(), created.dimmed());
            }
        }
    }

    // Summary
    let worktree_count = worktrees.len();
    if filtered_by_project {
        println!(
            "\n{}",
            format!(
                "Total: {} worktree{}. Use --all to see all projects.",
                worktree_count,
                if worktree_count == 1 { "" } else { "s" }
            )
            .dimmed()
        );
    } else {
        let project_count = by_project.len();
        println!(
            "\n{}",
            format!(
                "Total: {} worktree{} across {} project{}",
                worktree_count,
                if worktree_count == 1 { "" } else { "s" },
                project_count,
                if project_count == 1 { "" } else { "s" }
            )
            .dimmed()
        );
    }
}

fn display_json(worktrees: &[WorktreeState]) -> Result<()> {
    let json = serde_json::to_string_pretty(worktrees)?;
    println!("{}", json);
    Ok(())
}
