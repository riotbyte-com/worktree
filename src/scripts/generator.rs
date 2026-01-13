use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Check if Claude CLI is available
pub fn is_claude_available() -> bool {
    which::which("claude").is_ok()
}

/// Generate lifecycle scripts using Claude CLI with interactive output
pub fn generate_with_claude(project_dir: &Path) -> Result<GeneratedScripts> {
    let prompt = r#"Analyze this project and generate four shell scripts for worktree management.
The scripts will receive these environment variables:
- WORKTREE_NAME: unique worktree identifier
- WORKTREE_PROJECT: project name
- WORKTREE_DIR: path to worktree directory
- WORKTREE_ORIGINAL_DIR: path to original project
- WORKTREE_PORT_0 through WORKTREE_PORT_9: allocated ports

Generate these scripts:
1. setup.sh - Install dependencies, copy/generate .env files using ports, run migrations
2. run.sh - Start development server(s) using allocated ports
3. stop.sh - Stop all running services gracefully
4. close.sh - Final cleanup before worktree deletion

Output each script content between markers like:
=== setup.sh ===
(script content)
=== end setup.sh ===

Make scripts executable and include proper error handling."#;

    // Create a temp file to capture output while also displaying it
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join(format!("worktree-claude-{}.txt", std::process::id()));

    // Run Claude with output going to both terminal and file
    // Using a pipe to read output line by line
    let mut child = Command::new("claude")
        .args(["--print", prompt])
        .current_dir(project_dir)
        .stdin(Stdio::inherit()) // Allow user input for permission prompts
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // Show errors directly
        .spawn()
        .context("Failed to start claude CLI")?;

    // Read stdout and both display it and save to file
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut file = File::create(&output_file).context("Failed to create temp file")?;

    let mut output_buffer = String::new();

    println!();
    for line in reader.lines() {
        let line = line.context("Failed to read line from claude")?;
        // Print to terminal
        println!("{}", line);
        // Save to buffer
        output_buffer.push_str(&line);
        output_buffer.push('\n');
        // Also write to file for debugging
        writeln!(file, "{}", line)?;
    }

    let status = child.wait().context("Failed to wait for claude CLI")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&output_file);

    if !status.success() {
        anyhow::bail!("Claude CLI failed with exit code: {:?}", status.code());
    }

    parse_generated_scripts(&output_buffer)
}

/// Parse Claude's output to extract scripts
fn parse_generated_scripts(output: &str) -> Result<GeneratedScripts> {
    Ok(GeneratedScripts {
        setup_sh: extract_script(output, "setup.sh").unwrap_or_else(default_setup_script),
        run_sh: extract_script(output, "run.sh").unwrap_or_else(default_run_script),
        stop_sh: extract_script(output, "stop.sh").unwrap_or_else(default_stop_script),
        close_sh: extract_script(output, "close.sh").unwrap_or_else(default_close_script),
    })
}

fn extract_script(output: &str, name: &str) -> Option<String> {
    let start_marker = format!("=== {} ===", name);
    let end_marker = format!("=== end {} ===", name);

    let start = output.find(&start_marker)?;
    let end = output.find(&end_marker)?;

    if start >= end {
        return None;
    }

    let content = &output[start + start_marker.len()..end];
    Some(content.trim().to_string())
}

/// Generate template scripts without Claude
pub fn generate_templates() -> GeneratedScripts {
    GeneratedScripts {
        setup_sh: default_setup_script(),
        run_sh: default_run_script(),
        stop_sh: default_stop_script(),
        close_sh: default_close_script(),
    }
}

fn default_setup_script() -> String {
    r#"#!/bin/bash
# Setup script for worktree: $WORKTREE_NAME
# This script runs after the worktree is created

set -e

echo "Setting up worktree: $WORKTREE_NAME"
echo "Allocated ports: $WORKTREE_PORT_0 - $WORKTREE_PORT_9"

# TODO: Add your setup commands here
# Examples:
# - npm install
# - cp .env.example .env
# - Update .env with allocated ports
# - Run database migrations

echo "Setup complete!"
"#
    .to_string()
}

fn default_run_script() -> String {
    r#"#!/bin/bash
# Run script for worktree: $WORKTREE_NAME
# This script starts the development environment

set -e

echo "Starting development environment for: $WORKTREE_NAME"
echo "Using port: $WORKTREE_PORT_0"

# TODO: Add your run commands here
# Examples:
# - npm run dev -- --port $WORKTREE_PORT_0
# - docker-compose up -d
# - Start your dev server

echo "Development environment started!"
"#
    .to_string()
}

fn default_stop_script() -> String {
    r#"#!/bin/bash
# Stop script for worktree: $WORKTREE_NAME
# This script stops running services

echo "Stopping services for: $WORKTREE_NAME"

# TODO: Add your stop commands here
# Examples:
# - pkill -f "node.*$WORKTREE_PORT_0"
# - docker-compose down
# - Stop your dev server

echo "Services stopped!"
"#
    .to_string()
}

fn default_close_script() -> String {
    r#"#!/bin/bash
# Close script for worktree: $WORKTREE_NAME
# This script performs final cleanup before deletion

echo "Cleaning up worktree: $WORKTREE_NAME"

# TODO: Add your cleanup commands here
# Examples:
# - docker-compose down -v
# - Drop test databases
# - Remove cached files

echo "Cleanup complete!"
"#
    .to_string()
}

/// Generated scripts container
#[derive(Debug, Default)]
pub struct GeneratedScripts {
    pub setup_sh: String,
    pub run_sh: String,
    pub stop_sh: String,
    pub close_sh: String,
}

impl GeneratedScripts {
    /// Write scripts to the project config directory
    pub fn write_to(&self, config_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(config_dir)?;

        write_executable(config_dir.join("setup.sh"), &self.setup_sh)?;
        write_executable(config_dir.join("run.sh"), &self.run_sh)?;
        write_executable(config_dir.join("stop.sh"), &self.stop_sh)?;
        write_executable(config_dir.join("close.sh"), &self.close_sh)?;

        Ok(())
    }
}

fn write_executable(path: std::path::PathBuf, content: &str) -> Result<()> {
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}
