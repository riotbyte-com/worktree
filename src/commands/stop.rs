use anyhow::{bail, Result};
use colored::Colorize;

use crate::config::state;
use crate::scripts;

pub fn execute() -> Result<()> {
    // Detect if we're in a worktree
    let worktree_state = state::detect_worktree()?.ok_or_else(|| {
        anyhow::anyhow!("Not in a worktree. Run this command from within a worktree directory.")
    })?;

    println!(
        "{} {}/{}",
        "Stopping:".bold(),
        worktree_state.project_name.blue(),
        worktree_state.name.green()
    );

    // Find stop script
    let stop_script = worktree_state
        .worktree_dir
        .join(".worktree")
        .join("stop.sh");

    if !stop_script.exists() {
        bail!(
            "Stop script not found at {}\nCreate a stop.sh script to stop your services.",
            stop_script.display()
        );
    }

    // Build environment and execute
    let env = scripts::build_env_vars(&worktree_state);

    println!();
    scripts::execute_script(&stop_script, &env)?;

    println!();
    println!("{}", "Services stopped.".green());

    Ok(())
}
