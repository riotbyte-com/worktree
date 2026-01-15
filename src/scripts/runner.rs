use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::config::state::WorktreeState;

/// Build environment variables for lifecycle scripts
pub fn build_env_vars(state: &WorktreeState) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("WORKTREE_NAME".to_string(), state.name.clone());
    env.insert(
        "WORKTREE_DISPLAY_NAME".to_string(),
        state.effective_name().to_string(),
    );
    env.insert("WORKTREE_PROJECT".to_string(), state.project_name.clone());
    env.insert(
        "WORKTREE_DIR".to_string(),
        state.worktree_dir.to_string_lossy().to_string(),
    );
    env.insert(
        "WORKTREE_ORIGINAL_DIR".to_string(),
        state.original_dir.to_string_lossy().to_string(),
    );
    env.insert(
        "WORKTREE_ALLOCATION_KEY".to_string(),
        state.allocation_key.clone(),
    );

    if let Some(param) = &state.param {
        env.insert("WORKTREE_PARAM".to_string(), param.clone());
    }

    // Add port environment variables
    for (i, port) in state.ports.iter().enumerate() {
        env.insert(format!("WORKTREE_PORT_{}", i), port.to_string());
    }

    env
}

/// Execute a lifecycle script with environment variables
pub fn execute_script(script: &Path, env: &HashMap<String, String>) -> Result<()> {
    if !script.exists() {
        bail!("Script not found: {}", script.display());
    }

    // Check if script is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script)?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            bail!(
                "Script is not executable: {}\nRun: chmod +x {}",
                script.display(),
                script.display()
            );
        }
    }

    let status = Command::new("bash")
        .arg(script)
        .envs(env)
        .current_dir(
            script
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new(".")),
        )
        .status()
        .with_context(|| format!("Failed to execute {}", script.display()))?;

    if !status.success() {
        bail!("Script exited with status: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}

/// Execute a lifecycle script, ignoring errors (for cleanup)
pub fn execute_script_ignore_errors(script: &Path, env: &HashMap<String, String>) -> bool {
    if !script.exists() {
        return false;
    }

    Command::new("bash")
        .arg(script)
        .envs(env)
        .current_dir(
            script
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new(".")),
        )
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
