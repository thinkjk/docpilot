use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

use super::platform::{Platform, PlatformUtils};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub working_directory: String,
    pub shell: String,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct TerminalMonitor {
    pub(crate) session_id: String,
    commands: Vec<CommandEntry>,
    monitoring: bool,
    pub(crate) shell_type: ShellType,
    pub(crate) platform: Platform,
}

#[derive(Debug, Clone)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Unknown(String),
}

impl ShellType {
    pub fn detect() -> Self {
        if let Ok(shell) = env::var("SHELL") {
            if shell.contains("bash") {
                ShellType::Bash
            } else if shell.contains("zsh") {
                ShellType::Zsh
            } else if shell.contains("fish") {
                ShellType::Fish
            } else {
                ShellType::Unknown(shell)
            }
        } else {
            ShellType::Unknown("unknown".to_string())
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::Unknown(name) => name,
        }
    }

    pub fn history_file(&self) -> Option<PathBuf> {
        let home = env::var("HOME").ok()?;
        let home_path = PathBuf::from(home);
        
        match self {
            ShellType::Bash => Some(home_path.join(".bash_history")),
            ShellType::Zsh => Some(home_path.join(".zsh_history")),
            ShellType::Fish => Some(home_path.join(".local/share/fish/fish_history")),
            ShellType::Unknown(_) => None,
        }
    }
}

impl TerminalMonitor {
    pub fn new(session_id: String) -> Result<Self> {
        let platform = Platform::detect();
        
        // Check if platform is supported
        if !PlatformUtils::is_supported_environment() {
            return Err(anyhow!("Unsupported platform: {}", platform.name()));
        }

        Ok(Self {
            session_id,
            commands: Vec::new(),
            monitoring: false,
            shell_type: ShellType::detect(),
            platform,
        })
    }

    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.monitoring {
            return Err(anyhow!("Monitoring is already active"));
        }

        // Initialize platform-specific monitoring
        PlatformUtils::initialize_monitoring()?;

        println!("Starting terminal monitoring for shell: {} on {}",
                 self.shell_type.name(),
                 self.platform.name());
        
        if let Some(session) = self.platform.detect_terminal_session()? {
            println!("Detected terminal session: {}", session);
        }

        self.monitoring = true;
        Ok(())
    }

    pub fn stop_monitoring(&mut self) -> Result<()> {
        if !self.monitoring {
            return Err(anyhow!("Monitoring is not active"));
        }

        self.monitoring = false;
        println!("Stopped terminal monitoring. Captured {} commands", self.commands.len());
        Ok(())
    }

    pub fn is_monitoring(&self) -> bool {
        self.monitoring
    }

    pub fn get_commands(&self) -> &[CommandEntry] {
        &self.commands
    }

    pub fn add_command(&mut self, command: CommandEntry) {
        self.commands.push(command);
    }

    /// Monitor shell history for new commands
    pub async fn monitor_history(&mut self) -> Result<()> {
        if !self.monitoring {
            return Ok(());
        }

        let history_file = self.shell_type.history_file()
            .ok_or_else(|| anyhow!("Cannot determine history file for shell: {}", self.shell_type.name()))?;

        if !history_file.exists() {
            return Err(anyhow!("History file does not exist: {:?}", history_file));
        }

        // Get initial file size to track new entries
        let initial_metadata = fs::metadata(&history_file)?;
        let mut last_size = initial_metadata.len();

        println!("Monitoring history file: {:?}", history_file);

        while self.monitoring {
            sleep(Duration::from_millis(500)).await;

            if let Ok(metadata) = fs::metadata(&history_file) {
                let current_size = metadata.len();
                
                if current_size > last_size {
                    // New content added to history file
                    if let Ok(content) = fs::read_to_string(&history_file) {
                        let new_commands = self.parse_new_commands(&content, last_size)?;
                        for cmd in new_commands {
                            self.add_command(cmd);
                            println!("Captured command: {}", self.commands.last().unwrap().command);
                        }
                    }
                    last_size = current_size;
                }
            }
        }

        Ok(())
    }

    /// Parse new commands from history file content
    fn parse_new_commands(&self, content: &str, from_size: u64) -> Result<Vec<CommandEntry>> {
        let mut commands = Vec::new();
        
        // Skip to the new content (this is a simplified approach)
        let lines: Vec<&str> = content.lines().collect();
        let estimated_lines_to_skip = (from_size / 50) as usize; // Rough estimate
        
        for line in lines.iter().skip(estimated_lines_to_skip) {
            if let Some(command) = self.parse_history_line(line) {
                commands.push(command);
            }
        }

        Ok(commands)
    }

    /// Parse a single history line based on shell type
    pub(crate) fn parse_history_line(&self, line: &str) -> Option<CommandEntry> {
        if line.trim().is_empty() {
            return None;
        }

        let command = match self.shell_type {
            ShellType::Zsh => {
                // Zsh history format: ": timestamp:duration;command"
                if line.starts_with(": ") {
                    line.split(';').nth(1)?.to_string()
                } else {
                    line.to_string()
                }
            }
            ShellType::Fish => {
                // Fish history format is more complex, simplified here
                if line.starts_with("- cmd: ") {
                    line.strip_prefix("- cmd: ")?.to_string()
                } else {
                    return None;
                }
            }
            _ => {
                // Bash and others: simple line format
                line.to_string()
            }
        };

        // Filter out common non-productive commands
        if self.should_ignore_command(&command) {
            return None;
        }

        Some(CommandEntry {
            command: command.trim().to_string(),
            timestamp: Utc::now(),
            exit_code: None, // We'll need to implement exit code tracking separately
            working_directory: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            shell: self.shell_type.name().to_string(),
            output: None,
            error: None,
        })
    }

    /// Determine if a command should be ignored
    pub(crate) fn should_ignore_command(&self, command: &str) -> bool {
        let ignore_patterns = [
            "ls", "pwd", "cd", "clear", "history", "exit",
            "echo", "cat", "less", "more", "head", "tail",
        ];

        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            ignore_patterns.contains(first_word)
        } else {
            true
        }
    }

    /// Execute a command and capture its output (for testing purposes)
    pub async fn execute_and_capture(&mut self, command: &str) -> Result<CommandEntry> {
        let start_time = Utc::now();
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let entry = CommandEntry {
            command: command.to_string(),
            timestamp: start_time,
            exit_code: output.status.code(),
            working_directory: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            shell: self.shell_type.name().to_string(),
            output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            error: if output.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        };

        self.add_command(entry.clone());
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_detection() {
        let shell = ShellType::detect();
        // Just ensure it doesn't panic
        assert!(!shell.name().is_empty());
    }

    #[test]
    fn test_monitor_creation() {
        // This test might fail on unsupported platforms, which is expected
        if let Ok(monitor) = TerminalMonitor::new("test-session".to_string()) {
            assert_eq!(monitor.session_id, "test-session");
            assert!(!monitor.is_monitoring());
            assert_eq!(monitor.get_commands().len(), 0);
        }
    }

    #[test]
    fn test_command_filtering() {
        // This test might fail on unsupported platforms, which is expected
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            assert!(monitor.should_ignore_command("ls"));
            assert!(monitor.should_ignore_command("cd /home"));
            assert!(!monitor.should_ignore_command("cargo build"));
            assert!(!monitor.should_ignore_command("git commit -m 'test'"));
        }
    }
}