# DocPilot CLI Documentation

## Overview

DocPilot provides an intuitive command-line interface with comprehensive help documentation, command aliases, and enhanced error handling. The CLI is designed to be user-friendly for both beginners and advanced users.

## Key Features

### 🎯 Comprehensive Help System

- **Main help**: `docpilot --help` shows overview with examples
- **Command-specific help**: `docpilot <command> --help` shows detailed usage
- **Rich descriptions**: Each command includes purpose, examples, and usage patterns
- **Visual indicators**: Emojis help identify command types quickly

### 🔗 Command Aliases

Commands include intuitive aliases for faster typing:

| Primary Command | Aliases               | Purpose            |
| --------------- | --------------------- | ------------------ |
| `start`         | `begin`, `new`        | Start new session  |
| `stop`          | `end`, `finish`       | Stop session       |
| `pause`         | `hold`                | Pause monitoring   |
| `resume`        | `continue`, `unpause` | Resume monitoring  |
| `annotate`      | `add`, `comment`      | Add annotation     |
| `annotations`   | `list`, `show`        | List annotations   |
| `note`          | `n`                   | Quick note         |
| `explain`       | `exp`                 | Quick explanation  |
| `warn`          | `warning`, `alert`    | Quick warning      |
| `milestone`     | `mile`, `checkpoint`  | Quick milestone    |
| `config`        | `cfg`, `setup`        | Configure settings |
| `status`        | `info`, `stat`        | Show status        |

### 🚨 Enhanced Error Handling

- **Clear error messages**: Descriptive error descriptions with emojis
- **Actionable suggestions**: Specific steps to resolve issues
- **Context-aware help**: Different suggestions based on error type
- **Graceful degradation**: Helpful fallbacks when operations fail

## Command Categories

### Session Management

```bash
# Start documenting a workflow
docpilot start "Setting up development environment"
docpilot begin "Database migration" --output migration.md

# Control session state
docpilot pause     # Temporarily stop monitoring
docpilot resume    # Continue monitoring
docpilot stop      # End and save session

# Check session information
docpilot status    # Detailed session info
docpilot info      # Alias for status
```

### Annotations

```bash
# General annotation with type
docpilot annotate "Configuring database" --annotation-type explanation
docpilot add "Requires admin privileges" -a warning

# Quick annotation shortcuts
docpilot note "Starting backup process"
docpilot n "Server responding slowly"
docpilot explain "This rebuilds the search index"
docpilot exp "Using this approach for edge cases"
docpilot warn "This will delete all data"
docpilot alert "Requires admin privileges"
docpilot milestone "Database migration complete"
docpilot checkpoint "All tests passing"

# View annotations
docpilot annotations                    # All annotations
docpilot list --recent 5               # Last 5 annotations
docpilot show --filter-type warning    # Only warnings
```

### Configuration

```bash
# View current configuration
docpilot config
docpilot cfg

# Set up AI provider
docpilot config --provider claude --api-key sk-...
docpilot setup -p chatgpt -a your-api-key
```

## Help Examples

### Main Help Output

```
docpilot 0.1.0
DocPilot automatically captures and documents your terminal workflows...

Usage: docpilot <COMMAND>

Commands:
  start        🚀 Start a new documentation session
  stop         🛑 Stop the current documentation session
  pause        ⏸️ Pause the current documentation session
  resume       ▶️ Resume a paused documentation session
  annotate     📝 Add a manual annotation to the current session
  annotations  📋 List all annotations in the current session
  note         📝 Quick note annotation
  explain      💡 Quick explanation annotation
  warn         ⚠️ Quick warning annotation
  milestone    🎯 Quick milestone annotation
  config       ⚙️ Configure LLM settings
  status       📊 Show current session status
  help         Print this message or the help of the given subcommand(s)

EXAMPLES:
    # Start documenting a new workflow
    docpilot start "Setting up development environment"

    # Add annotations while working
    docpilot note "Installing dependencies"
    docpilot warn "This requires admin privileges"

    # Check session status
    docpilot status

    # Stop and save documentation
    docpilot stop
```

### Command-Specific Help

```bash
$ docpilot start --help
Begin monitoring terminal commands and start documenting your workflow.

This command creates a new session that will capture all terminal commands,
allowing you to add annotations and generate comprehensive documentation.

EXAMPLES:
    docpilot start "Setting up development environment"
    docpilot start "Database migration process" --output migration-guide.md
    docpilot begin "API testing workflow" -o api-tests.md

Usage: docpilot start [OPTIONS] --description <DESCRIPTION>

Options:
  -d, --description <DESCRIPTION>
          Describe what workflow you're documenting

  -o, --output <OUTPUT>
          Specify output markdown file (e.g., guide.md)
```

## Error Handling Examples

### No Active Session

```bash
$ docpilot note "Testing"
❌ Failed to add note: No active session for annotation
   Start a session first with 'docpilot start "description"'
   Then add annotations to document your workflow
```

### Invalid Annotation Type

```bash
$ docpilot annotate "Test" --annotation-type invalid
❌ Invalid annotation type: 'invalid'

✅ Valid annotation types:
   • note        - General observations and context
   • explanation - Detailed explanations of processes
   • warning     - Important warnings and cautions
   • milestone   - Significant progress markers

💡 Example: docpilot annotate "Database connected" --annotation-type milestone
```

### Session Already Active

```bash
$ docpilot start "New session"
❌ Failed to start session: Session already active

🔍 Possible causes:
   • Another session is already active
   • Insufficient permissions to create session files
   • Invalid output file path specified

💡 Try these solutions:
   • Check current status: docpilot status
   • Stop existing session: docpilot stop
   • Use a different output file name
```

## Design Principles

### User Experience

- **Progressive disclosure**: Basic commands are simple, advanced features available when needed
- **Consistent patterns**: Similar commands follow similar syntax patterns
- **Visual feedback**: Emojis and formatting make output scannable
- **Helpful defaults**: Sensible default values reduce required parameters

### Error Recovery

- **Specific diagnostics**: Errors explain exactly what went wrong
- **Actionable guidance**: Every error includes suggested next steps
- **Context awareness**: Suggestions adapt to current application state
- **Graceful fallbacks**: Partial failures don't break the entire workflow

### Accessibility

- **Multiple input methods**: Aliases accommodate different user preferences
- **Clear documentation**: Every feature is documented with examples
- **Consistent terminology**: Same concepts use same words throughout
- **Logical grouping**: Related commands are organized together

## Implementation Details

### Command Structure

- Built with `clap` for robust argument parsing
- Subcommand-based architecture for clear organization
- Rich help templates with custom formatting
- Comprehensive validation and error handling

### Help System

- Long descriptions with examples for each command
- Custom help templates with consistent formatting
- Context-sensitive help based on command usage
- Progressive disclosure from basic to advanced features

### Error Handling

- Structured error types with specific handling
- User-friendly error messages with technical details
- Actionable suggestions based on error context
- Graceful degradation when operations fail

This CLI design ensures DocPilot is approachable for new users while providing the power and flexibility that advanced users need.
