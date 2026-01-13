use anyhow::Result;

use crate::config::state::WorktreeState;
use crate::git;
use crate::ports;
use crate::scripts;

/// Options for worktree removal
#[derive(Default)]
pub struct RemoveOptions {
    /// Whether to print verbose output
    pub verbose: bool,
}

/// Result of a worktree removal operation
pub struct RemoveResult {
    /// Whether the close script ran successfully
    pub close_script_success: Option<bool>,
    /// The ports that were deallocated, if any
    pub deallocated_ports: Option<Vec<u16>>,
    /// Whether the git worktree was removed successfully
    pub worktree_removed: bool,
}

/// Remove a worktree: run close script, deallocate ports, and remove the git worktree
/// This is the shared logic between the `close` and `cleanup` commands.
pub fn remove_worktree(state: &WorktreeState, options: &RemoveOptions) -> Result<RemoveResult> {
    let mut result = RemoveResult {
        close_script_success: None,
        deallocated_ports: None,
        worktree_removed: false,
    };

    // Run close script if it exists
    let close_script = state.worktree_dir.join(".worktree").join("close.sh");
    if close_script.exists() {
        if options.verbose {
            println!("  Running close script...");
        }
        let env = scripts::build_env_vars(state);
        let success = scripts::execute_script_ignore_errors(&close_script, &env);
        result.close_script_success = Some(success);
    }

    // Deallocate ports
    if options.verbose {
        println!("  Deallocating ports...");
    }
    match ports::deallocate(&state.allocation_key) {
        Ok(ports) => {
            result.deallocated_ports = ports;
        }
        Err(_) => {
            // Ignore errors during port deallocation
        }
    }

    // Remove git worktree
    if options.verbose {
        println!("  Removing git worktree...");
    }
    match git::remove_worktree(&state.original_dir, &state.worktree_dir, true) {
        Ok(_) => {
            result.worktree_removed = true;
        }
        Err(_) => {
            // Try manual removal as fallback
            if std::fs::remove_dir_all(&state.worktree_dir).is_ok() {
                result.worktree_removed = true;
            }
        }
    }

    Ok(result)
}
