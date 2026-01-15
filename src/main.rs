mod cli;
mod commands;
mod config;
mod git;
mod names;
mod ports;
mod scripts;
mod terminal;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use config::settings::UserSettings;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure user configuration exists for all commands except completions and help
    // This prompts for first-time setup if ~/.config/worktree/config.json doesn't exist
    if let Some(ref cmd) = cli.command {
        if !matches!(cmd, Commands::Completions { .. }) {
            UserSettings::ensure_configured()?;
        }
    }

    match cli.command {
        Some(Commands::Init {
            defaults,
            no_scripts,
        }) => commands::init::execute(defaults, no_scripts),
        Some(Commands::New { param }) => commands::new::execute(param),
        Some(Commands::Run) => commands::run::execute(),
        Some(Commands::Stop) => commands::stop::execute(),
        Some(Commands::Close {
            name,
            force,
            interactive,
        }) => commands::close::execute(name, force, interactive),
        Some(Commands::Open { name, interactive }) => commands::open::execute(name, interactive),
        Some(Commands::Rename {
            worktree,
            new_name,
            clear,
        }) => commands::rename::execute(worktree, new_name, clear),
        Some(Commands::List { json }) => commands::list::execute(json),
        Some(Commands::Cleanup { older_than, force }) => {
            commands::cleanup::execute(older_than, force)
        }
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "worktree", &mut std::io::stdout());
            Ok(())
        }
        None => {
            // Show help when no command is provided
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
