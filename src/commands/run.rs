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
        "Running:".bold(),
        worktree_state.project_name.blue(),
        worktree_state.name.green()
    );

    // Find run script
    let run_script = worktree_state.worktree_dir.join(".worktree").join("run.sh");

    if !run_script.exists() {
        bail!(
            "Run script not found at {}\nCreate a run.sh script to start your development environment.",
            run_script.display()
        );
    }

    // Build environment and execute
    let env = scripts::build_env_vars(&worktree_state);

    println!(
        "  Ports: {}-{}",
        worktree_state.ports.first().unwrap_or(&0),
        worktree_state.ports.last().unwrap_or(&0)
    );
    println!();

    scripts::execute_script(&run_script, &env)?;

    Ok(())
}
