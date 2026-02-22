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

    println!("{}", worktree_state.worktree_dir.display());
    Ok(())
}

fn resolve_worktree(name: Option<String>) -> Result<Option<WorktreeState>> {
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

    crate::config::state::detect_worktree()
}

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
