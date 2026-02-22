mod cli;
mod commands;
mod completions;
mod config;
mod git;
mod names;
mod ports;
mod scripts;
mod terminal;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use cli::{Cli, Commands};
use config::settings::UserSettings;

fn main() -> Result<()> {
    // Handle dynamic shell completions when COMPLETE env var is set
    CompleteEnv::with_factory(Cli::command).complete();

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
            new_name,
            worktree,
            clear,
        }) => commands::rename::execute(new_name, worktree, clear),
        Some(Commands::List { json, all }) => commands::list::execute(json, all),
        Some(Commands::Cleanup {
            older_than,
            force,
            all,
        }) => commands::cleanup::execute(older_than, force, all),
        Some(Commands::Status { name }) => commands::status::execute(name),
        Some(Commands::Path { name }) => commands::path::execute(name),
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
