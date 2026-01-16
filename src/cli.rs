use clap::{Parser, Subcommand};
use clap_complete::engine::ArgValueCandidates;
use clap_complete::Shell;

use crate::completions::worktree_names;

#[derive(Parser)]
#[command(name = "worktree")]
#[command(about = "Manage git worktrees with port allocation", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new worktree
    New {
        /// Parameter for the new worktree (e.g., branch name, issue ID)
        param: Option<String>,
    },

    /// Initialize worktree configuration for this project
    Init {
        /// Skip interactive prompts, use defaults
        #[arg(long)]
        defaults: bool,

        /// Skip script generation
        #[arg(long)]
        no_scripts: bool,
    },

    /// Execute the project's run script
    Run,

    /// Execute the project's stop script
    Stop,

    /// Clean up and delete a worktree
    Close {
        /// Worktree name to close (optional if inside a worktree)
        #[arg(add = ArgValueCandidates::new(worktree_names))]
        name: Option<String>,

        /// Force close without confirmation
        #[arg(short, long)]
        force: bool,

        /// Interactively select worktree to close
        #[arg(short, long)]
        interactive: bool,
    },

    /// Open an existing worktree in the configured terminal
    Open {
        /// Worktree name to open (optional if inside a worktree)
        #[arg(add = ArgValueCandidates::new(worktree_names))]
        name: Option<String>,

        /// Interactively select worktree to open
        #[arg(short, long)]
        interactive: bool,
    },

    /// Rename a worktree's display name
    Rename {
        /// New display name for the worktree. If omitted, prompts for input.
        new_name: Option<String>,

        /// Worktree to rename (name or directory). If omitted, uses current worktree or prompts.
        #[arg(add = ArgValueCandidates::new(worktree_names))]
        worktree: Option<String>,

        /// Clear custom name and revert to directory name
        #[arg(long)]
        clear: bool,
    },

    /// List active worktrees (current project by default)
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show worktrees from all projects
        #[arg(short, long)]
        all: bool,
    },

    /// Clean up inactive worktrees
    Cleanup {
        /// Only show worktrees older than N days
        #[arg(long)]
        older_than: Option<u32>,

        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,

        /// Show worktrees from all projects
        #[arg(short, long)]
        all: bool,
    },

    /// Show information about a worktree
    Status {
        /// Worktree name to show status for (optional, defaults to current worktree)
        #[arg(add = ArgValueCandidates::new(worktree_names))]
        name: Option<String>,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
