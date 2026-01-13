use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::paths;

/// Team-shared settings (committed to repo)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_port_count")]
    pub port_count: u16,

    #[serde(default = "default_port_range_start")]
    pub port_range_start: u16,

    #[serde(default = "default_port_range_end")]
    pub port_range_end: u16,

    #[serde(default = "default_branch_prefix")]
    pub branch_prefix: String,

    #[serde(default = "default_auto_launch_terminal")]
    pub auto_launch_terminal: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port_count: default_port_count(),
            port_range_start: default_port_range_start(),
            port_range_end: default_port_range_end(),
            branch_prefix: default_branch_prefix(),
            auto_launch_terminal: default_auto_launch_terminal(),
        }
    }
}

fn default_port_count() -> u16 {
    10
}
fn default_port_range_start() -> u16 {
    50000
}
fn default_port_range_end() -> u16 {
    60000
}
fn default_branch_prefix() -> String {
    "worktree/".to_string()
}
fn default_auto_launch_terminal() -> bool {
    true
}

/// Personal settings (gitignored)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_dir: Option<PathBuf>,
}

/// Merged settings for runtime use
#[derive(Debug, Clone)]
pub struct MergedSettings {
    pub port_count: u16,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub branch_prefix: String,
    pub auto_launch_terminal: bool,
    pub worktree_dir: Option<PathBuf>,
}

impl MergedSettings {
    /// Load and merge settings from a specific root directory
    pub fn load_from(root: &Path) -> Result<Self> {
        let settings_path = paths::settings_file_in(root);
        let local_settings_path = paths::local_settings_file_in(root);

        let settings: Settings = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)
                .with_context(|| format!("Failed to read {}", settings_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", settings_path.display()))?
        } else {
            Settings::default()
        };

        let local_settings: LocalSettings = if local_settings_path.exists() {
            let content = std::fs::read_to_string(&local_settings_path)
                .with_context(|| format!("Failed to read {}", local_settings_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", local_settings_path.display()))?
        } else {
            LocalSettings::default()
        };

        Ok(Self {
            port_count: settings.port_count,
            port_range_start: settings.port_range_start,
            port_range_end: settings.port_range_end,
            branch_prefix: settings.branch_prefix,
            auto_launch_terminal: settings.auto_launch_terminal,
            worktree_dir: local_settings.worktree_dir,
        })
    }

    /// Get the worktree directory for a project
    /// Uses custom dir if set, otherwise defaults to ~/.worktree/worktrees/<project>/
    pub fn get_worktree_base_dir(&self, project_name: &str) -> Result<PathBuf> {
        match &self.worktree_dir {
            Some(dir) => Ok(dir.clone()),
            None => Ok(paths::global_worktrees_dir()?.join(project_name)),
        }
    }
}

/// Save settings to file
pub fn save_settings(settings: &Settings, root: &Path) -> Result<()> {
    let settings_path = paths::settings_file_in(root);
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&settings_path, content)
        .with_context(|| format!("Failed to write {}", settings_path.display()))?;
    Ok(())
}

/// Save local settings to file
pub fn save_local_settings(settings: &LocalSettings, root: &Path) -> Result<()> {
    let settings_path = paths::local_settings_file_in(root);
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&settings_path, content)
        .with_context(|| format!("Failed to write {}", settings_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_values() {
        let settings = Settings::default();
        assert_eq!(settings.port_count, 10);
        assert_eq!(settings.port_range_start, 50000);
        assert_eq!(settings.port_range_end, 60000);
        assert_eq!(settings.branch_prefix, "worktree/");
        assert!(settings.auto_launch_terminal);
    }

    #[test]
    fn test_settings_json_parsing() {
        let json = r#"{
            "portCount": 5,
            "portRangeStart": 40000,
            "portRangeEnd": 45000,
            "branchPrefix": "feature/",
            "autoLaunchTerminal": false
        }"#;

        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.port_count, 5);
        assert_eq!(settings.port_range_start, 40000);
        assert_eq!(settings.port_range_end, 45000);
        assert_eq!(settings.branch_prefix, "feature/");
        assert!(!settings.auto_launch_terminal);
    }

    #[test]
    fn test_settings_partial_json_uses_defaults() {
        let json = r#"{"portCount": 20}"#;

        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.port_count, 20);
        // Other fields should use defaults
        assert_eq!(settings.port_range_start, 50000);
        assert_eq!(settings.port_range_end, 60000);
        assert_eq!(settings.branch_prefix, "worktree/");
        assert!(settings.auto_launch_terminal);
    }

    #[test]
    fn test_local_settings_json_parsing() {
        let json = r#"{"worktreeDir": "/custom/path"}"#;

        let settings: LocalSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.worktree_dir, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_local_settings_empty_json() {
        let json = r#"{}"#;

        let settings: LocalSettings = serde_json::from_str(json).unwrap();
        assert!(settings.worktree_dir.is_none());
    }
}
