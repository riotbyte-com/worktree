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

    /// Custom display name (optional, defaults to directory name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Builder for creating WorktreeState
pub struct WorktreeStateBuilder {
    name: String,
    project_name: String,
    original_dir: PathBuf,
    worktree_dir: PathBuf,
    branch: String,
    ports: Vec<u16>,
    param: Option<String>,
    display_name: Option<String>,
}

impl WorktreeStateBuilder {
    /// Create a new builder with required fields
    pub fn new(name: String, project_name: String, worktree_dir: PathBuf) -> Self {
        Self {
            name,
            project_name,
            original_dir: PathBuf::new(),
            worktree_dir,
            branch: String::new(),
            ports: Vec::new(),
            param: None,
            display_name: None,
        }
    }

    pub fn original_dir(mut self, dir: PathBuf) -> Self {
        self.original_dir = dir;
        self
    }

    pub fn branch(mut self, branch: String) -> Self {
        self.branch = branch;
        self
    }

    pub fn ports(mut self, ports: Vec<u16>) -> Self {
        self.ports = ports;
        self
    }

    pub fn param(mut self, param: Option<String>) -> Self {
        self.param = param;
        self
    }

    pub fn display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn build(self) -> WorktreeState {
        let allocation_key = format!("{}/{}", self.project_name, self.name);
        WorktreeState {
            name: self.name,
            project_name: self.project_name,
            original_dir: self.original_dir,
            worktree_dir: self.worktree_dir,
            branch: self.branch,
            ports: self.ports,
            allocation_key,
            created_at: Utc::now(),
            param: self.param,
            display_name: self.display_name,
        }
    }
}

impl WorktreeState {
    /// Create a builder for WorktreeState
    pub fn builder(
        name: String,
        project_name: String,
        worktree_dir: PathBuf,
    ) -> WorktreeStateBuilder {
        WorktreeStateBuilder::new(name, project_name, worktree_dir)
    }

    /// Get the effective display name (custom name or directory name)
    pub fn effective_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    /// Check if the worktree has a custom display name
    pub fn has_custom_name(&self) -> bool {
        self.display_name.is_some()
    }

    /// Check if a given identifier matches this worktree
    /// Matches against: directory name, display name, or allocation_key suffix
    pub fn matches_identifier(&self, identifier: &str) -> bool {
        self.name == identifier
            || self.display_name.as_deref() == Some(identifier)
            || self.allocation_key.ends_with(&format!("/{}", identifier))
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
