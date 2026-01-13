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

    let repo_root = git::get_repo_root()?;
    let project_name = git::get_project_name()?;

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

    println!("{} {}", "Creating worktree:".bold(), worktree_name.green());
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
    let state = WorktreeState::new(
        worktree_name.clone(),
        project_name.clone(),
        repo_root.clone(),
        worktree_dir.clone(),
        branch.clone(),
        allocation.ports.clone(),
        param.clone(),
    );

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
        if let Some(term) = terminal::detect_terminal() {
            println!("  Launching {}...", term.name());
            if let Err(e) = terminal::launch(&term, &worktree_dir) {
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
    println!("  {} {}", "Name:".dimmed(), worktree_name.green());
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
