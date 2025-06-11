# DocPilot Shell Integration Guide

## Overview

DocPilot uses an advanced shell integration system to capture commands in real-time from your active terminal session. This approach eliminates the problems with shell history file monitoring and ensures that only commands from your current DocPilot session are captured.

## How It Works

### 1. Automatic Hook Generation

When you start a DocPilot session, it automatically creates shell-specific hook files in `~/.docpilot/`:

- **Zsh**: `~/.docpilot/zsh_hooks.zsh`
- **Bash**: `~/.docpilot/bash_hooks.bash`
- **Fish**: `~/.docpilot/fish_hooks.fish`

### 2. Shell-Specific Command Capture

Each shell uses its native mechanisms for command capture:

#### Zsh Integration

- Uses `preexec()` function to capture commands before execution
- Uses `precmd()` function to capture exit codes after execution
- Preserves existing preexec/precmd functions if they exist
- Format: `timestamp|working_dir|exit_code|command`

#### Bash Integration

- Modifies `PROMPT_COMMAND` to capture commands from history
- Preserves existing PROMPT_COMMAND if it exists
- Uses `history 1` to get the most recent command
- Format: `timestamp|working_dir|exit_code|command`

#### Fish Integration

- Uses event-based functions for command capture
- `fish_preexec` event captures commands before execution
- `fish_postexec` event captures exit codes after execution
- Format: `timestamp|working_dir|exit_code|command`

### 3. Real-Time Log Monitoring

- Commands are written to temporary log files in `/tmp/`
- DocPilot monitors these files for new entries
- Only commands with timestamps after the session start are included
- Automatic cleanup when sessions end

## Auto-Sourcing vs Manual Sourcing

### Auto-Sourcing (Recommended)

DocPilot now supports **automatic shell hook sourcing** using the `eval` command approach:

```bash
# Start session
docpilot start "My workflow"

# Auto-source hooks (displayed in DocPilot output)
eval "$(./target/release/docpilot hooks SESSION_ID)"
```

**Benefits of Auto-Sourcing:**
- **No file paths to remember** - command is self-contained
- **Session-specific** - automatically includes correct session ID
- **Works from any directory** - no need to navigate to hook files
- **Clean output** - only outputs shell hooks, no extra messages
- **Shell agnostic** - works with Zsh, Bash, and Fish

### Manual Sourcing (Fallback)

Traditional approach using direct file sourcing:

```bash
# Start session
docpilot start "My workflow"

# Manual sourcing
source ~/.docpilot/zsh_hooks.zsh    # For Zsh
source ~/.docpilot/bash_hooks.bash  # For Bash
source ~/.docpilot/fish_hooks.fish  # For Fish
```

**When to use Manual Sourcing:**
- Auto-sourcing command is not available
- Working with older DocPilot versions
- Debugging shell integration issues
- Custom shell integration setups

## Setup Process

### Quick Start

1. **Start a DocPilot session**:

```bash
docpilot start "My workflow documentation"
```

2. **Enable command capture** (one-time per shell session):

```bash
# Auto-sourcing (recommended):
eval "$(./target/release/docpilot hooks SESSION_ID)"

# Alternative (manual sourcing):
source ~/.docpilot/zsh_hooks.zsh
```

3. **Run your commands normally** - they'll be captured automatically:

```bash
npm install
npm run build
docker build -t myapp .
```

4. **Stop the session**:

```bash
docpilot stop
```

### Detailed Workflow

#### For Zsh Users

```bash
# 1. Start DocPilot
docpilot start "Setting up development environment"

# 2. DocPilot creates ~/.docpilot/zsh_hooks.zsh and shows:
# 🚨 IMPORTANT: To capture commands in THIS shell session, run:
#    eval "$(./target/release/docpilot hooks SESSION_ID)"

# 3. Enable command capture (auto-sourcing)
eval "$(./target/release/docpilot hooks SESSION_ID)"

# 4. Your commands are now captured automatically
git clone https://github.com/example/repo.git
cd repo
npm install
npm test

# 5. Stop session and generate documentation
docpilot stop
docpilot generate --output setup-guide.md
```

#### For Bash Users

```bash
# 1. Start DocPilot
docpilot start "Database migration process"

# 2. Enable command capture (auto-sourcing)
eval "$(./target/release/docpilot hooks SESSION_ID)"

# 3. Run your commands
mysql -u root -p < migration.sql
systemctl restart mysql
mysql -u root -p -e "SHOW DATABASES;"

# 4. Stop and generate
docpilot stop
docpilot generate --output migration-guide.md
```

## Advanced Usage

### Automatic Setup for New Shells

To automatically enable DocPilot command capture in new shell sessions, add this to your shell configuration file:

#### For Zsh (`~/.zshrc`):

```bash
# DocPilot auto-setup
[[ -f ~/.docpilot/zsh_hooks.zsh ]] && source ~/.docpilot/zsh_hooks.zsh
```

#### For Bash (`~/.bashrc`):

```bash
# DocPilot auto-setup
[[ -f ~/.docpilot/bash_hooks.bash ]] && source ~/.docpilot/bash_hooks.bash
```

#### For Fish (`~/.config/fish/config.fish`):

```fish
# DocPilot auto-setup
if test -f ~/.docpilot/fish_hooks.fish
    source ~/.docpilot/fish_hooks.fish
end
```

### Multiple Terminal Sessions

Each terminal session needs to source the hooks independently:

```bash
# Terminal 1
docpilot start "Frontend development"
eval "$(./target/release/docpilot hooks SESSION_ID)"
npm run dev

# Terminal 2 (same session)
eval "$(./target/release/docpilot hooks SESSION_ID)"
npm run test

# Both terminals will contribute to the same DocPilot session
```

### Session Isolation

Each DocPilot session creates unique log files:

- Session ID: `1a65c01b-7c94-4bdf-8a11-0350313dbc89`
- Log file: `/tmp/docpilot_commands_1a65c01b-7c94-4bdf-8a11-0350313dbc89.log`
- Hooks file: `~/.docpilot/zsh_hooks.zsh` (updated for each session)

This ensures that:

- Commands from different sessions don't mix
- Only commands after session start are captured
- No contamination from shell history files

## Troubleshooting

### Commands Not Being Captured

1. **Check if hooks are sourced**:

```bash
# For Zsh, check if functions exist:
type preexec
type precmd

# For Bash, check PROMPT_COMMAND:
echo $PROMPT_COMMAND
```

2. **Verify log file is being written**:

```bash
# Check if log file exists and is growing
ls -la /tmp/docpilot_commands_*.log
tail -f /tmp/docpilot_commands_*.log
```

3. **Check DocPilot session status**:

```bash
docpilot status
# Should show "Commands captured: X" increasing
```

### Hook Conflicts

If you have existing shell hooks, DocPilot preserves them:

- **Zsh**: Saves existing `preexec`/`precmd` as `docpilot_original_*`
- **Bash**: Preserves existing `PROMPT_COMMAND`
- **Fish**: Uses separate event functions

### Cleanup Issues

If hooks aren't cleaned up properly:

```bash
# Manual cleanup for Zsh
unset -f preexec precmd
unset -f docpilot_original_preexec docpilot_original_precmd

# Manual cleanup for Bash
export PROMPT_COMMAND="$DOCPILOT_ORIGINAL_PROMPT_COMMAND"
unset DOCPILOT_ORIGINAL_PROMPT_COMMAND

# Manual cleanup for Fish
functions -e docpilot_log_command
functions -e docpilot_log_exit
```

## Security Considerations

### Sensitive Data Protection

- Commands containing sensitive data are logged to temporary files
- DocPilot includes privacy filtering for final documentation
- Log files are cleaned up when sessions end
- Consider using `--exclude-patterns` for sensitive commands

### File Permissions

- Hook files: `~/.docpilot/` (user-readable only)
- Log files: `/tmp/docpilot_commands_*` (user-readable only)
- Automatic cleanup on session end

### Network Commands

SSH sessions and remote commands are captured as local commands:

```bash
ssh user@server "ls -la"  # Captured as local command
# Remote commands within SSH session are not captured
```

## Best Practices

### 1. Session Management

- Use descriptive session names
- Stop sessions when workflows are complete
- Use annotations for non-terminal activities

### 2. Shell Integration

- Source hooks once per terminal session
- Check `docpilot status` to verify capture is working
- Use foreground mode for debugging if needed

### 3. Documentation Quality

- Add annotations for context
- Use meaningful session descriptions
- Generate documentation promptly after sessions

### 4. Performance

- Sessions automatically clean up temporary files
- Use `docpilot pause` for long breaks
- Monitor disk space in `/tmp/` for long sessions

## Examples

### Complete Workflow Example

```bash
# 1. Start documenting
docpilot start "Deploying React app to production"

# 2. Enable command capture (auto-sourcing)
eval "$(./target/release/docpilot hooks SESSION_ID)"

# 3. Your workflow (captured automatically)
npm run build
docker build -t myapp:latest .
docker tag myapp:latest registry.example.com/myapp:latest
docker push registry.example.com/myapp:latest

# 4. Add context annotations
docpilot note "Build completed successfully"
docpilot warn "Ensure environment variables are set in production"
docpilot milestone "Application deployed to production"

# 5. Verify capture is working
docpilot status
# Should show: Commands captured: 4

# 6. Stop and generate documentation
docpilot stop
docpilot generate --output deployment-guide.md

# 7. Hooks are automatically cleaned up
```

This shell integration system provides reliable, real-time command capture without the issues of shell history file monitoring, ensuring that your DocPilot documentation accurately reflects your actual workflow.
