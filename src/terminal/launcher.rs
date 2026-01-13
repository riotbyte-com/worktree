use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Escape a string for safe use in shell commands
/// This handles single quotes by ending the quoted string, adding an escaped quote, and resuming
fn shell_escape(s: &str) -> String {
    // Replace single quotes with '\'' (end quote, escaped quote, start quote)
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Supported terminal emulators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Terminal {
    // Cross-platform
    Tmux,
    // macOS
    AppleTerminal,
    ITerm2,
    Warp,
    Ghostty,
    VSCode,
    // Linux
    #[cfg(target_os = "linux")]
    GnomeTerminal,
    #[cfg(target_os = "linux")]
    Konsole,
    #[cfg(target_os = "linux")]
    Xfce4Terminal,
    #[cfg(target_os = "linux")]
    Kitty,
    #[cfg(target_os = "linux")]
    Alacritty,
}

impl Terminal {
    pub fn name(&self) -> &'static str {
        match self {
            Terminal::Tmux => "tmux",
            Terminal::AppleTerminal => "Terminal.app",
            Terminal::ITerm2 => "iTerm2",
            Terminal::Warp => "Warp",
            Terminal::Ghostty => "Ghostty",
            Terminal::VSCode => "VS Code",
            #[cfg(target_os = "linux")]
            Terminal::GnomeTerminal => "GNOME Terminal",
            #[cfg(target_os = "linux")]
            Terminal::Konsole => "Konsole",
            #[cfg(target_os = "linux")]
            Terminal::Xfce4Terminal => "Xfce Terminal",
            #[cfg(target_os = "linux")]
            Terminal::Kitty => "Kitty",
            #[cfg(target_os = "linux")]
            Terminal::Alacritty => "Alacritty",
        }
    }

    /// Parse a terminal from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tmux" => Some(Terminal::Tmux),
            "terminal" | "terminal.app" | "apple_terminal" => Some(Terminal::AppleTerminal),
            "iterm" | "iterm2" => Some(Terminal::ITerm2),
            "warp" => Some(Terminal::Warp),
            "ghostty" => Some(Terminal::Ghostty),
            "vscode" | "code" => Some(Terminal::VSCode),
            #[cfg(target_os = "linux")]
            "gnome-terminal" | "gnome" => Some(Terminal::GnomeTerminal),
            #[cfg(target_os = "linux")]
            "konsole" => Some(Terminal::Konsole),
            #[cfg(target_os = "linux")]
            "xfce4-terminal" | "xfce" => Some(Terminal::Xfce4Terminal),
            #[cfg(target_os = "linux")]
            "kitty" => Some(Terminal::Kitty),
            #[cfg(target_os = "linux")]
            "alacritty" => Some(Terminal::Alacritty),
            _ => None,
        }
    }
}

/// Detect the current terminal emulator
pub fn detect_terminal() -> Option<Terminal> {
    // Check TERM_PROGRAM environment variable (macOS)
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "Apple_Terminal" => return Some(Terminal::AppleTerminal),
            "iTerm.app" => return Some(Terminal::ITerm2),
            "WarpTerminal" => return Some(Terminal::Warp),
            "ghostty" => return Some(Terminal::Ghostty),
            "vscode" => return Some(Terminal::VSCode),
            _ => {}
        }
    }

    // Check for Linux terminals by availability
    #[cfg(target_os = "linux")]
    {
        if which::which("gnome-terminal").is_ok() {
            return Some(Terminal::GnomeTerminal);
        }
        if which::which("konsole").is_ok() {
            return Some(Terminal::Konsole);
        }
        if which::which("xfce4-terminal").is_ok() {
            return Some(Terminal::Xfce4Terminal);
        }
        if which::which("kitty").is_ok() {
            return Some(Terminal::Kitty);
        }
        if which::which("alacritty").is_ok() {
            return Some(Terminal::Alacritty);
        }
        None
    }

    // Fallback for macOS: check for installed applications
    #[cfg(target_os = "macos")]
    {
        if std::path::Path::new("/Applications/iTerm.app").exists() {
            return Some(Terminal::ITerm2);
        }
        if std::path::Path::new("/Applications/Warp.app").exists() {
            return Some(Terminal::Warp);
        }
        if std::path::Path::new("/Applications/Ghostty.app").exists() {
            return Some(Terminal::Ghostty);
        }
        // Terminal.app is always available on macOS
        Some(Terminal::AppleTerminal)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    None
}

/// Launch a new terminal window in the specified directory
/// Note: For Tmux, use `launch_tmux_session` instead as it requires additional context
pub fn launch(terminal: &Terminal, dir: &Path) -> Result<()> {
    let dir_str = dir.to_str().context("Invalid directory path")?;

    match terminal {
        Terminal::Tmux => {
            anyhow::bail!(
                "Use launch_tmux_session for tmux, which requires project and worktree names"
            )
        }
        Terminal::AppleTerminal => launch_apple_terminal(dir_str),
        Terminal::ITerm2 => launch_iterm2(dir_str),
        Terminal::Warp => launch_warp(dir_str),
        Terminal::Ghostty => launch_ghostty(dir_str),
        Terminal::VSCode => launch_vscode(dir_str),
        #[cfg(target_os = "linux")]
        Terminal::GnomeTerminal => launch_gnome_terminal(dir_str),
        #[cfg(target_os = "linux")]
        Terminal::Konsole => launch_konsole(dir_str),
        #[cfg(target_os = "linux")]
        Terminal::Xfce4Terminal => launch_xfce4_terminal(dir_str),
        #[cfg(target_os = "linux")]
        Terminal::Kitty => launch_kitty(dir_str),
        #[cfg(target_os = "linux")]
        Terminal::Alacritty => launch_alacritty(dir_str),
    }
}

/// Generate the tmux session name for a worktree
pub fn tmux_session_name(project_name: &str, worktree_name: &str) -> String {
    format!("{}-{}", project_name, worktree_name)
}

/// Check if we're currently inside a tmux session
fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Launch a new tmux session for a worktree
pub fn launch_tmux_session(project_name: &str, worktree_name: &str, dir: &Path) -> Result<()> {
    let dir_str = dir.to_str().context("Invalid directory path")?;
    let session_name = tmux_session_name(project_name, worktree_name);
    let inside_tmux = is_inside_tmux();

    // Check if session already exists
    let session_exists = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !session_exists {
        // Create new session (always detached first)
        Command::new("tmux")
            .args(["new-session", "-d", "-s", &session_name, "-c", dir_str])
            .output()
            .context("Failed to create tmux session")?;
    }

    // Switch to or attach to the session
    if inside_tmux {
        // Already in tmux, switch to the session
        Command::new("tmux")
            .args(["switch-client", "-t", &session_name])
            .status()
            .context("Failed to switch to tmux session")?;
    } else {
        // Not in tmux, attach to the session
        Command::new("tmux")
            .args(["attach-session", "-t", &session_name])
            .status()
            .context("Failed to attach to tmux session")?;
    }

    Ok(())
}

/// Kill a tmux session for a worktree
pub fn kill_tmux_session(project_name: &str, worktree_name: &str) -> Result<bool> {
    let session_name = tmux_session_name(project_name, worktree_name);

    // Check if session exists
    let check = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output();

    if let Ok(output) = check {
        if output.status.success() {
            // Session exists, kill it
            let result = Command::new("tmux")
                .args(["kill-session", "-t", &session_name])
                .output()
                .context("Failed to kill tmux session")?;

            return Ok(result.status.success());
        }
    }

    // Session doesn't exist
    Ok(false)
}

fn launch_apple_terminal(dir: &str) -> Result<()> {
    let escaped_dir = shell_escape(dir);
    let script = format!(
        r#"tell application "Terminal"
            do script "cd {}"
            activate
        end tell"#,
        escaped_dir
    );

    Command::new("osascript")
        .args(["-e", &script])
        .output()
        .context("Failed to launch Terminal.app")?;

    Ok(())
}

fn launch_iterm2(dir: &str) -> Result<()> {
    let escaped_dir = shell_escape(dir);
    let script = format!(
        r#"tell application "iTerm"
            create window with default profile
            tell current session of current window
                write text "cd {}"
            end tell
            activate
        end tell"#,
        escaped_dir
    );

    Command::new("osascript")
        .args(["-e", &script])
        .output()
        .context("Failed to launch iTerm2")?;

    Ok(())
}

fn launch_warp(dir: &str) -> Result<()> {
    let escaped_dir = shell_escape(dir);
    let script = format!(
        r#"tell application "Warp"
            do script "cd {}"
            activate
        end tell"#,
        escaped_dir
    );

    Command::new("osascript")
        .args(["-e", &script])
        .output()
        .context("Failed to launch Warp")?;

    Ok(())
}

fn launch_ghostty(dir: &str) -> Result<()> {
    let escaped_dir = shell_escape(dir);
    Command::new("ghostty")
        .args(["-e", &format!("cd {} && $SHELL", escaped_dir)])
        .spawn()
        .context("Failed to launch Ghostty")?;

    Ok(())
}

fn launch_vscode(dir: &str) -> Result<()> {
    Command::new("code")
        .args([dir])
        .spawn()
        .context("Failed to launch VS Code")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_gnome_terminal(dir: &str) -> Result<()> {
    Command::new("gnome-terminal")
        .args(["--tab", "--working-directory", dir])
        .spawn()
        .context("Failed to launch GNOME Terminal")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_konsole(dir: &str) -> Result<()> {
    Command::new("konsole")
        .args(["--new-tab", "--workdir", dir])
        .spawn()
        .context("Failed to launch Konsole")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_xfce4_terminal(dir: &str) -> Result<()> {
    Command::new("xfce4-terminal")
        .args(["--tab", "--working-directory", dir])
        .spawn()
        .context("Failed to launch Xfce Terminal")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_kitty(dir: &str) -> Result<()> {
    Command::new("kitty")
        .args(["--directory", dir])
        .spawn()
        .context("Failed to launch Kitty")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_alacritty(dir: &str) -> Result<()> {
    Command::new("alacritty")
        .args(["--working-directory", dir])
        .spawn()
        .context("Failed to launch Alacritty")?;

    Ok(())
}

/// Get the command to manually open a terminal in a directory
pub fn get_manual_command(dir: &Path) -> String {
    let escaped_dir = shell_escape(&dir.display().to_string());
    format!("cd {} && $SHELL", escaped_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple_path() {
        let result = shell_escape("/home/user/projects");
        assert_eq!(result, "'/home/user/projects'");
    }

    #[test]
    fn test_shell_escape_path_with_spaces() {
        let result = shell_escape("/home/user/my project");
        assert_eq!(result, "'/home/user/my project'");
    }

    #[test]
    fn test_shell_escape_path_with_single_quote() {
        let result = shell_escape("/home/user/it's mine");
        assert_eq!(result, "'/home/user/it'\\''s mine'");
    }

    #[test]
    fn test_shell_escape_path_with_special_chars() {
        let result = shell_escape("/home/user/$HOME");
        // Inside single quotes, $ is literal
        assert_eq!(result, "'/home/user/$HOME'");
    }

    #[test]
    fn test_shell_escape_path_with_semicolon() {
        let result = shell_escape("/path/with;semicolon");
        // Inside single quotes, semicolon is literal
        assert_eq!(result, "'/path/with;semicolon'");
    }

    #[test]
    fn test_get_manual_command_escapes_path() {
        let path = Path::new("/home/user/project's dir");
        let cmd = get_manual_command(path);
        assert!(cmd.contains("'\\''"));
        assert!(cmd.starts_with("cd "));
        assert!(cmd.ends_with(" && $SHELL"));
    }
}
