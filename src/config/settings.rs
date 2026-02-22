use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::paths;

/// User-scoped settings (~/.config/worktree/config.json)
/// These are personal preferences that apply across all projects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    /// Whether to automatically launch terminal on worktree creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_launch_terminal: Option<bool>,

    /// Preferred terminal emulator (e.g., "tmux", "iterm2", "ghostty")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<String>,
}

impl UserSettings {
    /// Load user settings from ~/.config/worktree/config.json
    pub fn load() -> Result<Option<Self>> {
        let config_path = paths::user_config_file()?;
        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        let settings: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", config_path.display()))?;
        Ok(Some(settings))
    }

    /// Save user settings to ~/.config/worktree/config.json
    pub fn save(&self) -> Result<()> {
        paths::ensure_user_config_dir()?;
        let config_path = paths::user_config_file()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
        Ok(())
    }

    /// Interactively prompt user for their preferences and save them
    pub fn setup_interactive() -> Result<Self> {
        use crate::terminal::{detect_terminal, Terminal};

        println!();
        println!(
            "{} {}",
            "First-time setup:".bold(),
            "Let's configure your preferences.".dimmed()
        );
        println!(
            "{}",
            "These settings will be saved to ~/.config/worktree/config.json".dimmed()
        );
        println!();

        // Ask about auto-launch terminal
        print!(
            "{}",
            "Automatically launch terminal when creating a worktree? (Y/n): ".cyan()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let auto_launch = input.trim().to_lowercase();
        let auto_launch_terminal =
            auto_launch.is_empty() || auto_launch == "y" || auto_launch == "yes";

        // Ask about terminal preference
        println!();
        println!("{}", "Select your preferred terminal:".cyan());

        // Build terminal options based on platform
        let mut options: Vec<(&str, &str)> = vec![("auto", "Auto-detect")];

        #[cfg(target_os = "macos")]
        {
            options.extend([
                ("tmux", "tmux (creates named sessions)"),
                ("iterm2", "iTerm2"),
                ("warp", "Warp"),
                ("ghostty", "Ghostty"),
                ("terminal", "Terminal.app"),
                ("vscode", "VS Code"),
            ]);
        }

        #[cfg(target_os = "linux")]
        {
            options.extend([
                ("tmux", "tmux (creates named sessions)"),
                ("ghostty", "Ghostty"),
                ("gnome-terminal", "GNOME Terminal"),
                ("konsole", "Konsole"),
                ("kitty", "Kitty"),
                ("alacritty", "Alacritty"),
                ("xfce4-terminal", "Xfce Terminal"),
                ("vscode", "VS Code"),
            ]);
        }

        // Show current detected terminal as hint
        if let Some(detected) = detect_terminal() {
            println!(
                "  {} {}",
                "Currently detected:".dimmed(),
                detected.name().green()
            );
        }
        println!();

        for (i, (value, label)) in options.iter().enumerate() {
            println!(
                "  {} {} ({})",
                format!("[{}]", i + 1).dimmed(),
                label,
                value
            );
        }

        println!();
        print!("{}", "Enter choice [1]: ".cyan());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        let terminal = if choice.is_empty() || choice == "1" {
            None // Auto-detect
        } else if let Ok(num) = choice.parse::<usize>() {
            if num > 0 && num <= options.len() {
                let value = options[num - 1].0;
                if value == "auto" {
                    None
                } else {
                    Some(value.to_string())
                }
            } else {
                None
            }
        } else {
            // User typed a terminal name directly
            if Terminal::from_str(choice).is_some() {
                Some(choice.to_string())
            } else {
                None
            }
        };

        let settings = Self {
            auto_launch_terminal: Some(auto_launch_terminal),
            terminal,
        };

        // Save the settings
        settings.save()?;

        println!();
        println!("{}", "User preferences saved!".green().bold());
        println!(
            "  {}",
            format!("Config file: {}", paths::user_config_file()?.display()).dimmed()
        );
        println!();

        Ok(settings)
    }

    /// Load existing settings or run interactive setup if none exist
    pub fn load_or_setup() -> Result<Self> {
        match Self::load()? {
            Some(settings) => Ok(settings),
            None => Self::setup_interactive(),
        }
    }

    /// Check if user configuration exists
    pub fn exists() -> Result<bool> {
        let config_path = paths::user_config_file()?;
        Ok(config_path.exists())
    }

    /// Ensure user configuration exists, prompting for setup if not
    /// This is called early in command execution to ensure first-time setup happens
    pub fn ensure_configured() -> Result<()> {
        if !Self::exists()? {
            Self::setup_interactive()?;
        }
        Ok(())
    }
}

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

    /// Terminal settings can be overridden at project level (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_launch_terminal: Option<bool>,

    /// Terminal to use at project level (optional, overrides user setting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port_count: default_port_count(),
            port_range_start: default_port_range_start(),
            port_range_end: default_port_range_end(),
            branch_prefix: default_branch_prefix(),
            auto_launch_terminal: None,
            terminal: None,
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
    /// Terminal to use (e.g., "tmux", "iterm2"). If None, auto-detects.
    pub terminal: Option<String>,
}

impl MergedSettings {
    /// Load and merge settings from a specific root directory
    /// Priority: project settings > user settings > defaults
    pub fn load_from(root: &Path) -> Result<Self> {
        let settings_path = paths::settings_file_in(root);
        let local_settings_path = paths::local_settings_file_in(root);

        // Load project settings
        let settings: Settings = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)
                .with_context(|| format!("Failed to read {}", settings_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", settings_path.display()))?
        } else {
            Settings::default()
        };

        // Load project-local settings
        let local_settings: LocalSettings = if local_settings_path.exists() {
            let content = std::fs::read_to_string(&local_settings_path)
                .with_context(|| format!("Failed to read {}", local_settings_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", local_settings_path.display()))?
        } else {
            LocalSettings::default()
        };

        // Load user settings (or prompt for setup if not exists)
        let user_settings = UserSettings::load_or_setup()?;

        // Merge with priority: project > user > default
        let auto_launch_terminal = settings
            .auto_launch_terminal
            .or(user_settings.auto_launch_terminal)
            .unwrap_or(true);

        let terminal = settings.terminal.or(user_settings.terminal);

        Ok(Self {
            port_count: settings.port_count,
            port_range_start: settings.port_range_start,
            port_range_end: settings.port_range_end,
            branch_prefix: settings.branch_prefix,
            auto_launch_terminal,
            worktree_dir: local_settings.worktree_dir,
            terminal,
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
        // auto_launch_terminal is None by default (uses user settings or defaults to true)
        assert!(settings.auto_launch_terminal.is_none());
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
        assert_eq!(settings.auto_launch_terminal, Some(false));
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
        // auto_launch_terminal defaults to None when not specified
        assert!(settings.auto_launch_terminal.is_none());
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

    #[test]
    fn test_user_settings_json_parsing() {
        let json = r#"{"autoLaunchTerminal": true, "terminal": "iterm2"}"#;

        let settings: UserSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.auto_launch_terminal, Some(true));
        assert_eq!(settings.terminal, Some("iterm2".to_string()));
    }

    #[test]
    fn test_user_settings_empty_json() {
        let json = r#"{}"#;

        let settings: UserSettings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_launch_terminal.is_none());
        assert!(settings.terminal.is_none());
    }
}
