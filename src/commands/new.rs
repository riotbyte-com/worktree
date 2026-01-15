use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::config::{paths, settings::MergedSettings, state::WorktreeState};
use crate::git;
use crate::names;
use crate::ports;
use crate::scripts;
use crate::terminal;

pub fn execute(param: Option<String>) -> Result<()> {
    // Check if we're in a git repository
    if !git::is_git_repo() {
        bail!("Not in a git repository. Please run this command from within a git repository.");
    }

    // Use main repo root to ensure worktrees are created from the main project,
    // even when running from within an existing worktree
    let repo_root = git::get_main_repo_root()?;
    let project_name = git::get_main_project_name()?;

    // Check if project is initialized
    let config_dir = paths::project_config_dir_in(&repo_root);
    if !config_dir.exists() || !paths::settings_file_in(&repo_root).exists() {
        println!("{} Project not initialized.", "⚠".yellow());
        print!("Would you like to initialize it now? (y/n) [y]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            // Run init with defaults=false, no_scripts=false
            super::init::execute(false, false)?;
            println!();
        } else {
            bail!(
                "Project not initialized. Run {} first.",
                "worktree init".cyan()
            );
        }
    }

    // Load settings
    let settings = MergedSettings::load_from(&repo_root)?;

    // Generate worktree name
    let worktree_name = names::generate();

    // Determine branch name
    let branch = format!("{}{}", settings.branch_prefix, worktree_name);

    // Check if branch already exists
    if git::branch_exists(&branch) {
        bail!("Branch {} already exists", branch);
    }

    // Calculate worktree path
    let worktree_base = settings.get_worktree_base_dir(&project_name)?;
    let worktree_dir = worktree_base.join(&worktree_name);

    // Ensure parent directory exists
    if let Some(parent) = worktree_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Show display name if provided, with directory name
    let name_display = if let Some(ref p) = param {
        format!("{} - {}", p.green(), worktree_name.dimmed())
    } else {
        worktree_name.green().to_string()
    };
    println!("{} {}", "Creating worktree:".bold(), name_display);
    println!("  {} {}", "Branch:".dimmed(), branch.cyan());
    println!("  {} {}", "Path:".dimmed(), worktree_dir.display());

    // Create git worktree
    println!();
    println!("  Creating git worktree...");
    git::create_worktree(&worktree_dir, &branch)?;
    println!("  {} Git worktree created", "✓".green());

    // Allocate ports
    let allocation_key = format!("{}/{}", project_name, worktree_name);
    let allocation = ports::allocate(
        settings.port_count,
        &allocation_key,
        settings.port_range_start,
        settings.port_range_end,
    )?;

    if allocation.existing {
        println!("  {} Using existing port allocation", "✓".green());
    } else {
        println!(
            "  {} Allocated ports {}-{}",
            "✓".green(),
            allocation.ports.first().unwrap_or(&0),
            allocation.ports.last().unwrap_or(&0)
        );
    }

    // Create state
    // When param is provided, use it as the display name
    let state = WorktreeState::builder(
        worktree_name.clone(),
        project_name.clone(),
        worktree_dir.clone(),
    )
    .original_dir(repo_root.clone())
    .branch(branch.clone())
    .ports(allocation.ports.clone())
    .param(param.clone())
    .display_name(param.clone())
    .build();

    // Save state to worktree
    state.save()?;
    println!("  {} State saved", "✓".green());

    // Run setup script if it exists
    let setup_script = worktree_dir.join(".worktree").join("setup.sh");
    if setup_script.exists() {
        println!();
        println!("  Running setup script...");
        let env = scripts::build_env_vars(&state);
        match scripts::execute_script(&setup_script, &env) {
            Ok(_) => println!("  {} Setup complete", "✓".green()),
            Err(e) => println!("  {} Setup failed: {}", "⚠".yellow(), e),
        }
    }

    // Launch terminal if configured
    if settings.auto_launch_terminal {
        println!();
        // Get terminal from settings or auto-detect
        let term = settings
            .terminal
            .as_ref()
            .and_then(|t| terminal::Terminal::from_str(t))
            .or_else(terminal::detect_terminal);

        if let Some(term) = term {
            println!("  Launching {}...", term.name());
            // Use effective name (display name if set, otherwise directory name) for tmux session
            let effective_name = state.effective_name();
            let launch_result = if term == terminal::Terminal::Tmux {
                terminal::launch_tmux_session(&project_name, effective_name, &worktree_dir)
            } else {
                terminal::launch(&term, &worktree_dir)
            };

            if let Err(e) = launch_result {
                println!("  {} Failed to launch terminal: {}", "⚠".yellow(), e);
                println!(
                    "  Run manually: {}",
                    terminal::get_manual_command(&worktree_dir).dimmed()
                );
            }
        } else {
            println!(
                "  No terminal detected. Run manually:\n  {}",
                terminal::get_manual_command(&worktree_dir).dimmed()
            );
        }
    }

    // Summary
    println!();
    println!("{}", "Worktree created successfully!".green().bold());
    println!();
    // Show display name if set, with directory name
    let summary_name = if state.has_custom_name() {
        format!(
            "{} - {}",
            state.effective_name().green(),
            state.name.dimmed()
        )
    } else {
        state.name.green().to_string()
    };
    println!("  {} {}", "Name:".dimmed(), summary_name);
    println!("  {} {}", "Path:".dimmed(), worktree_dir.display());
    println!("  {} {}", "Branch:".dimmed(), branch.cyan());
    println!(
        "  {} {}-{}",
        "Ports:".dimmed(),
        allocation.ports.first().unwrap_or(&0),
        allocation.ports.last().unwrap_or(&0)
    );

    if !settings.auto_launch_terminal {
        println!();
        println!("To start working:");
        println!("  cd {}", worktree_dir.display());
    }

    Ok(())
}
