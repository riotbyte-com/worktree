use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Worktree state stored in state.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeState {
    pub name: String,
    pub project_name: String,
    pub original_dir: PathBuf,
    pub worktree_dir: PathBuf,
    pub branch: String,
    pub ports: Vec<u16>,
    pub allocation_key: String,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

impl WorktreeState {
    /// Create a new worktree state
    pub fn new(
        name: String,
        project_name: String,
        original_dir: PathBuf,
        worktree_dir: PathBuf,
        branch: String,
        ports: Vec<u16>,
        param: Option<String>,
    ) -> Self {
        let allocation_key = format!("{}/{}", project_name, name);
        Self {
            name,
            project_name,
            original_dir,
            worktree_dir,
            branch,
            ports,
            allocation_key,
            created_at: Utc::now(),
            param,
        }
    }

    /// Save state to the worktree directory
    pub fn save(&self) -> Result<()> {
        let state_path = self.worktree_dir.join("state.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&state_path, content)
            .with_context(|| format!("Failed to write {}", state_path.display()))?;
        Ok(())
    }

    /// Load state from a state.json file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let state: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(state)
    }
}

/// Detect if the current directory is within a worktree by traversing up
pub fn detect_worktree() -> Result<Option<WorktreeState>> {
    detect_worktree_from(&std::env::current_dir()?)
}

/// Detect if a directory is within a worktree by traversing up
pub fn detect_worktree_from(start: &Path) -> Result<Option<WorktreeState>> {
    let mut current = start.to_path_buf();

    loop {
        let state_path = current.join("state.json");
        if state_path.exists() {
            let state = WorktreeState::load(&state_path)?;
            return Ok(Some(state));
        }

        match current.parent() {
            Some(parent) => {
                if parent == current {
                    break;
                }
                current = parent.to_path_buf();
            }
            None => break,
        }
    }

    Ok(None)
}
