use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::checker::find_consecutive_free;
use crate::config::paths;

/// Port allocations stored in ~/.worktree/port-allocations.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortAllocations {
    #[serde(flatten)]
    pub allocations: HashMap<String, Vec<u16>>,
}

impl PortAllocations {
    /// Load allocations from file
    pub fn load() -> Result<Self> {
        let path = paths::allocations_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let allocations: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(allocations)
    }

    /// Save allocations to file
    pub fn save(&self) -> Result<()> {
        paths::ensure_global_dir()?;
        let path = paths::allocations_file()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Get all currently allocated ports as a set
    pub fn all_allocated_ports(&self) -> HashSet<u16> {
        self.allocations.values().flatten().copied().collect()
    }

    /// Clean up stale allocations (worktrees that no longer exist)
    pub fn cleanup_stale(&mut self) -> Vec<String> {
        let mut stale_keys = Vec::new();

        for key in self.allocations.keys() {
            // Check if the worktree still exists by looking for its state.json
            let worktree_exists = self.worktree_exists(key);
            if !worktree_exists {
                stale_keys.push(key.clone());
            }
        }

        for key in &stale_keys {
            self.allocations.remove(key);
        }

        stale_keys
    }

    /// Check if a worktree exists for the given allocation key
    fn worktree_exists(&self, key: &str) -> bool {
        // Key format is "project/worktree" or just "worktree"
        let parts: Vec<&str> = key.split('/').collect();

        let global_dir = match paths::global_worktrees_dir() {
            Ok(dir) => dir,
            Err(_) => return false, // Can't determine home directory
        };

        let worktree_path = if parts.len() == 2 {
            // Default path: ~/.worktree/worktrees/project/worktree
            global_dir.join(parts[0]).join(parts[1])
        } else {
            // Single name - could be in custom directory, check global too
            global_dir.join(key)
        };

        let state_path = worktree_path.join("state.json");
        state_path.exists()
    }
}

/// Allocate ports for a worktree
pub fn allocate(
    count: u16,
    key: &str,
    range_start: u16,
    range_end: u16,
) -> Result<AllocationResult> {
    let mut allocations = PortAllocations::load()?;

    // Clean up stale allocations first
    allocations.cleanup_stale();

    // Check if already allocated
    if let Some(ports) = allocations.allocations.get(key) {
        return Ok(AllocationResult {
            ports: ports.clone(),
            existing: true,
        });
    }

    // Find free ports
    let excluded = allocations.all_allocated_ports();
    let ports =
        find_consecutive_free(count, range_start, range_end, &excluded).ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find {} consecutive free ports in range {}-{}",
                count,
                range_start,
                range_end
            )
        })?;

    // Save allocation
    allocations
        .allocations
        .insert(key.to_string(), ports.clone());
    allocations.save()?;

    Ok(AllocationResult {
        ports,
        existing: false,
    })
}

/// Deallocate ports for a worktree
pub fn deallocate(key: &str) -> Result<Option<Vec<u16>>> {
    let mut allocations = PortAllocations::load()?;

    let removed = allocations.allocations.remove(key);
    if removed.is_some() {
        allocations.save()?;
    }

    Ok(removed)
}

/// Result of a port allocation
#[derive(Debug)]
pub struct AllocationResult {
    pub ports: Vec<u16>,
    pub existing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocations_default() {
        let allocations = PortAllocations::default();
        assert!(allocations.allocations.is_empty());
    }

    #[test]
    fn test_port_allocations_all_allocated_ports() {
        let mut allocations = PortAllocations::default();
        allocations
            .allocations
            .insert("project1/wt1".to_string(), vec![50000, 50001, 50002]);
        allocations
            .allocations
            .insert("project2/wt2".to_string(), vec![50010, 50011]);

        let all_ports = allocations.all_allocated_ports();
        assert_eq!(all_ports.len(), 5);
        assert!(all_ports.contains(&50000));
        assert!(all_ports.contains(&50001));
        assert!(all_ports.contains(&50002));
        assert!(all_ports.contains(&50010));
        assert!(all_ports.contains(&50011));
    }

    #[test]
    fn test_port_allocations_json_roundtrip() {
        let mut allocations = PortAllocations::default();
        allocations
            .allocations
            .insert("test/worktree".to_string(), vec![50000, 50001]);

        let json = serde_json::to_string(&allocations).unwrap();
        let parsed: PortAllocations = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.allocations.len(), 1);
        assert_eq!(
            parsed.allocations.get("test/worktree"),
            Some(&vec![50000, 50001])
        );
    }
}
