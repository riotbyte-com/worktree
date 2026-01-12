---
description: Manage git worktrees with port reservation and lifecycle management
argument-hint: [init|run|stop|close|list|<param>]
allowed-tools: Bash, Read, Write, AskUserQuestion
---

# Worktree Management

You are managing git worktrees for isolated development environments. Parse `$ARGUMENTS` to determine the action:

**Arguments received:** $ARGUMENTS

## Action Routing

1. If `$ARGUMENTS` is "init" → Execute the **Init** section
2. If `$ARGUMENTS` is "run" → Execute the **Run** section
3. If `$ARGUMENTS` is "stop" → Execute the **Stop** section
4. If `$ARGUMENTS` is "close" → Execute the **Close** section
5. If `$ARGUMENTS` is "list" → Execute the **List** section
6. Otherwise → Execute the **Create Worktree** section (use `$ARGUMENTS` as parameter if provided)

---

## Init

Initialize worktree configuration for this project. Checks prerequisites and creates configuration files.

### Steps:

1. **Check prerequisites**
   ```bash
   python3 ~/.claude/scripts/worktree/init.py --check-only
   ```
   If this fails (exit code 1), show the output and stop. The user needs to install missing dependencies.

2. **Check if already initialized**
   ```bash
   python3 ~/.claude/scripts/worktree/init.py
   ```
   If exit code is 2, it's already initialized. Show the warning and stop.
   If exit code is 0, continue with the output JSON which contains defaults.

3. **Ask user for worktree directory preference**
   Use AskUserQuestion to ask:
   - Question: "Where should worktrees be created?"
   - Options:
     - "Default (~/.claude/worktrees)" - Recommended
     - "Custom path" - User will provide a custom directory path

   If user selects custom, ask for the path.

4. **Ask user for additional settings** (optional)
   Use AskUserQuestion to ask:
   - Question: "Configure additional settings?"
   - Options:
     - "Use defaults" - Use default port count (10), port range (50000-60000), branch prefix (worktree/)
     - "Customize" - Configure port count, port range, branch prefix

   If user selects customize, ask for each setting.

5. **Create configuration directory**
   ```bash
   mkdir -p .claude/worktree
   ```

6. **Create settings.json** (shared team settings)
   Write `.claude/worktree/settings.json` with team-shareable defaults:
   ```json
   {
     "portCount": 10,
     "portRangeStart": 50000,
     "portRangeEnd": 60000,
     "branchPrefix": "worktree/",
     "autoLaunchTerminal": true
   }
   ```

7. **Create settings.local.json** (personal settings, gitignored)
   Write `.claude/worktree/settings.local.json` with user-specific settings:
   ```json
   {
     "worktreeDir": "<user's chosen directory>"
   }
   ```

8. **Create .gitignore**
   Write `.claude/worktree/.gitignore`:
   ```
   # Local settings (user-specific paths)
   settings.local.json
   ```

9. **Copy documentation**
   Copy the README template:
   ```bash
   cp ~/.claude/scripts/worktree/README.template.md .claude/worktree/README.md
   ```

10. **Create template files**
    Create placeholder scripts that the user can customize:

    `.claude/worktree/SETUP.md`:
    ```markdown
    # Worktree Parameter Handling

    This file instructs Claude how to handle parameters passed to `/worktree <param>`.

    ## Instructions

    When a parameter is provided (e.g., `/worktree CHR-12`):

    1. If the parameter matches a Linear issue ID pattern (e.g., CHR-12, ABC-123):
       - Fetch the issue details using Linear MCP
       - Use the issue's `gitBranchName` as the branch name
       - If no gitBranchName, use: `feature/<issue-id>-<slugified-title>`

    2. If the parameter is a branch name:
       - Use it directly as the branch name

    3. Otherwise:
       - Ignore the parameter and generate a random branch name
    ```

11. **Report progress**
    Show what was created so far:
    - `.claude/worktree/settings.json` - Team settings
    - `.claude/worktree/settings.local.json` - Personal settings (gitignored)
    - `.claude/worktree/.gitignore`
    - `.claude/worktree/README.md` - Documentation
    - `.claude/worktree/SETUP.md` - Parameter handling instructions

12. **Ask about script generation**
    Use AskUserQuestion to ask:
    - Question: "Would you like me to analyze this project and generate worktree scripts?"
    - Options:
      - "Yes, generate scripts" - Claude will analyze the project and create setup.sh, run.sh, stop.sh, close.sh
      - "No, I'll create them manually" - Skip script generation

    If user selects "No", skip to step 14.

13. **Generate scripts** (if user said yes)

    Analyze the project to understand its structure, language, and tooling. Look for:
    - Package manager files (package.json, composer.json, requirements.txt, Gemfile, go.mod, Cargo.toml, etc.)
    - Lock files to determine exact package manager (bun.lockb, pnpm-lock.yaml, yarn.lock, composer.lock, etc.)
    - Environment files (.env.example, .env.local.example, .env.sample, etc.)
    - Docker/container files (docker-compose.yml, Dockerfile, etc.)
    - Configuration files for frameworks and tools
    - README or documentation that explains how to run the project

    Based on what you find, create these scripts:

    **`.claude/worktree/setup.sh`** - Runs after worktree is created:
    - Install dependencies using the project's package manager
    - Copy/create environment files from templates
    - Update environment variables with allocated ports (WORKTREE_PORT_0 through WORKTREE_PORT_9)
    - Run any database migrations or setup commands
    - Any other initialization the project needs

    **`.claude/worktree/run.sh`** - Starts the development environment:
    - Start the dev server/application on allocated port
    - Start any required services (databases, queues, etc.)
    - Use the port environment variables (WORKTREE_PORT_0, etc.)

    **`.claude/worktree/stop.sh`** - Stops running services:
    - Stop the dev server
    - Stop any background services
    - Kill processes on allocated ports if needed

    **`.claude/worktree/close.sh`** - Cleanup before worktree deletion:
    - Stop any running services
    - Clean up temporary files if needed
    - Any project-specific cleanup

    Make all scripts executable:
    ```bash
    chmod +x .claude/worktree/*.sh
    ```

    Show the generated scripts to the user and explain what each does.

14. **Report final success**
    Show complete summary of what was created and next steps.

---

## Create Worktree (default action)

Create a new git worktree with port reservation.

### Steps:

1. **Verify git repository**
   ```bash
   git rev-parse --git-dir
   ```
   If this fails, inform the user they must be in a git repository.

2. **Load project settings** (if exists)
   Check if `.claude/worktree/settings.json` exists:
   ```bash
   cat .claude/worktree/settings.json 2>/dev/null
   cat .claude/worktree/settings.local.json 2>/dev/null
   ```
   Merge settings (local overrides shared). Use these defaults if no settings:
   - `worktreeDir`: `~/.claude/worktrees` (default, will include project subdirectory)
   - `portCount`: 10
   - `portRangeStart`: 50000
   - `portRangeEnd`: 60000
   - `branchPrefix`: `worktree/`
   - `autoLaunchTerminal`: true

3. **Check for uncommitted worktree configuration**
   If `.claude/worktree/` exists, check if there are uncommitted files:
   ```bash
   git status --porcelain .claude/worktree/ 2>/dev/null
   ```

   If there are uncommitted files (output is not empty), warn the user with a clear explanation:

   **Explain the impact:**
   - Git worktrees only include committed files
   - Uncommitted scripts (setup.sh, run.sh, stop.sh, close.sh) won't exist in the new worktree
   - This means: no automatic dependency installation, no environment setup, no port configuration
   - The worktree will be a bare checkout without any of your configured automation

   **List the uncommitted files** from the git status output so the user knows exactly what's missing.

   Use AskUserQuestion to ask:
   - Question: "There are uncommitted worktree configuration files. Without committing these, the new worktree won't have your setup scripts - no automatic dependency installation, environment configuration, or port setup will run. What would you like to do?"
   - Options:
     - "Commit for me" (Recommended) - Commit the worktree configuration files and continue
     - "Continue anyway" - Create a bare worktree without setup automation
     - "Cancel" - Stop so I can review/modify the files first

   If user selects "Commit for me":
   ```bash
   git add .claude/worktree/
   git commit -m "Add worktree configuration"
   ```
   Then continue with step 4.

   If user selects "Cancel", stop with a message:
   ```
   Worktree creation cancelled. When you're ready, run /worktree again.
   ```

4. **Determine project name**
   Get the project name from the git repository:
   ```bash
   basename $(git rev-parse --show-toplevel)
   ```
   This will be used to organize worktrees by project.

5. **Generate random worktree name**
   Use a random adjective-noun combination with a short random suffix (e.g., `swift-falcon-a3b2`).

6. **Check for parameter handling instructions**
   If `$ARGUMENTS` is not empty (and not a subcommand), check if `.claude/worktree/SETUP.md` exists in the current project:
   ```bash
   cat .claude/worktree/SETUP.md 2>/dev/null
   ```
   If it exists, read and follow its instructions for handling the parameter (e.g., using it as a Linear issue ID to fetch branch name).
   If it doesn't exist, ignore the parameter.

7. **Determine branch name**
   - If SETUP.md provided instructions and a branch name was derived, use that
   - Otherwise, use settings `branchPrefix` + random name (e.g., `worktree/swift-falcon-a3b2`)

8. **Get the original directory**
   ```bash
   pwd
   ```

9. **Determine worktree path**
   - If using **default** worktree directory (`~/.claude/worktrees`):
     - Path: `~/.claude/worktrees/<projectName>/<worktreeName>`
     - This organizes worktrees by project
   - If using **custom** worktree directory (from `settings.local.json`):
     - Path: `<customDir>/<worktreeName>`
     - Custom directories are project-specific, so no project subdirectory needed

10. **Create the worktree**
    ```bash
    mkdir -p <worktreeDir>  # Ensure parent directory exists
    git worktree add <fullWorktreePath> -b <branch>
    ```

11. **Allocate ports**
    Use `portCount` from settings. Use `<projectName>/<worktreeName>` as the allocation key for default dir, or just `<worktreeName>` for custom dir:
    ```bash
    python3 ~/.claude/scripts/worktree/allocate-ports.py <portCount> <allocationKey>
    ```
    This returns JSON with the allocated ports.

12. **Write state.json**
    Create `<fullWorktreePath>/state.json` with:
    ```json
    {
      "name": "<worktreeName>",
      "projectName": "<projectName>",
      "originalDir": "<original directory>",
      "worktreeDir": "<fullWorktreePath>",
      "branch": "<branch>",
      "ports": [<allocated ports>],
      "allocationKey": "<allocationKey>",
      "createdAt": "<ISO timestamp>"
    }
    ```

13. **Run project setup script** (if exists)
    Check if `.claude/worktree/setup.sh` exists in the worktree and is executable:
    ```bash
    if [ -x <fullWorktreePath>/.claude/worktree/setup.sh ]; then
      WORKTREE_NAME=<worktreeName> \
      WORKTREE_PROJECT=<projectName> \
      WORKTREE_DIR=<fullWorktreePath> \
      WORKTREE_ORIGINAL_DIR=<original> \
      WORKTREE_PORT_0=<port0> WORKTREE_PORT_1=<port1> ... \
      WORKTREE_PARAM="$ARGUMENTS" \
      <fullWorktreePath>/.claude/worktree/setup.sh
    fi
    ```

14. **Launch new terminal** (if `autoLaunchTerminal` is true)
    ```bash
    ~/.claude/scripts/worktree/open-terminal.sh <fullWorktreePath>
    ```

15. **Report success** with the worktree name, project, and allocated ports.

---

## Run

Execute the project's run script with allocated ports.

### Steps:

1. **Detect worktree**
   ```bash
   ~/.claude/scripts/worktree/detect-worktree.sh
   ```
   This outputs the worktree name if current directory is within a worktree.

2. **Read state.json**
   ```bash
   cat ~/.claude/worktrees/<name>/state.json
   ```

3. **Check for run script**
   ```bash
   ls -la .claude/worktree/run.sh
   ```

4. **Execute run script** with environment variables:
   ```bash
   WORKTREE_NAME=<name> \
   WORKTREE_DIR=<dir> \
   WORKTREE_ORIGINAL_DIR=<original> \
   WORKTREE_PORT_0=<port0> WORKTREE_PORT_1=<port1> ... \
   .claude/worktree/run.sh
   ```

---

## Stop

Execute the project's stop script.

### Steps:

1. **Detect worktree** (same as Run)

2. **Check for stop script**
   ```bash
   ls -la .claude/worktree/stop.sh
   ```

3. **Execute stop script**:
   ```bash
   .claude/worktree/stop.sh
   ```

---

## Close

Clean up and delete the worktree.

### Steps:

1. **Detect worktree**
   ```bash
   ~/.claude/scripts/worktree/detect-worktree.sh
   ```

2. **Read state.json** to get original directory and worktree info.

3. **Execute close script** (if exists):
   ```bash
   if [ -x .claude/worktree/close.sh ]; then
     .claude/worktree/close.sh
   fi
   ```

4. **Deallocate ports**:
   ```bash
   python3 ~/.claude/scripts/worktree/deallocate-ports.py <name>
   ```

5. **Remove worktree**:
   ```bash
   cd ~ && git -C <original_dir> worktree remove ~/.claude/worktrees/<name> --force
   ```

6. **Report success** and provide command to return to original project:
   ```
   Worktree closed. To return to original project:
   cd <original_dir> && claude
   ```

---

## List

Show all active worktrees and their port allocations.

### Steps:

1. **Run list script**:
   ```bash
   python3 ~/.claude/scripts/worktree/list.py
   ```

2. **Display the output** formatted as a table.
