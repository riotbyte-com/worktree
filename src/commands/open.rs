use anyhow::{bail, Result};
use colored::Colorize;
use std::io::{self, Write};
use walkdir::WalkDir;

use crate::config::{paths, settings::MergedSettings, state::WorktreeState};
use crate::terminal;

pub fn execute(name: Option<String>, interactive: bool) -> Result<()> {
    // Determine which worktree to open
    let worktree_state = resolve_worktree(name, interactive)?;

    println!(
        "{} {}/{}",
        "Opening:".bold(),
        worktree_state.project_name.blue(),
        worktree_state.name.green()
    );
    println!(
        "  {} {}",
        "Path:".dimmed(),
        worktree_state.worktree_dir.display()
    );

    // Load settings from the original project directory to get terminal preference
    let settings = MergedSettings::load_from(&worktree_state.original_dir).unwrap_or_else(|_| {
        MergedSettings {
            port_count: 10,
            port_range_start: 50000,
            port_range_end: 60000,
            branch_prefix: "worktree/".to_string(),
            auto_launch_terminal: true,
            worktree_dir: None,
            terminal: None,
        }
    });

    // Get terminal from settings or auto-detect
    let term = settings
        .terminal
        .as_ref()
        .and_then(|t| terminal::Terminal::from_str(t))
        .or_else(terminal::detect_terminal);

    if let Some(term) = term {
        println!();
        println!("  Launching {}...", term.name());
        let launch_result = if term == terminal::Terminal::Tmux {
            terminal::launch_tmux_session(
                &worktree_state.project_name,
                &worktree_state.name,
                &worktree_state.worktree_dir,
            )
        } else {
            terminal::launch(&term, &worktree_state.worktree_dir)
        };

        if let Err(e) = launch_result {
            println!("  {} Failed to launch terminal: {}", "⚠".yellow(), e);
            println!(
                "  Run manually: {}",
                terminal::get_manual_command(&worktree_state.worktree_dir).dimmed()
            );
        }
    } else {
        println!(
            "\n  No terminal detected. Run manually:\n  {}",
            terminal::get_manual_command(&worktree_state.worktree_dir).dimmed()
        );
    }

    Ok(())
}

/// Resolve which worktree to open based on arguments
fn resolve_worktree(name: Option<String>, interactive: bool) -> Result<WorktreeState> {
    // If interactive mode, show selection
    if interactive {
        let worktrees = find_all_worktrees()?;
        if worktrees.is_empty() {
            bail!("No worktrees found.");
        }
        return select_worktree(&worktrees);
    }

    // If name provided, find by name
    if let Some(name) = name {
        let worktrees = find_all_worktrees()?;
        let matches: Vec<_> = worktrees
            .into_iter()
            .filter(|wt| wt.name == name || wt.allocation_key.ends_with(&format!("/{}", name)))
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

    // Not in a worktree - show interactive selection
    let worktrees = find_all_worktrees()?;
    if worktrees.is_empty() {
        bail!("No worktrees found.");
    }
    select_worktree(&worktrees)
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
    println!("\n{}", "Select worktree to open:".bold());

    for (i, wt) in worktrees.iter().enumerate() {
        let port_range = if wt.ports.is_empty() {
            "no ports".to_string()
        } else {
            format!("{}-{}", wt.ports.first().unwrap(), wt.ports.last().unwrap())
        };

        println!(
            "  {}) {}/{} {} {}",
            (i + 1).to_string().cyan(),
            wt.project_name.blue(),
            wt.name.green(),
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
