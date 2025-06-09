# Session Persistence and Recovery

## Overview

DocPilot now includes robust session persistence and recovery mechanisms to ensure data integrity and prevent loss of documentation work. The enhanced system provides automatic backups, corruption detection, session validation, and comprehensive recovery options.

## Key Features

### 🔄 Enhanced Persistence

- **Atomic writes**: Sessions are written to temporary files first, then atomically renamed to prevent corruption
- **Automatic backups**: Previous session versions are automatically backed up before overwriting
- **Configurable backup retention**: Keeps a configurable number of backups per session (default: 5)
- **Auto-save functionality**: Periodic automatic saving with configurable intervals (default: 30 seconds)

### 🛡️ Data Protection

- **Backup creation**: Automatic timestamped backups before each save operation
- **Corruption detection**: Validation of session data integrity on load
- **Recovery mechanisms**: Multiple fallback strategies for data recovery
- **Session validation**: Comprehensive checks for data consistency

### 🔧 Recovery Capabilities

- **Automatic recovery**: Detects and recovers interrupted sessions on startup
- **Backup recovery**: Falls back to most recent backup if main session file is corrupted
- **Session validation**: Ensures recovered sessions meet integrity requirements
- **Multiple recovery attempts**: Tries multiple backups if recent ones are corrupted

## Architecture

### Session Storage Structure

```
~/.docpilot/
├── sessions/           # Main session files
│   ├── {session-id}.json
│   └── {session-id}.tmp  # Temporary files during writes
└── backups/            # Session backups
    ├── {session-id}_{timestamp}.json
    └── {session-id}_{timestamp}.json
```

### Backup Strategy

- **Trigger**: Backup created before each save operation (if session file exists)
- **Naming**: `{session-id}_{unix-timestamp}.json`
- **Retention**: Configurable maximum number of backups per session
- **Cleanup**: Automatic removal of excess backups (oldest first)

### Auto-Save Mechanism

- **Interval**: Configurable auto-save interval (default: 30 seconds)
- **Trigger**: Automatic save when interval has elapsed since last save
- **Manual override**: Force save functionality available
- **Session state**: Only saves when session is in modifiable state

## API Reference

### SessionManager Methods

#### Persistence Methods

```rust
// Enhanced save with backup support
pub fn save_session(&mut self, session: &Session) -> Result<()>

// Force immediate save
pub fn force_save(&mut self) -> Result<()>

// Check and perform auto-save if needed
pub fn check_auto_save(&mut self) -> Result<bool>
```

#### Recovery Methods

```rust
// Enhanced recovery with backup fallback
pub fn recover_session(&mut self) -> Result<Option<String>>

// Load session with recovery attempts
fn load_session_with_recovery(&mut self, session_id: &str) -> Result<Session>

// Recover from backup files
fn recover_from_backup(&self, session_id: &str) -> Result<Session>

// Validate session integrity
fn validate_session(&self, session: &Session) -> bool
```

#### Backup Management

```rust
// Get backup information for a session
pub fn get_backup_info(&self, session_id: &str) -> Result<Vec<(PathBuf, SystemTime)>>

// Clean up old sessions and backups
pub fn cleanup_old_data(&self, max_age_days: u64) -> Result<usize>

// Get storage statistics
pub fn get_storage_stats(&self) -> Result<StorageStats>
```

#### Import/Export

```rust
// Export session to external file
pub fn export_session(&self, session_id: &str, export_path: &Path) -> Result<()>

// Import session from external file
pub fn import_session(&mut self, import_path: &Path) -> Result<String>
```

### Configuration Options

#### SessionManager Configuration

```rust
pub struct SessionManager {
    auto_save_interval: u64,    // Auto-save interval in seconds (default: 30)
    max_backups: usize,         // Maximum backups per session (default: 5)
    // ... other fields
}
```

#### Storage Statistics

```rust
pub struct StorageStats {
    pub session_count: usize,   // Number of session files
    pub backup_count: usize,    // Number of backup files
    pub total_size: u64,        // Total storage used (bytes)
    pub backup_size: u64,       // Storage used by backups (bytes)
}
```

## Usage Examples

### Basic Session Management

```rust
let mut manager = SessionManager::new()?;

// Start a session (automatically saved)
let session_id = manager.start_session("My workflow".to_string(), None)?;

// Add annotations (automatically saved)
manager.add_annotation("Step 1 complete".to_string(), AnnotationType::Note)?;

// Force save if needed
manager.force_save()?;

// Stop session (automatically saved)
let session = manager.stop_session()?;
```

### Recovery Operations

```rust
let mut manager = SessionManager::new()?;

// Attempt to recover interrupted sessions
if let Some(recovered_id) = manager.recover_session()? {
    println!("Recovered session: {}", recovered_id);
}

// Get backup information
let backups = manager.get_backup_info(&session_id)?;
println!("Found {} backups", backups.len());
```

### Maintenance Operations

```rust
// Clean up old data (older than 30 days)
let cleaned_count = manager.cleanup_old_data(30)?;
println!("Cleaned up {} old files", cleaned_count);

// Get storage statistics
let stats = manager.get_storage_stats()?;
println!("Sessions: {}, Backups: {}, Total size: {} bytes",
         stats.session_count, stats.backup_count, stats.total_size);
```

### Import/Export

```rust
// Export a session
manager.export_session(&session_id, Path::new("backup.json"))?;

// Import a session
let imported_id = manager.import_session(Path::new("backup.json"))?;
```

## Error Handling

### Recovery Scenarios

1. **Main session file corrupted**: Automatically attempts recovery from most recent backup
2. **All backups corrupted**: Reports failure with detailed error information
3. **No backups available**: Reports inability to recover
4. **Session validation failure**: Rejects invalid sessions and reports issues

### Validation Checks

- **Basic integrity**: Non-empty ID and description
- **Timestamp consistency**: Logical ordering of creation, start, and stop times
- **Statistics consistency**: Command counts match actual data
- **UUID validation**: Valid UUIDs for annotations and events

## Best Practices

### For Users

1. **Regular cleanup**: Periodically clean up old sessions to manage storage
2. **Monitor storage**: Check storage statistics to understand space usage
3. **Export important sessions**: Create external backups of critical documentation
4. **Validate recovery**: Test recovery procedures periodically

### For Developers

1. **Handle errors gracefully**: Always check return values from persistence operations
2. **Use auto-save**: Rely on auto-save for most scenarios, force-save only when necessary
3. **Validate before import**: Always validate imported sessions
4. **Monitor backup creation**: Ensure backups are being created as expected

## Performance Considerations

### Storage Efficiency

- **Backup rotation**: Automatic cleanup prevents unlimited backup accumulation
- **Atomic writes**: Temporary files prevent partial writes but require additional space
- **JSON format**: Human-readable but larger than binary formats

### Recovery Performance

- **Backup ordering**: Most recent backups tried first for faster recovery
- **Validation caching**: Session validation results could be cached for performance
- **Lazy loading**: Sessions loaded on-demand rather than all at startup

## Future Enhancements

### Planned Features

1. **Compression**: Compress backup files to save storage space
2. **Encryption**: Encrypt sensitive session data
3. **Remote backup**: Support for cloud storage backup destinations
4. **Incremental backups**: Only backup changed portions of sessions
5. **Recovery UI**: Interactive recovery interface for corrupted sessions

### Configuration Improvements

1. **Per-session backup policies**: Different backup strategies per session type
2. **Storage quotas**: Configurable limits on total storage usage
3. **Backup scheduling**: Time-based backup creation independent of saves
4. **Recovery preferences**: User-configurable recovery behavior

This enhanced persistence and recovery system ensures that DocPilot users never lose their documentation work, even in the face of system crashes, file corruption, or other unexpected issues.
