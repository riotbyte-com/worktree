use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::{
    paths,
    settings::{LocalSettings, Settings},
};
use crate::git;
use crate::scripts;

pub fn execute(defaults: bool, no_scripts: bool) -> Result<()> {
    // Check if we're in a git repository
    if !git::is_git_repo() {
        bail!("Not in a git repository. Please run this command from within a git repository.");
    }

    let repo_root = git::get_repo_root()?;
    let config_dir = paths::project_config_dir_in(&repo_root);

    // Check if already initialized
    if config_dir.exists() && paths::settings_file_in(&repo_root).exists() {
        bail!(
            "Worktree configuration already exists at {}\nTo reinitialize, remove the .worktree directory first.",
            config_dir.display()
        );
    }

    println!("{}", "Initializing worktree configuration...".bold());
    println!();

    // Get settings
    let settings = if defaults {
        Settings::default()
    } else {
        prompt_settings()?
    };

    // Get local settings
    let local_settings = if defaults {
        LocalSettings::default()
    } else {
        prompt_local_settings()?
    };

    // Create config directory (if not exists)
    std::fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create {}", config_dir.display()))?;

    // Save settings
    crate::config::settings::save_settings(&settings, &repo_root)?;
    println!("  {} .worktree/settings.json", "Created".green());

    // Save local settings if custom directory was set
    if local_settings.worktree_dir.is_some() {
        crate::config::settings::save_local_settings(&local_settings, &repo_root)?;
        println!("  {} .worktree/settings.local.json", "Created".green());
    }

    // Create .gitignore
    let gitignore_content = "# Local settings (user-specific paths)\nsettings.local.json\n";
    std::fs::write(config_dir.join(".gitignore"), gitignore_content)?;
    println!("  {} .worktree/.gitignore", "Created".green());

    // Create README.md
    let readme_content = include_str!("../../templates/README.md");
    std::fs::write(config_dir.join("README.md"), readme_content)?;
    println!("  {} .worktree/README.md", "Created".green());

    // Create SETUP.md
    let setup_md_content = include_str!("../../templates/SETUP.md");
    std::fs::write(config_dir.join("SETUP.md"), setup_md_content)?;
    println!("  {} .worktree/SETUP.md", "Created".green());

    // Generate scripts
    if !no_scripts {
        println!();
        generate_scripts(&config_dir, &repo_root)?;
    }

    println!();
    println!("{}", "Initialization complete!".green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Review and customize the scripts in {}",
        ".worktree/".cyan()
    );
    println!("  2. Commit the .worktree directory to your repository");
    println!("  3. Run {} to create a new worktree", "worktree".cyan());

    Ok(())
}

fn prompt_settings() -> Result<Settings> {
    let mut settings = Settings::default();

    println!("Configure project worktree settings (press Enter for defaults):");
    println!();

    // Port count
    print!(
        "  Number of ports to allocate [{}]: ",
        settings.port_count.to_string().dimmed()
    );
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        settings.port_count = input.parse().context("Invalid port count")?;
    }

    // Port range
    print!(
        "  Port range start [{}]: ",
        settings.port_range_start.to_string().dimmed()
    );
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        settings.port_range_start = input.parse().context("Invalid port range start")?;
    }

    print!(
        "  Port range end [{}]: ",
        settings.port_range_end.to_string().dimmed()
    );
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        settings.port_range_end = input.parse().context("Invalid port range end")?;
    }

    // Branch prefix
    print!("  Branch prefix [{}]: ", settings.branch_prefix.dimmed());
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        settings.branch_prefix = input;
    }

    println!();

    Ok(settings)
}

fn prompt_local_settings() -> Result<LocalSettings> {
    let mut local_settings = LocalSettings::default();

    let default_dir = paths::global_worktrees_dir()?;
    println!(
        "  Default worktree directory: {}",
        default_dir.display().to_string().dimmed()
    );
    print!("  Custom worktree directory (or Enter for default): ");
    io::stdout().flush()?;

    let input = read_line()?;
    if !input.is_empty() {
        let path = PathBuf::from(shellexpand::tilde(&input).to_string());
        local_settings.worktree_dir = Some(path);
    }

    println!();

    Ok(local_settings)
}

fn generate_scripts(config_dir: &std::path::Path, project_dir: &std::path::Path) -> Result<()> {
    let scripts = if scripts::is_claude_available() {
        print!("Generate scripts with Claude CLI? (y/n) [y]: ");
        io::stdout().flush()?;
        let input = read_line()?.to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            println!("  Generating scripts with Claude CLI...");
            match scripts::generate_with_claude(project_dir) {
                Ok(scripts) => {
                    println!("  {} Generated scripts with Claude", "✓".green());
                    scripts
                }
                Err(e) => {
                    println!("  {} Failed to generate with Claude: {}", "⚠".yellow(), e);
                    println!("  Using template scripts instead...");
                    scripts::generate_templates()
                }
            }
        } else {
            scripts::generate_templates()
        }
    } else {
        println!("  Claude CLI not found, using template scripts...");
        scripts::generate_templates()
    };

    scripts.write_to(config_dir)?;
    println!("  {} .worktree/setup.sh", "Created".green());
    println!("  {} .worktree/run.sh", "Created".green());
    println!("  {} .worktree/stop.sh", "Created".green());
    println!("  {} .worktree/close.sh", "Created".green());

    Ok(())
}

fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Add shellexpand for tilde expansion
mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<'_, str> {
        if path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                return std::borrow::Cow::Owned(path.replacen('~', &home.to_string_lossy(), 1));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}
