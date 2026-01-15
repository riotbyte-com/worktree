use anyhow::{Context, Result};
use std::path::PathBuf;

/// Returns the user config directory (~/.config/worktree/)
pub fn user_config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .context(
            "Could not determine home directory. Please ensure HOME environment variable is set.",
        )
        .map(|p| p.join(".config").join("worktree"))
}

/// Returns the user config file path (~/.config/worktree/config.json)
pub fn user_config_file() -> Result<PathBuf> {
    Ok(user_config_dir()?.join("config.json"))
}

/// Ensures the user config directory exists
pub fn ensure_user_config_dir() -> Result<()> {
    std::fs::create_dir_all(user_config_dir()?)?;
    Ok(())
}

/// Returns the global worktree directory (~/.worktree/)
pub fn global_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .context(
            "Could not determine home directory. Please ensure HOME environment variable is set.",
        )
        .map(|p| p.join(".worktree"))
}

/// Returns the global worktrees storage directory (~/.worktree/worktrees/)
pub fn global_worktrees_dir() -> Result<PathBuf> {
    Ok(global_dir()?.join("worktrees"))
}

/// Returns the port allocations file path (~/.worktree/port-allocations.json)
pub fn allocations_file() -> Result<PathBuf> {
    Ok(global_dir()?.join("port-allocations.json"))
}

/// Returns the project config directory relative to a given root
pub fn project_config_dir_in(root: &std::path::Path) -> PathBuf {
    root.join(".worktree")
}

/// Returns the settings file path relative to a given root
pub fn settings_file_in(root: &std::path::Path) -> PathBuf {
    project_config_dir_in(root).join("settings.json")
}

/// Returns the local settings file path relative to a given root
pub fn local_settings_file_in(root: &std::path::Path) -> PathBuf {
    project_config_dir_in(root).join("settings.local.json")
}

/// Ensures the global directory exists
pub fn ensure_global_dir() -> Result<()> {
    std::fs::create_dir_all(global_dir()?)?;
    Ok(())
}
