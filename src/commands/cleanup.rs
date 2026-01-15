use anyhow::{bail, Result};
use chrono::Utc;
use colored::Colorize;
use std::io::{self, Write};
use walkdir::WalkDir;

use super::common::{self, RemoveOptions};
use crate::config::{paths, state::WorktreeState};
use crate::git;

/// Worktree with activity information
struct WorktreeInfo {
    state: WorktreeState,
    last_commit: Option<chrono::DateTime<Utc>>,
    days_inactive: i64,
}

pub fn execute(older_than: Option<u32>, force: bool) -> Result<()> {
    // Find all worktrees and get their activity info
    let mut worktrees = find_worktrees_with_activity()?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found.".dimmed());
        return Ok(());
    }

    // Filter by age if specified
    if let Some(days) = older_than {
        worktrees.retain(|wt| wt.days_inactive >= days as i64);
        if worktrees.is_empty() {
            println!(
                "{}",
                format!("No worktrees older than {} days.", days).dimmed()
            );
            return Ok(());
        }
    }

    // Display worktrees with activity info
    display_worktrees(&worktrees);

    // Prompt for selection
    let selected = prompt_selection(&worktrees)?;
    if selected.is_empty() {
        println!("{}", "No worktrees selected.".dimmed());
        return Ok(());
    }

    // Confirm deletion
    if !force {
        println!();
        println!(
            "{} {} worktree{}:",
            "Will delete".yellow().bold(),
            selected.len(),
            if selected.len() == 1 { "" } else { "s" }
        );
        for idx in &selected {
            let wt = &worktrees[*idx];
            // Show display name with directory if custom name is set
            let name_display = if wt.state.has_custom_name() {
                format!(
                    "{} - {}",
                    wt.state.effective_name().green(),
                    wt.state.name.dimmed()
                )
            } else {
                wt.state.name.green().to_string()
            };
            println!("  • {}/{}", wt.state.project_name.blue(), name_display);
        }

        print!("\n{} ", "Proceed? (y/N):".yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    // Delete selected worktrees
    println!();
    for idx in selected {
        let wt = &worktrees[idx];
        delete_worktree(&wt.state)?;
    }

    println!();
    println!("{}", "Cleanup complete!".green().bold());

    Ok(())
}

fn find_worktrees_with_activity() -> Result<Vec<WorktreeInfo>> {
    let mut result = Vec::new();
    let base_dir = paths::global_worktrees_dir()?;

    if !base_dir.exists() {
        return Ok(result);
    }

    let now = Utc::now();

    for entry in WalkDir::new(&base_dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "state.json" {
            if let Ok(state) = WorktreeState::load(entry.path()) {
                let last_commit = git::get_latest_commit_date(&state.worktree_dir).ok();

                let days_inactive = if let Some(commit_date) = last_commit {
                    (now - commit_date).num_days()
                } else {
                    // If no commit info, use creation date
                    (now - state.created_at).num_days()
                };

                result.push(WorktreeInfo {
                    state,
                    last_commit,
                    days_inactive,
                });
            }
        }
    }

    // Sort by days inactive (most inactive first)
    result.sort_by(|a, b| b.days_inactive.cmp(&a.days_inactive));

    Ok(result)
}

fn display_worktrees(worktrees: &[WorktreeInfo]) {
    println!("\n{}", "Worktrees (sorted by inactivity):".bold());
    println!("{}", "─".repeat(80).dimmed());

    println!(
        "  {:>3}  {:20} {:25} {:>12} {:>12}",
        "#".dimmed(),
        "Project".dimmed(),
        "Name".dimmed(),
        "Last Commit".dimmed(),
        "Inactive".dimmed()
    );
    println!("{}", "─".repeat(80).dimmed());

    for (i, wt) in worktrees.iter().enumerate() {
        let last_commit_str = wt
            .last_commit
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let inactive_str = format_inactive_days(wt.days_inactive);
        let inactive_colored = if wt.days_inactive > 30 {
            inactive_str.red()
        } else if wt.days_inactive > 7 {
            inactive_str.yellow()
        } else {
            inactive_str.normal()
        };

        // Show display name with directory if custom name is set
        let name_str = if wt.state.has_custom_name() {
            format!("{} ({})", wt.state.effective_name(), wt.state.name)
        } else {
            wt.state.name.clone()
        };

        println!(
            "  {:>3}  {:20} {:25} {:>12} {:>12}",
            (i + 1).to_string().cyan(),
            truncate(&wt.state.project_name, 20).blue(),
            truncate(&name_str, 25).green(),
            last_commit_str.dimmed(),
            inactive_colored
        );
    }

    println!("{}", "─".repeat(80).dimmed());
}

fn format_inactive_days(days: i64) -> String {
    if days == 0 {
        "today".to_string()
    } else if days == 1 {
        "1 day".to_string()
    } else if days < 7 {
        format!("{} days", days)
    } else if days < 30 {
        format!("{} weeks", days / 7)
    } else {
        format!("{} months", days / 30)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn prompt_selection(worktrees: &[WorktreeInfo]) -> Result<Vec<usize>> {
    println!();
    println!(
        "Enter numbers to delete (comma-separated), '{}' for all, or press Enter to cancel:",
        "all".cyan()
    );
    print!("{} ", ">".bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(vec![]);
    }

    if input.to_lowercase() == "all" {
        return Ok((0..worktrees.len()).collect());
    }

    let mut selected = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Check for range (e.g., "1-5")
        if part.contains('-') {
            let parts: Vec<&str> = part.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (
                    parts[0].trim().parse::<usize>(),
                    parts[1].trim().parse::<usize>(),
                ) {
                    if start > 0 && end <= worktrees.len() && start <= end {
                        for i in start..=end {
                            if !selected.contains(&(i - 1)) {
                                selected.push(i - 1);
                            }
                        }
                        continue;
                    }
                }
            }
            bail!("Invalid range: {}", part);
        }

        let idx: usize = part
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid number: {}", part))?;

        if idx == 0 || idx > worktrees.len() {
            bail!("Invalid selection: {}. Choose 1-{}", idx, worktrees.len());
        }

        if !selected.contains(&(idx - 1)) {
            selected.push(idx - 1);
        }
    }

    Ok(selected)
}

fn delete_worktree(state: &WorktreeState) -> Result<()> {
    // Show display name with directory if custom name is set
    let name_display = if state.has_custom_name() {
        format!(
            "{} - {}",
            state.effective_name().green(),
            state.name.dimmed()
        )
    } else {
        state.name.green().to_string()
    };
    println!(
        "  Deleting {}/{}...",
        state.project_name.blue(),
        name_display
    );

    let result = common::remove_worktree(state, &RemoveOptions::default())?;

    if result.worktree_removed {
        println!("    {} Removed", "✓".green());
    } else {
        println!("    {} Failed to remove", "✗".red());
    }

    Ok(())
}
