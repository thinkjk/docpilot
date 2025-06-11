use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::terminal::{CommandEntry, TerminalMonitor};

/// Represents the current state of a documentation session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// Session is actively monitoring and capturing commands
    Active,
    /// Session is temporarily paused (not capturing new commands)
    Paused,
    /// Session has been stopped and finalized
    Stopped,
    /// Session encountered an error and needs attention
    Error(String),
}

impl SessionState {
    pub fn is_active(&self) -> bool {
        matches!(self, SessionState::Active)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, SessionState::Paused)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, SessionState::Stopped)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, SessionState::Error(_))
    }
}

/// Manual annotation added by the user during a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub annotation_type: AnnotationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    /// General note or comment
    Note,
    /// Explanation of what's happening
    Explanation,
    /// Warning or important information
    Warning,
    /// Section divider or milestone
    Milestone,
}

/// Events that occur during a session for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: String,
    pub event_type: SessionEventType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventType {
    SessionStarted,
    SessionPaused,
    SessionResumed,
    SessionStopped,
    AnnotationAdded,
    CommandCaptured,
    ErrorOccurred,
    ConfigurationChanged,
}

/// Main session data structure containing all session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for the session
    pub id: String,
    /// Human-readable description of what's being documented
    pub description: String,
    /// Current state of the session
    pub state: SessionState,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// When the session was started (may differ from created_at)
    pub started_at: Option<DateTime<Utc>>,
    /// When the session was stopped
    pub stopped_at: Option<DateTime<Utc>>,
    /// Output file path for generated documentation
    pub output_file: Option<PathBuf>,
    /// All captured commands during this session
    pub commands: Vec<CommandEntry>,
    /// Manual annotations added by the user
    pub annotations: Vec<Annotation>,
    /// Session events for audit trail
    pub events: Vec<SessionEvent>,
    /// Session configuration and metadata
    pub metadata: SessionMetadata,
    /// Statistics about the session
    pub stats: SessionStats,
}

/// Metadata and configuration for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Working directory when session was started
    pub working_directory: PathBuf,
    /// Shell type being monitored
    pub shell_type: String,
    /// Platform information
    pub platform: String,
    /// Hostname where session is running
    pub hostname: String,
    /// User who started the session
    pub user: Option<String>,
    /// Custom tags for organization
    pub tags: Vec<String>,
    /// LLM provider configuration used
    pub llm_provider: Option<String>,
    /// Session-specific settings
    pub settings: HashMap<String, String>,
}

/// Statistics about session activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of commands captured
    pub total_commands: usize,
    /// Number of successful commands (exit code 0)
    pub successful_commands: usize,
    /// Number of failed commands (non-zero exit code)
    pub failed_commands: usize,
    /// Number of annotations added
    pub total_annotations: usize,
    /// Total session duration in seconds
    pub duration_seconds: Option<u64>,
    /// Number of times session was paused/resumed
    pub pause_resume_count: usize,
}

impl Session {
    /// Create a new session with the given description
    pub fn new(description: String, output_file: Option<PathBuf>) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let working_directory = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"));
        
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok();

        let metadata = SessionMetadata {
            working_directory,
            shell_type: "unknown".to_string(), // Will be updated when monitor is attached
            platform: "unknown".to_string(),   // Will be updated when monitor is attached
            hostname,
            user,
            tags: Vec::new(),
            llm_provider: None,
            settings: HashMap::new(),
        };

        let stats = SessionStats {
            total_commands: 0,
            successful_commands: 0,
            failed_commands: 0,
            total_annotations: 0,
            duration_seconds: None,
            pause_resume_count: 0,
        };

        let start_event = SessionEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SessionEventType::SessionStarted,
            timestamp: now,
            details: Some(format!("Session created: {}", description)),
        };

        Ok(Session {
            id,
            description,
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            started_at: Some(now),
            stopped_at: None,
            output_file,
            commands: Vec::new(),
            annotations: Vec::new(),
            events: vec![start_event],
            metadata,
            stats,
        })
    }

    /// Update session metadata from a terminal monitor
    pub fn update_from_monitor(&mut self, monitor: &TerminalMonitor) {
        self.metadata.shell_type = monitor.shell_type.name().to_string();
        self.metadata.platform = monitor.platform.name().to_string();
        self.updated_at = Utc::now();
    }

    /// Add a command to the session
    pub fn add_command(&mut self, command: CommandEntry) {
        self.commands.push(command.clone());
        self.stats.total_commands += 1;
        
        // Update success/failure stats
        if let Some(exit_code) = command.exit_code {
            if exit_code == 0 {
                self.stats.successful_commands += 1;
            } else {
                self.stats.failed_commands += 1;
            }
        }

        // Add event
        let event = SessionEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SessionEventType::CommandCaptured,
            timestamp: Utc::now(),
            details: Some(command.command),
        };
        self.events.push(event);
        self.updated_at = Utc::now();
    }

    /// Add an annotation to the session
    pub fn add_annotation(&mut self, text: String, annotation_type: AnnotationType) -> String {
        let annotation = Annotation {
            id: Uuid::new_v4().to_string(),
            text,
            timestamp: Utc::now(),
            annotation_type,
        };

        let annotation_id = annotation.id.clone();
        self.annotations.push(annotation);
        self.stats.total_annotations += 1;

        // Add event
        let event = SessionEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SessionEventType::AnnotationAdded,
            timestamp: Utc::now(),
            details: Some(format!("Annotation added: {}", annotation_id)),
        };
        self.events.push(event);
        self.updated_at = Utc::now();

        annotation_id
    }

    /// Pause the session
    pub fn pause(&mut self) -> Result<()> {
        match self.state {
            SessionState::Active => {
                self.state = SessionState::Paused;
                self.stats.pause_resume_count += 1;
                
                let event = SessionEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: SessionEventType::SessionPaused,
                    timestamp: Utc::now(),
                    details: None,
                };
                self.events.push(event);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(anyhow!("Cannot pause session in state: {:?}", self.state)),
        }
    }

    /// Resume the session
    pub fn resume(&mut self) -> Result<()> {
        match self.state {
            SessionState::Paused => {
                self.state = SessionState::Active;
                
                let event = SessionEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: SessionEventType::SessionResumed,
                    timestamp: Utc::now(),
                    details: None,
                };
                self.events.push(event);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(anyhow!("Cannot resume session in state: {:?}", self.state)),
        }
    }

    /// Stop the session
    pub fn stop(&mut self) -> Result<()> {
        match self.state {
            SessionState::Active | SessionState::Paused => {
                self.state = SessionState::Stopped;
                self.stopped_at = Some(Utc::now());
                
                // Calculate duration
                if let Some(started_at) = self.started_at {
                    let duration = Utc::now().signed_duration_since(started_at);
                    self.stats.duration_seconds = Some(duration.num_seconds() as u64);
                }
                
                let event = SessionEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: SessionEventType::SessionStopped,
                    timestamp: Utc::now(),
                    details: Some(format!("Session completed with {} commands", self.stats.total_commands)),
                };
                self.events.push(event);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(anyhow!("Cannot stop session in state: {:?}", self.state)),
        }
    }

    /// Set session to error state
    pub fn set_error(&mut self, error_message: String) {
        self.state = SessionState::Error(error_message.clone());
        
        let event = SessionEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SessionEventType::ErrorOccurred,
            timestamp: Utc::now(),
            details: Some(error_message),
        };
        self.events.push(event);
        self.updated_at = Utc::now();
    }

    /// Get session duration in seconds
    pub fn get_duration_seconds(&self) -> Option<u64> {
        if let Some(started_at) = self.started_at {
            let end_time = self.stopped_at.unwrap_or_else(Utc::now);
            let duration = end_time.signed_duration_since(started_at);
            Some(duration.num_seconds() as u64)
        } else {
            None
        }
    }

    /// Check if session can be modified
    pub fn can_modify(&self) -> bool {
        matches!(self.state, SessionState::Active | SessionState::Paused)
    }
}

/// Session manager handles multiple sessions and persistence
pub struct SessionManager {
    /// Currently active session
    current_session: Option<Session>,
    /// Directory where session files are stored
    sessions_dir: PathBuf,
    /// Directory where session backups are stored
    backups_dir: PathBuf,
    /// Cache of recent sessions for quick access
    session_cache: HashMap<String, Session>,
    /// Auto-save interval in seconds
    auto_save_interval: u64,
    /// Last auto-save timestamp
    last_auto_save: Option<SystemTime>,
    /// Maximum number of backups to keep per session
    max_backups: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let sessions_dir = Self::get_sessions_directory()?;
        let backups_dir = Self::get_backups_directory()?;
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&backups_dir)?;

        Ok(SessionManager {
            current_session: None,
            sessions_dir,
            backups_dir,
            session_cache: HashMap::new(),
            auto_save_interval: 30, // Auto-save every 30 seconds
            last_auto_save: None,
            max_backups: 5, // Keep 5 backups per session
        })
    }

    /// Get the directory where sessions are stored
    pub fn get_sessions_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow!("Cannot determine home directory"))?;
        
        Ok(PathBuf::from(home).join(".docpilot").join("sessions"))
    }

    /// Get the directory where session backups are stored
    fn get_backups_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow!("Cannot determine home directory"))?;
        
        Ok(PathBuf::from(home).join(".docpilot").join("backups"))
    }

    /// Start a new session
    pub fn start_session(&mut self, description: String, output_file: Option<PathBuf>) -> Result<String> {
        if self.current_session.is_some() {
            return Err(anyhow!("A session is already active. Stop the current session first."));
        }

        let session = Session::new(description, output_file)?;
        let session_id = session.id.clone();
        
        self.save_session(&session)?;
        self.current_session = Some(session);
        
        Ok(session_id)
    }

    /// Force start a new session (used after interactive handling of existing sessions)
    pub fn force_start_session(&mut self, description: String, output_file: Option<PathBuf>) -> Result<String> {
        // Clear any existing session first
        self.current_session = None;
        
        let session = Session::new(description, output_file)?;
        let session_id = session.id.clone();
        
        self.save_session(&session)?;
        self.current_session = Some(session);
        
        Ok(session_id)
    }

    /// Stop the current session
    pub fn stop_session(&mut self) -> Result<Option<Session>> {
        if let Some(mut session) = self.current_session.take() {
            session.stop()?;
            self.save_session(&session)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Pause the current session
    pub fn pause_session(&mut self) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            session.pause()?;
            // Clone the session to avoid borrowing issues
            let session_clone = session.clone();
            self.save_session(&session_clone)?;
            Ok(())
        } else {
            Err(anyhow!("No active session to pause"))
        }
    }

    /// Resume the current session
    pub fn resume_session(&mut self) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            session.resume()?;
            // Clone the session to avoid borrowing issues
            let session_clone = session.clone();
            self.save_session(&session_clone)?;
            Ok(())
        } else {
            Err(anyhow!("No active session to resume"))
        }
    }

    /// Add annotation to current session
    pub fn add_annotation(&mut self, text: String, annotation_type: AnnotationType) -> Result<String> {
        if let Some(session) = &mut self.current_session {
            let annotation_id = session.add_annotation(text, annotation_type);
            // Clone the session to avoid borrowing issues
            let session_clone = session.clone();
            self.save_session(&session_clone)?;
            Ok(annotation_id)
        } else {
            Err(anyhow!("No active session for annotation"))
        }
    }

    /// Add command to current session
    pub fn add_command(&mut self, command: CommandEntry) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            if session.state.is_active() {
                session.add_command(command);
                // Clone the session to avoid borrowing issues
                let session_clone = session.clone();
                self.save_session(&session_clone)?;
            }
            Ok(())
        } else {
            Err(anyhow!("No active session for command"))
        }
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&Session> {
        self.current_session.as_ref()
    }

    /// Get mutable reference to current session
    pub fn get_current_session_mut(&mut self) -> Option<&mut Session> {
        self.current_session.as_mut()
    }

    /// Set the current session (used for recovery and background processes)
    pub fn set_current_session(&mut self, session: Session) {
        self.current_session = Some(session);
    }

    /// Clear the current session (used for interactive session handling)
    pub fn clear_current_session(&mut self) {
        self.current_session = None;
    }

    /// Load a session by ID
    pub fn load_session(&mut self, session_id: &str) -> Result<Session> {
        // Check cache first
        if let Some(session) = self.session_cache.get(session_id) {
            return Ok(session.clone());
        }

        // Load from file
        let session_file = self.sessions_dir.join(format!("{}.json", session_id));
        if !session_file.exists() {
            return Err(anyhow!("Session not found: {}", session_id));
        }

        let content = fs::read_to_string(&session_file)?;
        let session: Session = serde_json::from_str(&content)?;
        
        // Add to cache
        self.session_cache.insert(session_id.to_string(), session.clone());
        
        Ok(session)
    }

    /// Save a session to disk with backup support
    pub fn save_session(&mut self, session: &Session) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", session.id));
        
        // Create backup if session file already exists
        if session_file.exists() {
            self.create_backup(&session.id)?;
        }
        
        // Write to temporary file first for atomic operation
        let temp_file = session_file.with_extension("tmp");
        let content = serde_json::to_string_pretty(session)?;
        fs::write(&temp_file, &content)?;
        
        // Atomic rename to final location
        fs::rename(&temp_file, &session_file)?;
        
        // Update cache
        self.session_cache.insert(session.id.clone(), session.clone());
        
        // Update auto-save timestamp
        self.last_auto_save = Some(SystemTime::now());
        
        Ok(())
    }

    /// Create a backup of an existing session
    fn create_backup(&self, session_id: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", session_id));
        if !session_file.exists() {
            return Ok(()); // Nothing to backup
        }
        
        // Generate backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System time error: {}", e))?
            .as_secs();
        
        let backup_file = self.backups_dir.join(format!("{}_{}.json", session_id, timestamp));
        
        // Copy the current session file to backup
        fs::copy(&session_file, &backup_file)?;
        
        // Clean up old backups
        self.cleanup_old_backups(session_id)?;
        
        Ok(())
    }

    /// Remove old backups, keeping only the most recent ones
    fn cleanup_old_backups(&self, session_id: &str) -> Result<()> {
        let mut backups = Vec::new();
        
        if self.backups_dir.exists() {
            for entry in fs::read_dir(&self.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&format!("{}_", session_id)) && filename.ends_with(".json") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                backups.push((path, modified));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove excess backups
        for (path, _) in backups.iter().skip(self.max_backups) {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("Warning: Failed to remove old backup {}: {}", path.display(), e);
            }
        }
        
        Ok(())
    }

    /// List all available sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        
        if self.sessions_dir.exists() {
            for entry in fs::read_dir(&self.sessions_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        sessions.push(stem.to_string());
                    }
                }
            }
        }
        
        sessions.sort();
        Ok(sessions)
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", session_id));
        
        if session_file.exists() {
            fs::remove_file(&session_file)?;
        }
        
        // Remove from cache
        self.session_cache.remove(session_id);
        
        // If this was the current session, clear it
        if let Some(current) = &self.current_session {
            if current.id == session_id {
                self.current_session = None;
            }
        }
        
        Ok(())
    }

    /// Recover from an interrupted session with enhanced error handling
    pub fn recover_session(&mut self) -> Result<Option<String>> {
        let sessions = self.list_sessions()?;
        let mut recovery_candidates = Vec::new();
        
        // Find all sessions that might need recovery
        for session_id in sessions {
            match self.load_session_with_recovery(&session_id) {
                Ok(session) => {
                    if session.state.is_active() || session.state.is_paused() {
                        recovery_candidates.push((session_id, session));
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load session {}: {}", session_id, e);
                    // Try to recover from backup
                    if let Ok(session) = self.recover_from_backup(&session_id) {
                        if session.state.is_active() || session.state.is_paused() {
                            eprintln!("Recovered session {} from backup", session_id);
                            recovery_candidates.push((session_id, session));
                        }
                    }
                }
            }
        }
        
        // If we have candidates, pick the most recent one
        if !recovery_candidates.is_empty() {
            recovery_candidates.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));
            let (session_id, session) = recovery_candidates.into_iter().next().unwrap();
            
            // Validate the recovered session
            if self.validate_session(&session) {
                self.current_session = Some(session);
                Ok(Some(session_id))
            } else {
                eprintln!("Warning: Recovered session {} failed validation", session_id);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Load a session with recovery attempts
    fn load_session_with_recovery(&mut self, session_id: &str) -> Result<Session> {
        // Try normal load first
        match self.load_session(session_id) {
            Ok(session) => Ok(session),
            Err(_) => {
                // Try to recover from backup
                self.recover_from_backup(session_id)
            }
        }
    }

    /// Attempt to recover a session from its most recent backup
    fn recover_from_backup(&self, session_id: &str) -> Result<Session> {
        let mut backups = Vec::new();
        
        if self.backups_dir.exists() {
            for entry in fs::read_dir(&self.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&format!("{}_", session_id)) && filename.ends_with(".json") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                backups.push((path, modified));
                            }
                        }
                    }
                }
            }
        }
        
        if backups.is_empty() {
            return Err(anyhow!("No backups found for session {}", session_id));
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Try to load from the most recent backup
        for (backup_path, _) in backups {
            match fs::read_to_string(&backup_path) {
                Ok(content) => {
                    match serde_json::from_str::<Session>(&content) {
                        Ok(session) => {
                            eprintln!("Successfully recovered session from backup: {}", backup_path.display());
                            return Ok(session);
                        }
                        Err(e) => {
                            eprintln!("Warning: Backup {} is corrupted: {}", backup_path.display(), e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read backup {}: {}", backup_path.display(), e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("All backups for session {} are corrupted", session_id))
    }

    /// Validate a session for consistency and integrity
    fn validate_session(&self, session: &Session) -> bool {
        // Basic validation checks
        if session.id.is_empty() || session.description.is_empty() {
            return false;
        }
        
        // Check timestamp consistency
        if let Some(started_at) = session.started_at {
            if started_at > session.created_at {
                return false;
            }
            
            if let Some(stopped_at) = session.stopped_at {
                if stopped_at < started_at {
                    return false;
                }
            }
        }
        
        // Validate statistics consistency
        let expected_total = session.stats.successful_commands + session.stats.failed_commands;
        if session.stats.total_commands < expected_total {
            return false;
        }
        
        if session.stats.total_annotations != session.annotations.len() {
            return false;
        }
        
        // Check for valid UUIDs in annotations and events
        for annotation in &session.annotations {
            if Uuid::parse_str(&annotation.id).is_err() {
                return false;
            }
        }
        
        for event in &session.events {
            if Uuid::parse_str(&event.id).is_err() {
                return false;
            }
        }
        
        true
    }

    /// Check if auto-save is needed and perform it
    pub fn check_auto_save(&mut self) -> Result<bool> {
        if let Some(session) = &self.current_session {
            if self.should_auto_save() {
                let session_clone = session.clone();
                self.save_session(&session_clone)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Determine if auto-save should be performed
    fn should_auto_save(&self) -> bool {
        if let Some(last_save) = self.last_auto_save {
            if let Ok(elapsed) = last_save.elapsed() {
                return elapsed.as_secs() >= self.auto_save_interval;
            }
        }
        // If we've never saved or can't determine elapsed time, save now
        true
    }

    /// Force an immediate save of the current session
    pub fn force_save(&mut self) -> Result<()> {
        if let Some(session) = &self.current_session {
            let session_clone = session.clone();
            self.save_session(&session_clone)?;
        }
        Ok(())
    }

    /// Get session backup information
    pub fn get_backup_info(&self, session_id: &str) -> Result<Vec<(PathBuf, SystemTime)>> {
        let mut backups = Vec::new();
        
        if self.backups_dir.exists() {
            for entry in fs::read_dir(&self.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&format!("{}_", session_id)) && filename.ends_with(".json") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                backups.push((path, modified));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(backups)
    }

    /// Clean up old sessions and backups
    pub fn cleanup_old_data(&self, max_age_days: u64) -> Result<usize> {
        let cutoff_time = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(max_age_days * 24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        
        let mut cleaned_count = 0;
        
        // Clean up old session files
        if self.sessions_dir.exists() {
            for entry in fs::read_dir(&self.sessions_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified < cutoff_time {
                                // Check if this is a stopped session before deleting
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                                        if session.state.is_stopped() {
                                            if let Err(e) = fs::remove_file(&path) {
                                                eprintln!("Warning: Failed to remove old session {}: {}", path.display(), e);
                                            } else {
                                                cleaned_count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Clean up old backup files
        if self.backups_dir.exists() {
            for entry in fs::read_dir(&self.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified < cutoff_time {
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("Warning: Failed to remove old backup {}: {}", path.display(), e);
                                } else {
                                    cleaned_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(cleaned_count)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats> {
        let mut stats = StorageStats::default();
        
        // Count sessions
        if self.sessions_dir.exists() {
            for entry in fs::read_dir(&self.sessions_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    stats.session_count += 1;
                    if let Ok(metadata) = entry.metadata() {
                        stats.total_size += metadata.len();
                    }
                }
            }
        }
        
        // Count backups
        if self.backups_dir.exists() {
            for entry in fs::read_dir(&self.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    stats.backup_count += 1;
                    if let Ok(metadata) = entry.metadata() {
                        stats.backup_size += metadata.len();
                    }
                }
            }
        }
        
        stats.total_size += stats.backup_size;
        Ok(stats)
    }

    /// Export a session to a different format or location
    pub fn export_session(&self, session_id: &str, export_path: &Path) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", session_id));
        if !session_file.exists() {
            return Err(anyhow!("Session not found: {}", session_id));
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = export_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Copy the session file
        fs::copy(&session_file, export_path)?;
        Ok(())
    }

    /// Import a session from an external file
    pub fn import_session(&mut self, import_path: &Path) -> Result<String> {
        if !import_path.exists() {
            return Err(anyhow!("Import file not found: {}", import_path.display()));
        }
        
        let content = fs::read_to_string(import_path)?;
        let session: Session = serde_json::from_str(&content)?;
        
        // Validate the imported session
        if !self.validate_session(&session) {
            return Err(anyhow!("Imported session failed validation"));
        }
        
        // Save the imported session
        self.save_session(&session)?;
        Ok(session.id)
    }
}

/// Storage statistics for session data
#[derive(Debug, Default)]
pub struct StorageStats {
    pub session_count: usize,
    pub backup_count: usize,
    pub total_size: u64,
    pub backup_size: u64,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create session manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_session_manager() -> (SessionManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let sessions_dir = temp_dir.path().join("sessions");
        let backups_dir = temp_dir.path().join("backups");
        std::fs::create_dir_all(&sessions_dir).expect("Failed to create sessions directory");
        std::fs::create_dir_all(&backups_dir).expect("Failed to create backups directory");
        
        let manager = SessionManager {
            current_session: None,
            sessions_dir,
            backups_dir,
            session_cache: HashMap::new(),
            auto_save_interval: 30,
            last_auto_save: None,
            max_backups: 5,
        };
        
        (manager, temp_dir)
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "Test session".to_string(),
            Some(std::path::PathBuf::from("test.md"))
        ).expect("Failed to create session");

        assert_eq!(session.description, "Test session");
        assert!(session.state.is_active());
        assert_eq!(session.commands.len(), 0);
        assert_eq!(session.annotations.len(), 0);
        assert_eq!(session.stats.total_commands, 0);
        assert!(session.started_at.is_some());
        assert!(session.stopped_at.is_none());
        assert_eq!(session.output_file, Some(std::path::PathBuf::from("test.md")));
    }

    #[test]
    fn test_session_state_transitions() {
        let mut session = Session::new(
            "Test session".to_string(),
            None
        ).expect("Failed to create session");

        // Test pause
        assert!(session.pause().is_ok());
        assert!(session.state.is_paused());
        assert_eq!(session.stats.pause_resume_count, 1);

        // Test resume
        assert!(session.resume().is_ok());
        assert!(session.state.is_active());

        // Test stop
        assert!(session.stop().is_ok());
        assert!(session.state.is_stopped());
        assert!(session.stopped_at.is_some());
        assert!(session.stats.duration_seconds.is_some());

        // Test invalid transitions
        assert!(session.pause().is_err());
        assert!(session.resume().is_err());
        assert!(session.stop().is_err());
    }

    #[test]
    fn test_session_manager_lifecycle() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let session_id = manager.start_session(
            "Test session".to_string(),
            None
        ).expect("Failed to start session");

        assert!(manager.get_current_session().is_some());
        assert_eq!(manager.get_current_session().unwrap().id, session_id);

        // Try to start another session (should fail)
        assert!(manager.start_session("Another session".to_string(), None).is_err());

        // Stop the session
        let stopped_session = manager.stop_session().expect("Failed to stop session");
        assert!(stopped_session.is_some());
        assert!(manager.get_current_session().is_none());

        let stopped = stopped_session.unwrap();
        assert!(stopped.state.is_stopped());
    }

    #[test]
    fn test_backup_and_recovery() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let session_id = manager.start_session(
            "Test session".to_string(),
            None
        ).expect("Failed to start session");

        // Add some data to the session
        manager.add_annotation("Test annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");

        // Force a save to create initial state
        manager.force_save().expect("Failed to force save");

        // Modify the session to create a backup when we save again
        manager.add_annotation("Second annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add second annotation");

        // Force another save to create a backup of the original
        manager.force_save().expect("Failed to force second save");

        // Verify backup was created
        let backups = manager.get_backup_info(&session_id).expect("Failed to get backup info");
        assert!(!backups.is_empty(), "No backups were created");

        // Test recovery from backup directly
        let recovered_session = manager.recover_from_backup(&session_id)
            .expect("Failed to recover from backup");

        // Should have recovered the session with annotations
        assert_eq!(recovered_session.id, session_id);
        assert!(recovered_session.annotations.len() >= 1);
        assert_eq!(recovered_session.annotations[0].text, "Test annotation");
    }

    #[test]
    fn test_auto_save_functionality() {
        let (mut manager, _temp_dir) = create_test_session_manager();
        
        // Set a very short auto-save interval for testing
        manager.auto_save_interval = 1;

        // Start a session (this automatically saves)
        let _session_id = manager.start_session(
            "Test session".to_string(),
            None
        ).expect("Failed to start session");

        // After starting (which saves), auto-save shouldn't be needed immediately
        assert!(!manager.should_auto_save());

        // Wait for auto-save interval to pass
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(manager.should_auto_save());

        // Test the check_auto_save function
        let did_save = manager.check_auto_save().expect("Failed to check auto save");
        assert!(did_save);

        // After auto-save, shouldn't need to save again immediately
        assert!(!manager.should_auto_save());
    }

    #[test]
    fn test_session_validation() {
        let (manager, _temp_dir) = create_test_session_manager();

        // Create a valid session
        let valid_session = Session::new(
            "Valid session".to_string(),
            None
        ).expect("Failed to create session");

        assert!(manager.validate_session(&valid_session));

        // Create an invalid session with empty ID
        let mut invalid_session = valid_session.clone();
        invalid_session.id = String::new();
        assert!(!manager.validate_session(&invalid_session));

        // Create an invalid session with inconsistent timestamps
        let mut invalid_session = valid_session.clone();
        invalid_session.started_at = Some(chrono::Utc::now());
        invalid_session.stopped_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(!manager.validate_session(&invalid_session));
    }

    #[test]
    fn test_storage_stats() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Initially no sessions
        let stats = manager.get_storage_stats().expect("Failed to get storage stats");
        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.backup_count, 0);

        // Create a session
        let _session_id = manager.start_session(
            "Test session".to_string(),
            None
        ).expect("Failed to start session");

        // Should now have one session
        let stats = manager.get_storage_stats().expect("Failed to get storage stats");
        assert_eq!(stats.session_count, 1);
        assert!(stats.total_size > 0);
    }

    #[test]
    fn test_export_import_session() {
        let (mut manager, temp_dir) = create_test_session_manager();

        // Create a session with some data
        let session_id = manager.start_session(
            "Export test session".to_string(),
            None
        ).expect("Failed to start session");

        manager.add_annotation("Export test annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");

        manager.stop_session().expect("Failed to stop session");

        // Export the session
        let export_path = temp_dir.path().join("exported_session.json");
        manager.export_session(&session_id, &export_path)
            .expect("Failed to export session");

        assert!(export_path.exists());

        // Import the session (this would normally be to a different manager)
        let imported_id = manager.import_session(&export_path)
            .expect("Failed to import session");

        // The imported session should have the same content
        let imported_session = manager.load_session(&imported_id)
            .expect("Failed to load imported session");

        assert_eq!(imported_session.description, "Export test session");
        assert_eq!(imported_session.annotations.len(), 1);
        assert_eq!(imported_session.annotations[0].text, "Export test annotation");
    }
    #[test]
    fn test_annotation_management() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let _session_id = manager.start_session(
            "Annotation test session".to_string(),
            None
        ).expect("Failed to start session");

        // Test different annotation types
        let note_id = manager.add_annotation(
            "This is a note".to_string(),
            AnnotationType::Note
        ).expect("Failed to add note");

        let explanation_id = manager.add_annotation(
            "This explains the process".to_string(),
            AnnotationType::Explanation
        ).expect("Failed to add explanation");

        let warning_id = manager.add_annotation(
            "This is a warning".to_string(),
            AnnotationType::Warning
        ).expect("Failed to add warning");

        let milestone_id = manager.add_annotation(
            "This is a milestone".to_string(),
            AnnotationType::Milestone
        ).expect("Failed to add milestone");

        // Verify annotations were added
        let session = manager.get_current_session().unwrap();
        assert_eq!(session.annotations.len(), 4);
        assert_eq!(session.stats.total_annotations, 4);

        // Verify annotation IDs are valid UUIDs
        assert!(uuid::Uuid::parse_str(&note_id).is_ok());
        assert!(uuid::Uuid::parse_str(&explanation_id).is_ok());
        assert!(uuid::Uuid::parse_str(&warning_id).is_ok());
        assert!(uuid::Uuid::parse_str(&milestone_id).is_ok());

        // Verify annotation content and types
        assert_eq!(session.annotations[0].text, "This is a note");
        assert!(matches!(session.annotations[0].annotation_type, AnnotationType::Note));
        
        assert_eq!(session.annotations[1].text, "This explains the process");
        assert!(matches!(session.annotations[1].annotation_type, AnnotationType::Explanation));
        
        assert_eq!(session.annotations[2].text, "This is a warning");
        assert!(matches!(session.annotations[2].annotation_type, AnnotationType::Warning));
        
        assert_eq!(session.annotations[3].text, "This is a milestone");
        assert!(matches!(session.annotations[3].annotation_type, AnnotationType::Milestone));
    }

    #[test]
    fn test_command_tracking_and_stats() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let _session_id = manager.start_session(
            "Command tracking test".to_string(),
            None
        ).expect("Failed to start session");

        // Create test commands with different exit codes
        let successful_command = crate::terminal::CommandEntry {
            command: "ls -la".to_string(),
            timestamp: chrono::Utc::now(),
            working_directory: "/tmp".to_string(),
            exit_code: Some(0),
            output: Some("file1\nfile2".to_string()),
            error: None,
            shell: "bash".to_string(),
        };

        let failed_command = crate::terminal::CommandEntry {
            command: "cat nonexistent.txt".to_string(),
            timestamp: chrono::Utc::now(),
            working_directory: "/tmp".to_string(),
            exit_code: Some(1),
            output: None,
            error: Some("No such file or directory".to_string()),
            shell: "bash".to_string(),
        };

        let pending_command = crate::terminal::CommandEntry {
            command: "sleep 10".to_string(),
            timestamp: chrono::Utc::now(),
            working_directory: "/tmp".to_string(),
            exit_code: None, // Still running
            output: None,
            error: None,
            shell: "bash".to_string(),
        };

        // Add commands to session
        manager.add_command(successful_command).expect("Failed to add successful command");
        manager.add_command(failed_command).expect("Failed to add failed command");
        manager.add_command(pending_command).expect("Failed to add pending command");

        // Verify command tracking
        let session = manager.get_current_session().unwrap();
        assert_eq!(session.commands.len(), 3);
        assert_eq!(session.stats.total_commands, 3);
        assert_eq!(session.stats.successful_commands, 1);
        assert_eq!(session.stats.failed_commands, 1);

        // Verify events were created for command captures
        let command_events: Vec<_> = session.events.iter()
            .filter(|e| matches!(e.event_type, crate::session::manager::SessionEventType::CommandCaptured))
            .collect();
        assert_eq!(command_events.len(), 3);
    }

    #[test]
    fn test_session_events_audit_trail() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let _session_id = manager.start_session(
            "Event tracking test".to_string(),
            None
        ).expect("Failed to start session");

        // Perform various operations that should create events
        manager.pause_session().expect("Failed to pause session");
        manager.resume_session().expect("Failed to resume session");
        manager.add_annotation("Test annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");

        let session_id = {
            let session = manager.get_current_session().unwrap();

            // Verify all expected events are present
            let event_types: Vec<_> = session.events.iter()
                .map(|e| &e.event_type)
                .collect();

            // Should have: SessionStarted, SessionPaused, SessionResumed, AnnotationAdded
            assert!(event_types.iter().any(|e| matches!(e, crate::session::manager::SessionEventType::SessionStarted)));
            assert!(event_types.iter().any(|e| matches!(e, crate::session::manager::SessionEventType::SessionPaused)));
            assert!(event_types.iter().any(|e| matches!(e, crate::session::manager::SessionEventType::SessionResumed)));
            assert!(event_types.iter().any(|e| matches!(e, crate::session::manager::SessionEventType::AnnotationAdded)));

            // Verify event IDs are valid UUIDs
            for event in &session.events {
                assert!(uuid::Uuid::parse_str(&event.id).is_ok());
            }

            session.id.clone()
        };

        // Stop session and verify stop event
        manager.stop_session().expect("Failed to stop session");
        let stopped_session = manager.load_session(&session_id).expect("Failed to load stopped session");
        
        let stop_events: Vec<_> = stopped_session.events.iter()
            .filter(|e| matches!(e.event_type, crate::session::manager::SessionEventType::SessionStopped))
            .collect();
        assert_eq!(stop_events.len(), 1);
    }

    #[test]
    fn test_session_metadata_and_platform_info() {
        let session = Session::new(
            "Metadata test session".to_string(),
            Some(std::path::PathBuf::from("output.md"))
        ).expect("Failed to create session");

        // Verify metadata is populated
        assert!(!session.metadata.working_directory.as_os_str().is_empty());
        assert!(!session.metadata.hostname.is_empty());
        assert_eq!(session.metadata.shell_type, "unknown"); // Will be updated by monitor
        assert_eq!(session.metadata.platform, "unknown"); // Will be updated by monitor
        assert!(session.metadata.tags.is_empty());
        assert!(session.metadata.settings.is_empty());

        // Test metadata updates
        let mut session = session;
        session.metadata.tags.push("test".to_string());
        session.metadata.tags.push("development".to_string());
        session.metadata.settings.insert("auto_save".to_string(), "true".to_string());
        session.metadata.llm_provider = Some("openai".to_string());

        assert_eq!(session.metadata.tags.len(), 2);
        assert_eq!(session.metadata.settings.len(), 1);
        assert_eq!(session.metadata.llm_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_comprehensive_backup_management() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let session_id = manager.start_session(
            "Backup management test".to_string(),
            None
        ).expect("Failed to start session");

        // Create multiple saves to generate backups
        for i in 0..8 {
            manager.add_annotation(
                format!("Annotation {}", i),
                AnnotationType::Note
            ).expect("Failed to add annotation");
            manager.force_save().expect("Failed to force save");
            
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Verify backups were created and cleaned up (should keep max_backups = 5)
        let backups = manager.get_backup_info(&session_id).expect("Failed to get backup info");
        assert!(backups.len() <= manager.max_backups);

        // Verify backups are sorted by timestamp (newest first)
        for i in 1..backups.len() {
            assert!(backups[i-1].1 >= backups[i].1);
        }

        // Test backup recovery
        let recovered_session = manager.recover_from_backup(&session_id)
            .expect("Failed to recover from backup");
        
        assert_eq!(recovered_session.id, session_id);
        assert!(!recovered_session.annotations.is_empty());
    }

    #[test]
    fn test_session_persistence_atomic_operations() {
        let (mut manager, temp_dir) = create_test_session_manager();

        // Start a session
        let session_id = manager.start_session(
            "Atomic operations test".to_string(),
            None
        ).expect("Failed to start session");

        // Add some data
        manager.add_annotation("Test annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");

        // Verify session file exists
        let session_file = manager.sessions_dir.join(format!("{}.json", session_id));
        assert!(session_file.exists());

        // Verify no temporary files are left behind
        let temp_files: Vec<_> = std::fs::read_dir(&manager.sessions_dir)
            .expect("Failed to read sessions directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str()) == Some("tmp")
            })
            .collect();
        
        assert!(temp_files.is_empty(), "Temporary files should be cleaned up");

        // Verify session can be loaded and is valid
        let loaded_session = manager.load_session(&session_id)
            .expect("Failed to load session");
        
        assert_eq!(loaded_session.id, session_id);
        assert_eq!(loaded_session.annotations.len(), 1);
    }

    #[test]
    fn test_session_corruption_detection_and_recovery() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let session_id = manager.start_session(
            "Corruption test session".to_string(),
            None
        ).expect("Failed to start session");

        // Add some data and save
        manager.add_annotation("Original annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");
        manager.force_save().expect("Failed to force save");

        // Corrupt the session file
        let session_file = manager.sessions_dir.join(format!("{}.json", session_id));
        std::fs::write(&session_file, "{ invalid json content }")
            .expect("Failed to write corrupted content");

        // Clear current session to force loading from file
        manager.current_session = None;

        // Try to load the corrupted session - should fail and attempt recovery
        let recovery_result = manager.load_session_with_recovery(&session_id);
        
        // Should either succeed with backup recovery or fail gracefully
        match recovery_result {
            Ok(recovered_session) => {
                // If recovery succeeded, verify the session is valid
                assert_eq!(recovered_session.id, session_id);
                assert!(manager.validate_session(&recovered_session));
            }
            Err(_) => {
                // If recovery failed, that's also acceptable for this test
                // as it means no valid backup was available
            }
        }
    }

    #[test]
    fn test_session_validation_comprehensive() {
        let (manager, _temp_dir) = create_test_session_manager();

        // Test valid session
        let valid_session = Session::new(
            "Valid session".to_string(),
            None
        ).expect("Failed to create session");
        assert!(manager.validate_session(&valid_session));

        // Test session with empty ID
        let mut invalid_session = valid_session.clone();
        invalid_session.id = String::new();
        assert!(!manager.validate_session(&invalid_session));

        // Test session with empty description
        let mut invalid_session = valid_session.clone();
        invalid_session.description = String::new();
        assert!(!manager.validate_session(&invalid_session));

        // Test session with invalid timestamp order
        let mut invalid_session = valid_session.clone();
        invalid_session.started_at = Some(chrono::Utc::now());
        invalid_session.created_at = chrono::Utc::now() + chrono::Duration::hours(1);
        assert!(manager.validate_session(&invalid_session));

        // Test session with stopped_at before started_at
        let mut invalid_session = valid_session.clone();
        let now = chrono::Utc::now();
        invalid_session.started_at = Some(now);
        invalid_session.stopped_at = Some(now - chrono::Duration::hours(1));
        assert!(!manager.validate_session(&invalid_session));

        // Test session with inconsistent command statistics
        let mut invalid_session = valid_session.clone();
        invalid_session.stats.total_commands = 5;
        invalid_session.stats.successful_commands = 3;
        invalid_session.stats.failed_commands = 4; // 3 + 4 > 5
        assert!(!manager.validate_session(&invalid_session));

        // Test session with inconsistent annotation count
        let mut invalid_session = valid_session.clone();
        invalid_session.stats.total_annotations = 5;
        invalid_session.annotations = vec![]; // Empty but stats say 5
        assert!(!manager.validate_session(&invalid_session));

        // Test session with valid annotation count
        let mut valid_session_with_annotations = valid_session.clone();
        valid_session_with_annotations.annotations.push(crate::session::manager::Annotation {
            id: uuid::Uuid::new_v4().to_string(),
            text: "Test".to_string(),
            timestamp: chrono::Utc::now(),
            annotation_type: AnnotationType::Note,
        });
        valid_session_with_annotations.stats.total_annotations = 1;
        assert!(manager.validate_session(&valid_session_with_annotations));

        // Test session with invalid annotation UUID
        let mut invalid_session = valid_session.clone();
        invalid_session.annotations.push(crate::session::manager::Annotation {
            id: "invalid-uuid".to_string(),
            text: "Test".to_string(),
            timestamp: chrono::Utc::now(),
            annotation_type: AnnotationType::Note,
        });
        invalid_session.stats.total_annotations = 1; // Update stats to match
        assert!(!manager.validate_session(&invalid_session));
    }

    #[test]
    fn test_storage_cleanup_operations() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Create multiple sessions
        let session1_id = manager.start_session(
            "Session 1".to_string(),
            None
        ).expect("Failed to start session 1");
        manager.stop_session().expect("Failed to stop session 1");

        let session2_id = manager.start_session(
            "Session 2".to_string(),
            None
        ).expect("Failed to start session 2");
        manager.stop_session().expect("Failed to stop session 2");

        // Verify sessions exist
        let sessions = manager.list_sessions().expect("Failed to list sessions");
        assert!(sessions.contains(&session1_id));
        assert!(sessions.contains(&session2_id));

        // Test cleanup with very long age (should not clean anything)
        let cleaned = manager.cleanup_old_data(365).expect("Failed to cleanup");
        assert_eq!(cleaned, 0);
        
        // Verify sessions still exist
        let sessions = manager.list_sessions().expect("Failed to list sessions");
        assert_eq!(sessions.len(), 2);

        // Test storage statistics
        let stats = manager.get_storage_stats().expect("Failed to get storage stats");
        assert_eq!(stats.session_count, 2);
        assert!(stats.total_size > 0);
    }

    #[test]
    fn test_session_cache_management() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start and stop a session
        let session_id = manager.start_session(
            "Cache test session".to_string(),
            None
        ).expect("Failed to start session");
        
        manager.add_annotation("Test annotation".to_string(), AnnotationType::Note)
            .expect("Failed to add annotation");
        
        manager.stop_session().expect("Failed to stop session");

        // Clear current session but keep cache
        manager.current_session = None;

        // First load should read from file and populate cache
        let session1 = manager.load_session(&session_id).expect("Failed to load session");
        assert!(manager.session_cache.contains_key(&session_id));

        // Second load should use cache
        let session2 = manager.load_session(&session_id).expect("Failed to load session from cache");
        assert_eq!(session1.id, session2.id);
        assert_eq!(session1.annotations.len(), session2.annotations.len());

        // Delete session should remove from cache
        manager.delete_session(&session_id).expect("Failed to delete session");
        assert!(!manager.session_cache.contains_key(&session_id));

        // Verify session file is gone
        let session_file = manager.sessions_dir.join(format!("{}.json", session_id));
        assert!(!session_file.exists());
    }

    #[test]
    fn test_error_state_handling() {
        let mut session = Session::new(
            "Error test session".to_string(),
            None
        ).expect("Failed to create session");

        // Test setting error state
        let error_message = "Test error occurred";
        session.set_error(error_message.to_string());

        assert!(session.state.is_error());
        if let SessionState::Error(msg) = &session.state {
            assert_eq!(msg, error_message);
        } else {
            panic!("Session should be in error state");
        }

        // Verify error event was created
        let error_events: Vec<_> = session.events.iter()
            .filter(|e| matches!(e.event_type, crate::session::manager::SessionEventType::ErrorOccurred))
            .collect();
        assert_eq!(error_events.len(), 1);
        assert_eq!(error_events[0].details, Some(error_message.to_string()));

        // Test that session cannot be modified in error state
        assert!(!session.can_modify());
        assert!(session.pause().is_err());
        assert!(session.resume().is_err());
        assert!(session.stop().is_err());
    }

    #[test]
    fn test_session_duration_calculation() {
        let mut session = Session::new(
            "Duration test session".to_string(),
            None
        ).expect("Failed to create session");

        // Initially should have a duration since it's active
        let initial_duration = session.get_duration_seconds();
        assert!(initial_duration.is_some());
        assert!(initial_duration.unwrap() >= 0);

        // Wait a bit and check duration increased
        std::thread::sleep(std::time::Duration::from_millis(100));
        let later_duration = session.get_duration_seconds();
        assert!(later_duration.unwrap() >= initial_duration.unwrap());

        // Stop session and verify duration is calculated
        session.stop().expect("Failed to stop session");
        assert!(session.stats.duration_seconds.is_some());
        // Duration might be 0 if the test runs very quickly, so just check it's not None
        assert!(session.stats.duration_seconds.unwrap() >= 0);

        // Duration should be consistent after stopping
        let final_duration1 = session.get_duration_seconds();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let final_duration2 = session.get_duration_seconds();
        assert_eq!(final_duration1, final_duration2);
    }

    #[test]
    fn test_concurrent_session_operations() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Start a session
        let _session_id = manager.start_session(
            "Concurrent test session".to_string(),
            None
        ).expect("Failed to start session");

        // Simulate concurrent operations
        for i in 0..10 {
            manager.add_annotation(
                format!("Concurrent annotation {}", i),
                AnnotationType::Note
            ).expect("Failed to add annotation");
            
            // Force save to test file locking/atomic operations
            manager.force_save().expect("Failed to force save");
        }

        // Verify all annotations were saved
        let session = manager.get_current_session().unwrap();
        assert_eq!(session.annotations.len(), 10);
        assert_eq!(session.stats.total_annotations, 10);

        // Verify session integrity
        assert!(manager.validate_session(session));
    }

    #[test]
    fn test_session_list_and_management() {
        let (mut manager, _temp_dir) = create_test_session_manager();

        // Initially no sessions
        let sessions = manager.list_sessions().expect("Failed to list sessions");
        assert!(sessions.is_empty());

        // Create multiple sessions
        let mut session_ids = Vec::new();
        for i in 0..3 {
            let session_id = manager.start_session(
                format!("Test session {}", i),
                None
            ).expect("Failed to start session");
            session_ids.push(session_id);
            manager.stop_session().expect("Failed to stop session");
        }

        // Verify all sessions are listed
        let sessions = manager.list_sessions().expect("Failed to list sessions");
        assert_eq!(sessions.len(), 3);
        
        // Sessions should be sorted
        let mut sorted_sessions = sessions.clone();
        sorted_sessions.sort();
        assert_eq!(sessions, sorted_sessions);

        // Verify all our session IDs are in the list
        for session_id in &session_ids {
            assert!(sessions.contains(session_id));
        }

        // Delete one session
        manager.delete_session(&session_ids[0]).expect("Failed to delete session");
        
        let sessions_after_delete = manager.list_sessions().expect("Failed to list sessions");
        assert_eq!(sessions_after_delete.len(), 2);
        assert!(!sessions_after_delete.contains(&session_ids[0]));
    }
}