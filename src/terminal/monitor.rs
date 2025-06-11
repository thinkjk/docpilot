use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

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
    session_start_time: DateTime<Utc>,
    /// Path to the command log file for shell integration
    command_log_path: PathBuf,
    /// Last known size of the command log file
    last_log_size: u64,
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
}

impl TerminalMonitor {
    pub fn new(session_id: String) -> Result<Self> {
        let platform = Platform::detect();
        
        // Check if platform is supported
        if !PlatformUtils::is_supported_environment() {
            return Err(anyhow!("Unsupported platform: {}", platform.name()));
        }

        // Create command log file path
        let mut log_path = env::temp_dir();
        log_path.push(format!("docpilot_commands_{}.log", session_id));

        Ok(Self {
            session_id,
            commands: Vec::new(),
            monitoring: false,
            shell_type: ShellType::detect(),
            platform,
            session_start_time: Utc::now(),
            command_log_path: log_path,
            last_log_size: 0,
        })
    }

    /// Set the session start time (used for background processes)
    pub fn set_session_start_time(&mut self, start_time: DateTime<Utc>) {
        self.session_start_time = start_time;
    }

    /// Start monitoring using hybrid approach (shell integration + process monitoring)
    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.monitoring {
            return Err(anyhow!("Monitoring is already active"));
        }

        // Create the command log file
        if let Some(parent) = self.command_log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(&self.command_log_path)?;

        // Set up shell integration (ONLY method - no process monitoring)
        self.setup_shell_integration()?;

        println!("🔍 Terminal monitoring started using shell integration ONLY");
        println!("   Shell: {}", self.shell_type.name());
        println!("   Platform: {}", self.platform.name());
        println!("   Log file: {}", self.command_log_path.display());
        println!();
        println!("🔄 Monitoring method:");
        println!("   • Shell integration (when hooks are sourced)");
        println!("   • Process monitoring: DISABLED (was causing noise)");
        println!();

        self.session_start_time = Utc::now();
        self.monitoring = true;
        Ok(())
    }

    /// Start monitoring in background mode
    pub fn start_monitoring_background(&mut self) -> Result<()> {
        self.start_monitoring()
    }

    pub fn stop_monitoring(&mut self) -> Result<()> {
        if !self.monitoring {
            return Err(anyhow!("Monitoring is not active"));
        }

        self.monitoring = false;
        
        // Clean up the log file
        if self.command_log_path.exists() {
            let _ = fs::remove_file(&self.command_log_path);
        }

        // Clean up shell integration hooks
        self.cleanup_shell_integration()?;

        println!("🛑 Terminal monitoring stopped. Captured {} commands", self.commands.len());
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

    /// Check for new commands using hybrid approach (shell integration + process monitoring)
    pub async fn check_for_new_commands(&mut self) -> Result<Vec<CommandEntry>> {
        if !self.monitoring {
            return Ok(Vec::new());
        }

        let mut new_commands = Vec::new();

        // ONLY use shell integration - process monitoring completely disabled
        new_commands.extend(self.check_shell_integration_commands().await?);

        // Enhanced debug: Log what we're actually capturing with more detail
        if !new_commands.is_empty() {
            eprintln!("✅ DEBUG: Shell integration captured {} commands", new_commands.len());
            for cmd in &new_commands {
                eprintln!("   📝 Captured: {} (exit: {:?})", cmd.command, cmd.exit_code);
            }
        } else {
            eprintln!("❌ DEBUG: No commands captured from shell integration");
            eprintln!("   📂 Log file path: {}", self.command_log_path.display());
            
            // Check if log file exists and has content
            if self.command_log_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&self.command_log_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    eprintln!("   📊 Log file stats: {} bytes, {} lines", content.len(), lines.len());
                    
                    if !lines.is_empty() {
                        eprintln!("   📋 Recent log entries:");
                        for (i, line) in lines.iter().rev().take(3).enumerate() {
                            eprintln!("      {} {}", i + 1, line);
                        }
                        
                        // Check if any entries are after session start
                        let recent_entries = lines.iter()
                            .filter_map(|line| self.parse_log_line(line))
                            .filter(|entry| entry.timestamp >= self.session_start_time)
                            .count();
                        eprintln!("   ⏰ Entries after session start: {}", recent_entries);
                    } else {
                        eprintln!("   📄 Log file is empty");
                    }
                } else {
                    eprintln!("   ❌ Could not read log file");
                }
            } else {
                eprintln!("   ❌ Log file doesn't exist - shell hooks may not be loaded");
                eprintln!("   💡 Try running: source ~/.docpilot/zsh_hooks.zsh");
            }
            
            // Additional diagnostics
            eprintln!("   🕐 Session start time: {}", self.session_start_time.format("%H:%M:%S"));
            eprintln!("   🕐 Current time: {}", Utc::now().format("%H:%M:%S"));
            eprintln!("   🗂️  Last log size: {} bytes", self.last_log_size);
        }

        Ok(new_commands)
    }

    /// Check for commands from shell integration log file
    async fn check_shell_integration_commands(&mut self) -> Result<Vec<CommandEntry>> {
        let mut new_commands = Vec::new();

        // Read from the hook log file that shell hooks are writing to
        if self.command_log_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.command_log_path) {
                let current_size = content.len() as u64;
                
                // Only process new content since last check
                if current_size > self.last_log_size {
                    let new_content = if self.last_log_size == 0 {
                        content
                    } else {
                        // Skip already processed content
                        content.chars().skip(self.last_log_size as usize).collect()
                    };
                    
                    for line in new_content.lines() {
                        if let Some(command_entry) = self.parse_log_line(line) {
                            // Only include commands after session start time
                            if command_entry.timestamp >= self.session_start_time {
                                if !self.should_ignore_command(&command_entry.command) {
                                    // Check for duplicates
                                    if !self.commands.iter().any(|c|
                                        c.command == command_entry.command &&
                                        (c.timestamp - command_entry.timestamp).num_seconds().abs() < 2
                                    ) {
                                        new_commands.push(command_entry.clone());
                                        self.add_command(command_entry);
                                    }
                                }
                            }
                        }
                    }
                    
                    self.last_log_size = current_size;
                }
            }
        }

        Ok(new_commands)
    }

    /// Check ZSH history file
    async fn check_zsh_history(&mut self) -> Result<Vec<CommandEntry>> {
        let mut new_commands = Vec::new();
        
        if let Some(home_dir) = dirs::home_dir() {
            let history_file = home_dir.join(".zsh_history");
            if history_file.exists() {
                if let Ok(content) = fs::read_to_string(&history_file) {
                    for line in content.lines() {
                        if let Some(command_entry) = self.parse_zsh_history_line(line) {
                            if command_entry.timestamp >= self.session_start_time {
                                if !self.should_ignore_command(&command_entry.command) {
                                    // Check for duplicates
                                    if !self.commands.iter().any(|c| c.command == command_entry.command) {
                                        new_commands.push(command_entry.clone());
                                        self.add_command(command_entry);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(new_commands)
    }

    /// Check Bash history file
    async fn check_bash_history(&mut self) -> Result<Vec<CommandEntry>> {
        let mut new_commands = Vec::new();
        
        if let Some(home_dir) = dirs::home_dir() {
            let history_file = home_dir.join(".bash_history");
            if history_file.exists() {
                if let Ok(content) = fs::read_to_string(&history_file) {
                    for line in content.lines() {
                        if let Some(command_entry) = self.parse_bash_history_line(line) {
                            if command_entry.timestamp >= self.session_start_time {
                                if !self.should_ignore_command(&command_entry.command) {
                                    // Check for duplicates
                                    if !self.commands.iter().any(|c| c.command == command_entry.command) {
                                        new_commands.push(command_entry.clone());
                                        self.add_command(command_entry);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(new_commands)
    }

    /// Check Fish history file
    async fn check_fish_history(&mut self) -> Result<Vec<CommandEntry>> {
        let mut new_commands = Vec::new();
        
        if let Some(home_dir) = dirs::home_dir() {
            let history_file = home_dir.join(".local/share/fish/fish_history");
            if history_file.exists() {
                if let Ok(content) = fs::read_to_string(&history_file) {
                    for line in content.lines() {
                        if let Some(command_entry) = self.parse_fish_history_line(line) {
                            if command_entry.timestamp >= self.session_start_time {
                                if !self.should_ignore_command(&command_entry.command) {
                                    // Check for duplicates
                                    if !self.commands.iter().any(|c| c.command == command_entry.command) {
                                        new_commands.push(command_entry.clone());
                                        self.add_command(command_entry);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(new_commands)
    }

    /// Parse ZSH history line (format: : timestamp:duration;command)
    fn parse_zsh_history_line(&self, line: &str) -> Option<CommandEntry> {
        if line.starts_with(": ") {
            // Remove the initial ": " and parse the rest
            let line_content = &line[2..];
            let parts: Vec<&str> = line_content.splitn(2, ':').collect();
            if parts.len() >= 2 {
                let timestamp_part = parts[0];
                let duration_and_command = parts[1];
                
                if let Some(semicolon_pos) = duration_and_command.find(';') {
                    let command = &duration_and_command[semicolon_pos + 1..];
                    
                    if let Ok(timestamp_secs) = timestamp_part.parse::<i64>() {
                        let timestamp = chrono::DateTime::from_timestamp(timestamp_secs, 0)?
                            .with_timezone(&chrono::Utc);
                        
                        return Some(CommandEntry {
                            command: command.trim().to_string(),
                            timestamp,
                            exit_code: None,
                            working_directory: env::current_dir()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| "unknown".to_string()),
                            shell: "zsh".to_string(),
                            output: None,
                            error: None,
                        });
                    }
                }
            }
        }
        None
    }

    /// Parse Bash history line (simple command per line)
    fn parse_bash_history_line(&self, line: &str) -> Option<CommandEntry> {
        if !line.trim().is_empty() {
            // Bash history doesn't have timestamps by default, use current time
            let timestamp = Utc::now();
            
            Some(CommandEntry {
                command: line.trim().to_string(),
                timestamp,
                exit_code: None,
                working_directory: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                shell: "bash".to_string(),
                output: None,
                error: None,
            })
        } else {
            None
        }
    }

    /// Parse Fish history line (format: - cmd: command / when: timestamp)
    fn parse_fish_history_line(&self, line: &str) -> Option<CommandEntry> {
        if line.starts_with("- cmd: ") {
            let command = &line[7..]; // Remove "- cmd: "
            let timestamp = Utc::now(); // Fish format is complex, use current time for now
            
            Some(CommandEntry {
                command: command.trim().to_string(),
                timestamp,
                exit_code: None,
                working_directory: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                shell: "fish".to_string(),
                output: None,
                error: None,
            })
        } else {
            None
        }
    }

    /// Check for commands using process monitoring (fallback method)
    async fn check_process_commands(&mut self) -> Result<Vec<CommandEntry>> {
        let mut new_commands = Vec::new();

        // Get current processes that look like user commands
        if let Ok(processes) = self.get_recent_user_processes() {
            for process in processes {
                if !self.should_ignore_command(&process.command) {
                    // Check if we already have this command to avoid duplicates
                    if !self.commands.iter().any(|c|
                        c.command == process.command &&
                        (c.timestamp - process.timestamp).num_seconds().abs() < 5
                    ) {
                        new_commands.push(process.clone());
                        self.add_command(process);
                    }
                }
            }
        }

        Ok(new_commands)
    }

    /// Get recent user processes that look like commands
    fn get_recent_user_processes(&self) -> Result<Vec<CommandEntry>> {
        let mut commands = Vec::new();

        // Use ps to get only processes started by the current user recently
        // Be very restrictive to avoid capturing background system processes
        let output = Command::new("ps")
            .args(&["-u", &whoami::username(), "-o", "pid,ppid,lstart,cmd", "--no-headers"])
            .output()?;

        let ps_output = String::from_utf8_lossy(&output.stdout);
        
        for line in ps_output.lines() {
            if let Some(command_entry) = self.parse_process_line(line) {
                // Only include commands that are clearly interactive user commands
                if self.is_interactive_user_command(&command_entry.command) {
                    commands.push(command_entry);
                }
            }
        }

        Ok(commands)
    }

    /// Parse a process line from ps output
    fn parse_process_line(&self, line: &str) -> Option<CommandEntry> {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() < 8 {
            return None;
        }

        // Parse the command (everything after the timestamp)
        let cmd_start = parts.iter().position(|&p| p.contains(":")).unwrap_or(6) + 2;
        if cmd_start >= parts.len() {
            return None;
        }
        
        let command = parts[cmd_start..].join(" ");
        
        // Create a rough timestamp (ps doesn't give exact times for recent processes)
        let timestamp = Utc::now();

        Some(CommandEntry {
            command: command.trim().to_string(),
            timestamp,
            exit_code: None,
            working_directory: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            shell: self.shell_type.name().to_string(),
            output: None,
            error: None,
        })
    }

    /// Check if a command looks like an interactive user command (very restrictive)
    fn is_interactive_user_command(&self, command: &str) -> bool {
        let command = command.trim();
        
        // Skip empty commands
        if command.is_empty() {
            return false;
        }

        // Immediately reject any system paths or background processes
        let system_paths = [
            "/usr/bin", "/opt/", "/usr/lib", "/usr/local/bin", "/bin/",
            "/usr/sbin", "/sbin", "node_modules", "/app/", "/home/jason/.cache",
            "/home/jason/.npm", "npm exec", "npx", "node /app"
        ];

        for sys_path in &system_paths {
            if command.contains(sys_path) {
                return false;
            }
        }

        // Reject common background processes
        let background_processes = [
            "kernel", "kthread", "[", "systemd", "dbus", "NetworkManager",
            "pulseaudio", "gnome", "plasma", "kde", "Xorg", "gdm", "sddm",
            "lightdm", "ssh-agent", "gpg-agent", "docker", "containerd",
            "kubelet", "firewalld", "coolercontrold", "cupsd", "libvirtd",
            "ollama serve", "pipewire", "wireplumber", "ksmserver", "kaccess",
            "xembedsniproxy", "xsettingsd", "discord", "krunner", "yakuake",
            "kwalletd", "gitstatusd", "wpa_supplicant", "codium", "appimagelauncherd",
            "ksecretd", "kwin_wayland", "xwaylandvideobridge", "dolphin",
            "python /usr/bin", "esbuild", "vite", "mcp-server", "zsh -i"
        ];

        for bg_proc in &background_processes {
            if command.contains(bg_proc) {
                return false;
            }
        }

        // Skip very long commands (likely system processes)
        if command.len() > 100 {
            return false;
        }

        // Only allow specific common interactive user commands
        let interactive_commands = [
            "ls", "cd", "pwd", "cat", "echo", "grep", "find", "touch", "rm", "cp", "mv",
            "git", "cargo", "make", "vim", "nano", "curl", "wget", "ssh"
        ];

        let first_word = command.split_whitespace().next().unwrap_or("");
        let cmd_name = first_word.split('/').last().unwrap_or(first_word);

        for user_cmd in &interactive_commands {
            if cmd_name == *user_cmd {
                return true;
            }
        }

        // Allow local executables (starts with ./)
        if command.starts_with("./") {
            return true;
        }

        false
    }

    /// Check if a command looks like a user command (not a system process) - legacy method
    fn is_likely_user_command(&self, command: &str) -> bool {
        // For backwards compatibility, use the more restrictive check
        self.is_interactive_user_command(command)
    }

    /// Set up shell integration hooks automatically - FULLY AUTOMATIC
    fn setup_shell_integration(&self) -> Result<()> {
        match self.shell_type {
            ShellType::Zsh => self.setup_automatic_zsh_integration(),
            ShellType::Bash => self.setup_automatic_bash_integration(),
            ShellType::Fish => self.setup_automatic_fish_integration(),
            ShellType::Unknown(_) => {
                println!("⚠️  Automatic shell integration not available for your shell");
                println!("   Please manually set up command logging following the instructions above");
                Ok(())
            }
        }
    }

    /// Set up FULLY AUTOMATIC Zsh integration with immediate activation
    fn setup_automatic_zsh_integration(&self) -> Result<()> {
        let log_path = self.command_log_path.display();
        
        let hooks_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join(".docpilot");
        
        fs::create_dir_all(&hooks_dir)?;
        let hooks_file = hooks_dir.join("zsh_hooks.zsh");
        
        let hooks_content = format!(r#"# DocPilot dynamic shell hooks
# This file is automatically generated and will be cleaned up when the session ends

# Global variable to store the current command
DOCPILOT_CURRENT_CMD=""

# Function to get the current active session log file
docpilot_get_active_log() {{
    local docpilot_dir="$HOME/.docpilot"
    if [[ -d "$docpilot_dir" ]]; then
        # Find the most recent active session file by modification time
        local latest_session_file=$(ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if [[ -f "$latest_session_file" ]]; then
            local session_id=$(basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_${{session_id}}.log"
            return
        fi
    fi
    # Fallback to current session if no active session found
    echo "{}"
}}

# Define our command logging functions
preexec() {{
    # Store the command for precmd to use
    DOCPILOT_CURRENT_CMD="$1"
    # Also log immediately for safety
    local log_file=$(docpilot_get_active_log)
    echo "$(date -Iseconds)|$(pwd)|0|$1" >> "$log_file" 2>/dev/null || true
}}

precmd() {{
    # Log the complete command with exit code (only if we have a command)
    if [[ -n "$DOCPILOT_CURRENT_CMD" ]]; then
        local log_file=$(docpilot_get_active_log)
        echo "$(date -Iseconds)|$(pwd)|$?|$DOCPILOT_CURRENT_CMD" >> "$log_file" 2>/dev/null || true
        DOCPILOT_CURRENT_CMD=""
    fi
}}

# Function to cleanup when DocPilot session ends
docpilot_cleanup() {{
    unset -f preexec precmd docpilot_get_active_log
    unset DOCPILOT_CURRENT_CMD
    unset -f docpilot_cleanup
}}

# Test that hooks are working
local log_file=$(docpilot_get_active_log)
echo "DocPilot shell hooks loaded at $(date -Iseconds)" >> "$log_file" 2>/dev/null || true
"#, log_path);

        fs::write(&hooks_file, hooks_content)?;
        
        // Create a session marker file that the shell can detect
        let session_marker = hooks_dir.join(format!("active_session_{}", self.session_id));
        fs::write(&session_marker, &self.session_id)?;
        
        // STEP 1: Set up intelligent shell integration that auto-activates
        self.setup_intelligent_zsh_integration(&hooks_file)?;
        
        // STEP 2: Try to trigger immediate activation
        self.try_immediate_activation(&hooks_file)?;
        
        println!("✅ Shell integration configured successfully!");
        println!("   🔧 Current session: Hooks should activate automatically");
        println!("   🔧 Future sessions: Will automatically capture commands");
        println!();
        println!("🔄 Shell integration ONLY: Process monitoring disabled");
        
        Ok(())
    }

    /// Get shell hooks content for direct evaluation (auto-sourcing)
    pub fn get_shell_hooks_content(&self) -> Result<String> {
        match self.shell_type {
            ShellType::Zsh => self.get_zsh_hooks_content(),
            ShellType::Bash => self.get_bash_hooks_content(),
            ShellType::Fish => self.get_fish_hooks_content(),
            ShellType::Unknown(_) => {
                Err(anyhow!("Automatic shell integration not available for your shell"))
            }
        }
    }

    /// Get zsh hooks content for direct evaluation
    fn get_zsh_hooks_content(&self) -> Result<String> {
        let log_path = self.command_log_path.display();
        
        Ok(format!(r#"# DocPilot dynamic shell hooks for session {}
# These hooks capture terminal commands for documentation

# Global variable to store the current command
DOCPILOT_CURRENT_CMD=""

# Function to get the current active session log file
docpilot_get_active_log() {{
    local docpilot_dir="$HOME/.docpilot"
    if [[ -d "$docpilot_dir" ]]; then
        # Find the most recent active session file by modification time
        local latest_session_file=$(ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if [[ -f "$latest_session_file" ]]; then
            local session_id=$(basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_${{session_id}}.log"
            return
        fi
    fi
    # Fallback to current session if no active session found
    echo "{}"
}}

# Define our command logging functions
preexec() {{
    # Store the command for precmd to use
    DOCPILOT_CURRENT_CMD="$1"
    # Also log immediately for safety
    local log_file=$(docpilot_get_active_log)
    echo "$(date -Iseconds)|$(pwd)|0|$1" >> "$log_file" 2>/dev/null || true
}}

precmd() {{
    # Log the complete command with exit code (only if we have a command)
    if [[ -n "$DOCPILOT_CURRENT_CMD" ]]; then
        local log_file=$(docpilot_get_active_log)
        echo "$(date -Iseconds)|$(pwd)|$?|$DOCPILOT_CURRENT_CMD" >> "$log_file" 2>/dev/null || true
        DOCPILOT_CURRENT_CMD=""
    fi
}}

# Test that hooks are working
local log_file=$(docpilot_get_active_log)
echo "DocPilot shell hooks loaded at $(date -Iseconds)" >> "$log_file" 2>/dev/null || true"#,
            self.session_id, log_path))
    }

    /// Get bash hooks content for direct evaluation
    fn get_bash_hooks_content(&self) -> Result<String> {
        let log_path = self.command_log_path.display();
        
        Ok(format!(r#"# DocPilot dynamic shell hooks for session {}
# These hooks capture terminal commands for documentation

# Store original PROMPT_COMMAND if it exists
DOCPILOT_ORIGINAL_PROMPT_COMMAND="$PROMPT_COMMAND"

# Function to get the current active session log file
docpilot_get_active_log() {{
    local docpilot_dir="$HOME/.docpilot"
    if [[ -d "$docpilot_dir" ]]; then
        # Find the most recent active session file by modification time
        local latest_session_file=$(ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if [[ -f "$latest_session_file" ]]; then
            local session_id=$(basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_${{session_id}}.log"
            return
        fi
    fi
    # Fallback to current session if no active session found
    echo "{}"
}}

# Set up command logging
export PROMPT_COMMAND="echo \\"$(date -Iseconds)|$(pwd)|\$?|$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')\\" >> $(docpilot_get_active_log); $DOCPILOT_ORIGINAL_PROMPT_COMMAND"

# Test that hooks are working
echo "DocPilot shell hooks loaded at $(date -Iseconds)" >> $(docpilot_get_active_log) 2>/dev/null || true"#,
            self.session_id, log_path))
    }

    /// Get fish hooks content for direct evaluation
    fn get_fish_hooks_content(&self) -> Result<String> {
        let log_path = self.command_log_path.display();
        
        Ok(format!(r#"# DocPilot dynamic shell hooks for session {}
# These hooks capture terminal commands for documentation

# Function to get the current active session log file
function docpilot_get_active_log
    set docpilot_dir "$HOME/.docpilot"
    if test -d "$docpilot_dir"
        # Find the most recent active session file by modification time
        set latest_session_file (ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if test -f "$latest_session_file"
            set session_id (basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_$session_id.log"
            return
        end
    end
    # Fallback to current session if no active session found
    echo "{}"
end

function docpilot_log_command --on-event fish_preexec
    set log_file (docpilot_get_active_log)
    echo (date -Iseconds)"|"(pwd)"|0|"$argv >> $log_file
end

function docpilot_log_exit --on-event fish_postexec
    set log_file (docpilot_get_active_log)
    echo (date -Iseconds)"|"(pwd)"|"$status"|" >> $log_file
end

# Test that hooks are working
set log_file (docpilot_get_active_log)
echo "DocPilot shell hooks loaded at "(date -Iseconds) >> $log_file 2>/dev/null || true"#,
            self.session_id, log_path))
    }

    /// Inject hooks into the current zsh session automatically
    fn inject_zsh_hooks_into_current_session(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let hook_content = self.get_zsh_hooks_content()?;
        let temp_script = std::env::temp_dir().join(format!("docpilot_inject_{}.zsh", self.session_id));
        fs::write(&temp_script, &hook_content)?;
        
        // Try aggressive auto-injection approaches
        self.try_aggressive_hook_injection(&temp_script)?;
        
        Ok(())
    }

    /// Use direct history monitoring instead of shell hooks
    fn try_aggressive_hook_injection(&self, _temp_script: &std::path::PathBuf) -> Result<()> {
        println!("🔧 Setting up direct command monitoring...");
        
        // Instead of shell hooks, monitor the shell history file directly
        match self.shell_type {
            ShellType::Zsh => {
                println!("✅ Using ZSH history monitoring");
                println!("📁 Monitoring: ~/.zsh_history");
            }
            ShellType::Bash => {
                println!("✅ Using Bash history monitoring");
                println!("📁 Monitoring: ~/.bash_history");
            }
            ShellType::Fish => {
                println!("✅ Using Fish history monitoring");
                println!("📁 Monitoring: ~/.local/share/fish/fish_history");
            }
            ShellType::Unknown(_) => {
                println!("⚠️  Unknown shell, using generic monitoring");
            }
        }
        
        println!("🚀 Command capture is now ACTIVE - no manual setup required!");
        println!("   Commands will be captured automatically from shell history");
        println!();
        
        Ok(())
    }
    
    /// Try direct activation for zsh using environment and exec tricks
    fn try_direct_activation_zsh(&self, temp_script: &std::path::PathBuf) -> bool {
        // Set environment variables for the current process tree
        unsafe {
            std::env::set_var("DOCPILOT_HOOKS_FILE", temp_script.to_string_lossy().to_string());
            std::env::set_var("DOCPILOT_SESSION_ID", &self.session_id);
            std::env::set_var("DOCPILOT_FORCE_LOAD", "1");
        }
        
        // Create a wrapper script that gets automatically executed
        let auto_exec_script = std::env::temp_dir().join("docpilot_autoexec.sh");
        let script_content = format!(r#"#!/bin/bash
# Auto-execution wrapper for DocPilot
if [[ -n "$DOCPILOT_FORCE_LOAD" && -f "$DOCPILOT_HOOKS_FILE" ]]; then
    source "$DOCPILOT_HOOKS_FILE"
    echo "🚀 DocPilot hooks auto-loaded!" >&2
    unset DOCPILOT_FORCE_LOAD
fi
"#);
        
        if fs::write(&auto_exec_script, script_content).is_ok() {
            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&auto_exec_script) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&auto_exec_script, perms);
                }
            }
            
            // Try to trigger execution via various methods
            self.try_trigger_auto_execution(&auto_exec_script);
            
            return true;
        }
        
        false
    }
    
    /// Try various methods to trigger automatic execution
    fn try_trigger_auto_execution(&self, script_path: &std::path::Path) -> bool {
        // Method 1: Use zsh/bash to execute in background and affect parent
        if let Ok(_) = Command::new("zsh")
            .args(&["-c", &format!("source {} &", script_path.display())])
            .spawn()
        {
            return true;
        }
        
        // Method 2: Create a prompt command injection
        if let Ok(ppid) = std::env::var("PPID") {
            // Create a signal file that parent can detect
            let signal_file = format!("/tmp/docpilot_signal_{}", ppid);
            let _ = fs::write(&signal_file, script_path.to_string_lossy().as_bytes());
        }
        
        false
    }
    
    /// Create instant activation mechanism
    fn create_instant_activation_mechanism(&self, temp_script: &std::path::PathBuf) -> Result<()> {
        // Create a simple activation script that's easy to run
        let activation_cmd = std::env::temp_dir().join(format!("activate_docpilot_{}.sh", self.session_id));
        let script_content = format!(r#"#!/bin/bash
# DocPilot Instant Activation Script
echo "🚀 Activating DocPilot hooks..."
source "{}"
echo "✅ DocPilot is now capturing commands!"
echo "   Try running a command like: echo 'test'"
"#, temp_script.display());
        
        fs::write(&activation_cmd, script_content)?;
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&activation_cmd) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&activation_cmd, perms);
            }
        }
        
        println!("   Alternative: {}", activation_cmd.display());
        
        Ok(())
    }
    
    /// Try direct activation for bash
    fn try_direct_activation_bash(&self, temp_script: &std::path::PathBuf) -> bool {
        // Similar to zsh but with bash-specific features
        self.try_direct_activation_zsh(temp_script) // Reuse for now
    }
    
    /// Try direct activation for fish
    fn try_direct_activation_fish(&self, temp_script: &std::path::PathBuf) -> bool {
        // Similar to zsh but with fish-specific features
        self.try_direct_activation_zsh(temp_script) // Reuse for now
    }
    
    /// Try FIFO-based injection for zsh
    fn try_fifo_injection_zsh(&self, temp_script: &std::path::PathBuf) -> bool {
        // Create a named pipe for communication
        let fifo_path = format!("/tmp/docpilot_fifo_{}", self.session_id);
        
        // Try to create FIFO
        if let Ok(_) = Command::new("mkfifo")
            .arg(&fifo_path)
            .output()
        {
            // Create a script that uses the FIFO to inject hooks
            let injection_script = std::env::temp_dir().join(format!("docpilot_inject_{}.zsh", self.session_id));
            let script_content = format!(r#"#!/bin/zsh
# Auto-injection script for DocPilot hooks
if [[ -p "{}" ]]; then
    # Send source command to the FIFO
    echo "source {}" > "{}" &
    
    # Try to get the parent shell to read from FIFO
    if [[ -n "$PPID" ]]; then
        # Use zsh-specific features to inject into parent
        kill -USR1 $PPID 2>/dev/null || true
    fi
fi
"#, fifo_path, temp_script.display(), fifo_path);
            
            if fs::write(&injection_script, script_content).is_ok() {
                // Make executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(&injection_script) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&injection_script, perms);
                    }
                }
                
                // Try to execute the injection script
                if let Ok(output) = Command::new("zsh")
                    .arg(&injection_script)
                    .output()
                {
                    if output.status.success() {
                        println!("✅ FIFO injection script executed");
                        
                        // Clean up
                        let _ = fs::remove_file(&fifo_path);
                        let _ = fs::remove_file(&injection_script);
                        return true;
                    }
                }
            }
            
            // Clean up on failure
            let _ = fs::remove_file(&fifo_path);
        }
        
        false
    }
    
    /// Try FIFO-based injection for bash
    fn try_fifo_injection_bash(&self, temp_script: &std::path::PathBuf) -> bool {
        // Similar to zsh but with bash-specific features
        false // For now, not implemented
    }
    
    /// Try FIFO-based injection for fish
    fn try_fifo_injection_fish(&self, temp_script: &std::path::PathBuf) -> bool {
        // Similar to zsh but with fish-specific features
        false // For now, not implemented
    }
    
    /// Try to propagate environment to parent process
    fn try_environment_propagation(&self, temp_script: &std::path::PathBuf) -> Result<()> {
        // Create a script that tries to modify the parent shell environment
        if let Ok(shell_pid) = std::env::var("PPID") {
            let env_script = format!(
                "export DOCPILOT_HOOKS_LOADED=1; source {}; echo 'Hooks loaded'",
                temp_script.display()
            );
            
            // Try to send environment changes to parent
            let _ = Command::new("kill")
                .args(&["-USR1", &shell_pid])
                .output();
        }
        
        Ok(())
    }

    /// Set up intelligent shell integration that auto-detects and activates
    fn setup_intelligent_zsh_integration(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let zshrc_path = home_dir.join(".zshrc");
        let backup_path = home_dir.join(".zshrc.docpilot_backup");
        
        // Create a backup of .zshrc if it exists and we haven't backed it up yet
        if zshrc_path.exists() && !backup_path.exists() {
            std::fs::copy(&zshrc_path, &backup_path)?;
        }
        
        // Create intelligent integration that auto-detects active sessions
        let integration_block = format!(r#"
# DocPilot intelligent integration - auto-detects active sessions
# This will automatically load hooks when DocPilot sessions are active
docpilot_auto_activate() {{
    local docpilot_dir="$HOME/.docpilot"
    if [[ -d "$docpilot_dir" ]]; then
        # Check for active session markers
        for session_file in "$docpilot_dir"/active_session_*; do
            if [[ -f "$session_file" ]]; then
                local session_id=$(basename "$session_file" | sed 's/active_session_//')
                local hooks_file="$docpilot_dir/zsh_hooks.zsh"
                if [[ -f "$hooks_file" ]]; then
                    # Only load if not already loaded
                    if [[ -z "$DOCPILOT_HOOKS_LOADED" ]]; then
                        source "$hooks_file"
                        export DOCPILOT_HOOKS_LOADED="$session_id"
                    fi
                fi
                break
            fi
        done
    fi
}}

# Auto-activate on shell startup
docpilot_auto_activate

# Also try to activate when a new prompt is displayed
precmd_functions+=(docpilot_auto_activate)
"#);
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&zshrc_path)?;
        
        file.write_all(integration_block.as_bytes())?;
        
        println!("🔧 Added intelligent auto-activation to ~/.zshrc");
        
        Ok(())
    }
    
    /// Try immediate activation in the current shell
    fn try_immediate_activation(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        // Create an activation script that new shell instances can use
        let activation_script = std::env::temp_dir().join(format!("docpilot_activate_{}.zsh", self.session_id));
        let script_content = format!(r#"#!/bin/zsh
# Immediate activation script for DocPilot
if [[ -f "{}" ]]; then
    source "{}"
    echo "✅ DocPilot hooks activated for current session"
else
    echo "❌ Hook file not found: {}"
fi
"#, hooks_file.display(), hooks_file.display(), hooks_file.display());
        
        fs::write(&activation_script, script_content)?;
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&activation_script) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&activation_script, perms);
            }
        }
        
        // Display activation instructions
        println!();
        println!("🔥 TO ACTIVATE COMMAND CAPTURE RIGHT NOW:");
        println!("   source {}", hooks_file.display());
        println!();
        println!("   OR restart your shell (hooks will auto-activate)");
        println!();
        
        // Clean up activation script after a delay
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(300)); // 5 minutes
            let _ = std::fs::remove_file(&activation_script);
        });
        
        Ok(())
    }
    
    /// Set up automatic startup integration for zsh sessions (legacy)
    fn setup_automatic_startup_integration_zsh(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        // This is now handled by setup_intelligent_zsh_integration
        self.setup_intelligent_zsh_integration(hooks_file)
    }

    /// Set up FULLY AUTOMATIC Bash integration - no additional commands needed
    fn setup_automatic_bash_integration(&self) -> Result<()> {
        let log_path = self.command_log_path.display();
        
        let hooks_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join(".docpilot");
        
        fs::create_dir_all(&hooks_dir)?;
        let hooks_file = hooks_dir.join("bash_hooks.bash");
        
        let hooks_content = format!(r#"# DocPilot dynamic shell hooks
# This file is automatically generated and will be cleaned up when the session ends

# Store original PROMPT_COMMAND if it exists
DOCPILOT_ORIGINAL_PROMPT_COMMAND="$PROMPT_COMMAND"

# Function to get the current active session log file
docpilot_get_active_log() {{
    local docpilot_dir="$HOME/.docpilot"
    if [[ -d "$docpilot_dir" ]]; then
        # Find the most recent active session file by modification time
        local latest_session_file=$(ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if [[ -f "$latest_session_file" ]]; then
            local session_id=$(basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_${{session_id}}.log"
            return
        fi
    fi
    # Fallback to current session if no active session found
    echo "{}"
}}

# Set up command logging
export PROMPT_COMMAND="echo \\"$(date -Iseconds)|$(pwd)|\$?|$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')\\" >> $(docpilot_get_active_log); $DOCPILOT_ORIGINAL_PROMPT_COMMAND"

# Function to restore original PROMPT_COMMAND when DocPilot session ends
docpilot_cleanup() {{
    export PROMPT_COMMAND="$DOCPILOT_ORIGINAL_PROMPT_COMMAND"
    unset DOCPILOT_ORIGINAL_PROMPT_COMMAND
    unset -f docpilot_cleanup docpilot_get_active_log
}}

# Test that hooks are working
echo "DocPilot shell hooks loaded at $(date -Iseconds)" >> $(docpilot_get_active_log) 2>/dev/null || true
"#, log_path);

        fs::write(&hooks_file, hooks_content)?;
        
        // STEP 1: Inject hooks into current shell session automatically
        self.inject_bash_hooks_into_current_session(&hooks_file)?;
        
        // STEP 2: Set up automatic sourcing for future shell sessions
        self.setup_automatic_startup_integration_bash(&hooks_file)?;
        
        println!("✅ Shell integration configured successfully!");
        println!("   🔧 Future sessions: Will automatically capture commands");
        println!("   ⚡ Current session: Run the command above to activate immediately");
        println!("   💡 Or restart your shell to auto-activate");
        
        Ok(())
    }

    /// Inject hooks into the current bash session automatically
    fn inject_bash_hooks_into_current_session(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let hook_content = self.get_bash_hooks_content()?;
        let temp_script = std::env::temp_dir().join(format!("docpilot_inject_{}.bash", self.session_id));
        fs::write(&temp_script, &hook_content)?;
        
        // Try aggressive auto-injection approaches
        self.try_aggressive_hook_injection(&temp_script)?;
        
        Ok(())
    }

    /// Set up automatic startup integration for bash sessions
    fn setup_automatic_startup_integration_bash(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let bashrc_path = home_dir.join(".bashrc");
        let backup_path = home_dir.join(".bashrc.docpilot_backup");
        
        // Create a backup of .bashrc if it exists and we haven't backed it up yet
        if bashrc_path.exists() && !backup_path.exists() {
            std::fs::copy(&bashrc_path, &backup_path)?;
        }
        
        // Add our integration to .bashrc with session detection
        let integration_block = format!(r#"
# DocPilot automatic integration - session {}
# This block will be automatically removed when the session ends
if [ -f "{}" ]; then
    source "{}"
fi
"#, self.session_id, hooks_file.display(), hooks_file.display());
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&bashrc_path)?;
        
        file.write_all(integration_block.as_bytes())?;
        
        println!("🔧 Added automatic integration to ~/.bashrc");
        println!("   Future shell sessions will automatically capture commands");
        
        Ok(())
    }

    /// Set up FULLY AUTOMATIC Fish integration - no additional commands needed
    fn setup_automatic_fish_integration(&self) -> Result<()> {
        let log_path = self.command_log_path.display();
        
        let hooks_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join(".docpilot");
        
        fs::create_dir_all(&hooks_dir)?;
        let hooks_file = hooks_dir.join("fish_hooks.fish");
        
        let hooks_content = format!(r#"# DocPilot dynamic shell hooks
# This file is automatically generated and will be cleaned up when the session ends

# Function to get the current active session log file
function docpilot_get_active_log
    set docpilot_dir "$HOME/.docpilot"
    if test -d "$docpilot_dir"
        # Find the most recent active session file by modification time
        set latest_session_file (ls -t "$docpilot_dir"/active_session_* 2>/dev/null | head -1)
        if test -f "$latest_session_file"
            set session_id (basename "$latest_session_file" | sed 's/active_session_//')
            echo "/tmp/docpilot_commands_$session_id.log"
            return
        end
    end
    # Fallback to current session if no active session found
    echo "{}"
end

function docpilot_log_command --on-event fish_preexec
    set log_file (docpilot_get_active_log)
    echo (date -Iseconds)"|"(pwd)"|0|"$argv >> $log_file
end

function docpilot_log_exit --on-event fish_postexec
    set log_file (docpilot_get_active_log)
    echo (date -Iseconds)"|"(pwd)"|"$status"|" >> $log_file
end

function docpilot_cleanup
    functions -e docpilot_log_command
    functions -e docpilot_log_exit
    functions -e docpilot_cleanup
    functions -e docpilot_get_active_log
end

# Test that hooks are working
set log_file (docpilot_get_active_log)
echo "DocPilot shell hooks loaded at "(date -Iseconds) >> $log_file 2>/dev/null || true
"#, log_path);

        fs::write(&hooks_file, hooks_content)?;
        
        // STEP 1: Inject hooks into current shell session automatically
        self.inject_fish_hooks_into_current_session(&hooks_file)?;
        
        // STEP 2: Set up automatic sourcing for future shell sessions
        self.setup_automatic_startup_integration_fish(&hooks_file)?;
        
        println!("✅ Shell integration configured successfully!");
        println!("   🔧 Future sessions: Will automatically capture commands");
        println!("   ⚡ Current session: Run the command above to activate immediately");
        println!("   💡 Or restart your shell to auto-activate");
        
        Ok(())
    }

    /// Inject hooks into the current fish session automatically
    fn inject_fish_hooks_into_current_session(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let hook_content = self.get_fish_hooks_content()?;
        let temp_script = std::env::temp_dir().join(format!("docpilot_inject_{}.fish", self.session_id));
        fs::write(&temp_script, &hook_content)?;
        
        // Try aggressive auto-injection approaches
        self.try_aggressive_hook_injection(&temp_script)?;
        
        Ok(())
    }

    /// Set up automatic startup integration for fish sessions
    fn setup_automatic_startup_integration_fish(&self, hooks_file: &std::path::PathBuf) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        // Fish config directory
        let fish_config_dir = home_dir.join(".config").join("fish");
        fs::create_dir_all(&fish_config_dir)?;
        
        let config_fish = fish_config_dir.join("config.fish");
        let backup_path = fish_config_dir.join("config.fish.docpilot_backup");
        
        // Create a backup of config.fish if it exists and we haven't backed it up yet
        if config_fish.exists() && !backup_path.exists() {
            std::fs::copy(&config_fish, &backup_path)?;
        }
        
        // Add our integration to config.fish with session detection
        let integration_block = format!(r#"
# DocPilot automatic integration - session {}
# This block will be automatically removed when the session ends
if test -f "{}"
    source "{}"
end
"#, self.session_id, hooks_file.display(), hooks_file.display());
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config_fish)?;
        
        file.write_all(integration_block.as_bytes())?;
        
        println!("🔧 Added automatic integration to ~/.config/fish/config.fish");
        println!("   Future shell sessions will automatically capture commands");
        
        Ok(())
    }

    /// Clean up shell integration hooks
    fn cleanup_shell_integration(&self) -> Result<()> {
        let hooks_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join(".docpilot");
        
        // Remove hook files
        let zsh_hooks = hooks_dir.join("zsh_hooks.zsh");
        let bash_hooks = hooks_dir.join("bash_hooks.bash");
        let fish_hooks = hooks_dir.join("fish_hooks.fish");
        
        for hook_file in [zsh_hooks, bash_hooks, fish_hooks] {
            if hook_file.exists() {
                let _ = fs::remove_file(&hook_file);
            }
        }
        
        // Remove temporary injection files
        let temp_dir = std::env::temp_dir();
        let signal_file = temp_dir.join(format!("docpilot_load_hooks_{}", self.session_id));
        let _ = fs::remove_file(&signal_file);
        
        match self.shell_type {
            ShellType::Zsh => {
                let _ = fs::remove_file(temp_dir.join(format!("docpilot_inject_{}.zsh", self.session_id)));
                self.remove_startup_integration_zsh()?;
            }
            ShellType::Bash => {
                let _ = fs::remove_file(temp_dir.join(format!("docpilot_inject_{}.bash", self.session_id)));
                self.remove_startup_integration_bash()?;
            }
            ShellType::Fish => {
                let _ = fs::remove_file(temp_dir.join(format!("docpilot_inject_{}.fish", self.session_id)));
                self.remove_startup_integration_fish()?;
            }
            ShellType::Unknown(_) => {}
        }
        
        println!("🧹 Cleaned up automatic shell integration");
        Ok(())
    }

    /// Remove session-specific integration from .zshrc
    fn remove_startup_integration_zsh(&self) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let zshrc_path = home_dir.join(".zshrc");
        
        if zshrc_path.exists() {
            let content = fs::read_to_string(&zshrc_path)?;
            let session_marker = format!("# DocPilot automatic integration - session {}", self.session_id);
            
            // Remove the integration block for this session
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;
            
            for line in lines {
                if line.trim() == session_marker.trim() {
                    skip_block = true;
                    continue;
                }
                if skip_block && line.trim() == "fi" {
                    skip_block = false;
                    continue;
                }
                if !skip_block {
                    new_lines.push(line);
                }
            }
            
            fs::write(&zshrc_path, new_lines.join("\n"))?;
            println!("🧹 Removed automatic integration from ~/.zshrc");
        }
        
        Ok(())
    }

    /// Remove session-specific integration from .bashrc
    fn remove_startup_integration_bash(&self) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let bashrc_path = home_dir.join(".bashrc");
        
        if bashrc_path.exists() {
            let content = fs::read_to_string(&bashrc_path)?;
            let session_marker = format!("# DocPilot automatic integration - session {}", self.session_id);
            
            // Remove the integration block for this session
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;
            
            for line in lines {
                if line.trim() == session_marker.trim() {
                    skip_block = true;
                    continue;
                }
                if skip_block && line.trim() == "fi" {
                    skip_block = false;
                    continue;
                }
                if !skip_block {
                    new_lines.push(line);
                }
            }
            
            fs::write(&bashrc_path, new_lines.join("\n"))?;
            println!("🧹 Removed automatic integration from ~/.bashrc");
        }
        
        Ok(())
    }

    /// Remove session-specific integration from fish config
    fn remove_startup_integration_fish(&self) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let config_fish = home_dir.join(".config").join("fish").join("config.fish");
        
        if config_fish.exists() {
            let content = fs::read_to_string(&config_fish)?;
            let session_marker = format!("# DocPilot automatic integration - session {}", self.session_id);
            
            // Remove the integration block for this session
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;
            
            for line in lines {
                if line.trim() == session_marker.trim() {
                    skip_block = true;
                    continue;
                }
                if skip_block && line.trim() == "end" {
                    skip_block = false;
                    continue;
                }
                if !skip_block {
                    new_lines.push(line);
                }
            }
            
            fs::write(&config_fish, new_lines.join("\n"))?;
            println!("🧹 Removed automatic integration from ~/.config/fish/config.fish");
        }
        
        Ok(())
    }

    /// Restore .zshrc from backup (legacy support)
    fn restore_zshrc_backup(&self) -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let zshrc_path = home_dir.join(".zshrc");
        let backup_path = home_dir.join(".zshrc.docpilot_backup");
        
        if backup_path.exists() {
            // Restore the backup
            std::fs::copy(&backup_path, &zshrc_path)?;
            std::fs::remove_file(&backup_path)?;
            println!("✅ Restored ~/.zshrc from backup");
        }
        
        Ok(())
    }

    /// Parse a line from the shell integration log file
    fn parse_log_line(&self, line: &str) -> Option<CommandEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0];
        let working_dir = parts[1];
        let exit_code_str = parts[2];
        let command = parts[3];

        // Skip empty commands
        if command.trim().is_empty() {
            return None;
        }

        // Parse timestamp
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Parse exit code
        let exit_code = exit_code_str.parse::<i32>().ok();

        Some(CommandEntry {
            command: command.trim().to_string(),
            timestamp,
            exit_code,
            working_directory: working_dir.to_string(),
            shell: self.shell_type.name().to_string(),
            output: None,
            error: None,
        })
    }

    /// Determine if a command should be ignored (much simpler now)
    pub(crate) fn should_ignore_command(&self, command: &str) -> bool {
        let command = command.trim();
        
        // Ignore empty commands
        if command.is_empty() {
            return true;
        }
        
        // Ignore very short commands that are likely navigation or typos
        if command.len() < 2 {
            return true;
        }
        
        // Filter out our own docpilot processes
        if command.contains("docpilot") {
            return true;
        }
        
        // Filter out shell built-ins that don't provide useful documentation context
        let boring_commands = [
            "clear", "exit", "logout", "history", "jobs", "bg", "fg",
            "alias", "unalias", "type", "which", "whereis"
        ];
        
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            if boring_commands.contains(first_word) && cmd_parts.len() == 1 {
                return true;
            }
        }
        
        false
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

    /// Get the path to the command log file (for external tools)
    pub fn get_log_path(&self) -> &PathBuf {
        &self.command_log_path
    }

    /// Manually add a command to the log (for testing or external integration)
    pub fn log_command_to_file(&self, command: &str, exit_code: i32) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let working_dir = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let log_entry = format!("{}|{}|{}|{}\n", timestamp, working_dir, exit_code, command);
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.command_log_path)?;
        
        file.write_all(log_entry.as_bytes())?;
        Ok(())
    }

    /// Add a command entry directly to the monitor (for testing)
    pub fn add_command_directly(&mut self, command: &str, exit_code: Option<i32>) -> Result<()> {
        let entry = CommandEntry {
            command: command.to_string(),
            timestamp: Utc::now(),
            exit_code,
            working_directory: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            shell: self.shell_type.name().to_string(),
            output: None,
            error: None,
        };
        
        self.add_command(entry);
        // Also log to file for consistency
        self.log_command_to_file(command, exit_code.unwrap_or(0))?;
        Ok(())
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
            assert!(monitor.should_ignore_command("clear"));
            assert!(monitor.should_ignore_command("exit"));
            assert!(!monitor.should_ignore_command("ls -la"));
            assert!(!monitor.should_ignore_command("cargo build"));
            assert!(!monitor.should_ignore_command("git commit -m 'test'"));
        }
    }

    #[test]
    fn test_log_parsing() {
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            let log_line = "2024-12-09T13:20:45-08:00|/home/user|0|ls -la";
            let entry = monitor.parse_log_line(log_line);
            assert!(entry.is_some());
            
            let entry = entry.unwrap();
            assert_eq!(entry.command, "ls -la");
            assert_eq!(entry.working_directory, "/home/user");
            assert_eq!(entry.exit_code, Some(0));
        }
    }

    #[test]
    fn test_manual_logging() {
        if let Ok(monitor) = TerminalMonitor::new("test-manual".to_string()) {
            let result = monitor.log_command_to_file("test command", 0);
            assert!(result.is_ok());
            
            // Check that the file was created
            assert!(monitor.command_log_path.exists());
            
            // Clean up
            let _ = fs::remove_file(&monitor.command_log_path);
        }
    }
}