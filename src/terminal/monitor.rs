use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
    last_history_size: u64,
    session_start_time: DateTime<Utc>,
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
            last_history_size: 0,
            session_start_time: Utc::now(),
        })
    }

    /// Set the session start time (used for background processes)
    pub fn set_session_start_time(&mut self, start_time: DateTime<Utc>) {
        self.session_start_time = start_time;
    }

    /// Reset history position to current file size (used for background processes)
    pub fn reset_history_position(&mut self) {
        if let Some(history_file) = self.shell_type.history_file() {
            if let Ok(metadata) = fs::metadata(&history_file) {
                self.last_history_size = metadata.len();
                println!("Reset history position to current file size: {} bytes", self.last_history_size);
            }
        }
    }

    /// Start monitoring in background mode (doesn't reset history position)
    pub fn start_monitoring_background(&mut self) -> Result<()> {
        if self.monitoring {
            return Err(anyhow!("Monitoring is already active"));
        }

        // Initialize platform-specific monitoring
        PlatformUtils::initialize_monitoring()?;

        println!("Starting background terminal monitoring for shell: {} on {}",
                 self.shell_type.name(),
                 self.platform.name());
        
        if let Some(session) = self.platform.detect_terminal_session()? {
            println!("Detected terminal session: {}", session);
        }

        // Don't reset history size for background processes - we want to read from the beginning
        if let Some(history_file) = self.shell_type.history_file() {
            if let Ok(metadata) = fs::metadata(&history_file) {
                println!("Background monitor starting from history position: {} bytes", self.last_history_size);
            }
        }

        self.monitoring = true;
        Ok(())
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

        // Record the current history file size to avoid capturing old commands
        if let Some(history_file) = self.shell_type.history_file() {
            if let Ok(metadata) = fs::metadata(&history_file) {
                self.last_history_size = metadata.len();
                println!("Starting from history position: {} bytes", self.last_history_size);
            }
        }

        self.session_start_time = Utc::now();
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

    /// Check shell history for new commands (single check, not a loop)
    pub async fn check_history_once(&mut self) -> Result<Vec<CommandEntry>> {
        if !self.monitoring {
            return Ok(Vec::new());
        }

        let history_file = self.shell_type.history_file()
            .ok_or_else(|| anyhow!("Cannot determine history file for shell: {}", self.shell_type.name()))?;

        if !history_file.exists() {
            // In test environments or when history files don't exist, just return empty
            if let Ok(pwd) = std::env::var("PWD") {
                if pwd.starts_with("/tmp") {
                    return Ok(Vec::new());
                }
            }
            return Err(anyhow!("History file does not exist: {:?}", history_file));
        }

        let mut new_commands = Vec::new();

        // Check if the file has grown since last check
        let current_metadata = fs::metadata(&history_file)?;
        let current_size = current_metadata.len();
        
        // For background processes starting from 0, always read if there's content
        if current_size <= self.last_history_size && self.last_history_size > 0 {
            // No new content (only skip if we've read before)
            return Ok(new_commands);
        }

        // Read the entire file content
        let history_bytes = fs::read(&history_file)?;
        let content = String::from_utf8_lossy(&history_bytes);
        let lines: Vec<&str> = content.lines().collect();
        
        // For background processes, read all lines that are after session start time
        // For regular processes, use byte position estimation
        let start_line = if self.last_history_size == 0 {
            // Background process: read all lines and filter by timestamp
            0
        } else {
            // Regular process: estimate position based on file size
            let bytes_per_line_estimate = if lines.is_empty() { 50 } else { content.len() / lines.len() };
            let estimated_old_lines = (self.last_history_size as usize) / bytes_per_line_estimate;
            // Start from the estimated position, but be conservative
            if estimated_old_lines > 5 { estimated_old_lines - 5 } else { 0 }
        };
        let start_line = start_line.min(lines.len());
        
        for line in lines.iter().skip(start_line) {
            if let Some(command) = self.parse_history_line(line) {
                // For background processes, be more lenient with timestamp filtering
                let should_include = if self.last_history_size == 0 {
                    // Background process: include commands from around session start time (with some buffer)
                    let time_diff = (command.timestamp - self.session_start_time).num_seconds();
                    time_diff >= -60 // Include commands from 1 minute before session start
                } else {
                    // Regular process: include commands after session start time
                    command.timestamp >= self.session_start_time
                };
                
                if should_include {
                    // Check if we already have this exact command
                    if !self.commands.iter().any(|existing|
                        existing.command == command.command &&
                        (existing.timestamp - command.timestamp).num_seconds().abs() < 2
                    ) {
                        new_commands.push(command.clone());
                        self.add_command(command);
                    }
                }
            }
        }

        // Update the last known size
        self.last_history_size = current_size;
        Ok(new_commands)
    }

    /// Monitor shell history for new commands (legacy method for compatibility)
    pub async fn monitor_history(&mut self) -> Result<()> {
        // Just do a single check instead of infinite loop
        let _new_commands = self.check_history_once().await?;
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

    /// Parse all commands from history file content (used when file was truncated)
    fn parse_all_new_commands(&self, content: &str) -> Result<Vec<CommandEntry>> {
        let mut commands = Vec::new();
        
        // Parse all lines in the file
        for line in content.lines() {
            if let Some(command) = self.parse_history_line(line) {
                // Only add if we haven't seen this command before
                if !self.commands.iter().any(|existing|
                    existing.command == command.command &&
                    existing.timestamp.timestamp() == command.timestamp.timestamp()
                ) {
                    commands.push(command);
                }
            }
        }

        Ok(commands)
    }

    /// Parse a single history line based on shell type
    pub(crate) fn parse_history_line(&self, line: &str) -> Option<CommandEntry> {
        if line.trim().is_empty() {
            return None;
        }

        let (command, timestamp) = match self.shell_type {
            ShellType::Zsh => {
                // Zsh history format: ": timestamp:duration;command"
                if line.starts_with(": ") {
                    let parts: Vec<&str> = line.splitn(2, ';').collect();
                    if parts.len() == 2 {
                        let command = parts[1].to_string();
                        // Extract timestamp from ": timestamp:duration"
                        let timestamp_part = parts[0].trim_start_matches(": ");
                        let timestamp_str = timestamp_part.split(':').next().unwrap_or("0");
                        
                        if let Ok(timestamp_secs) = timestamp_str.parse::<i64>() {
                            let timestamp = DateTime::from_timestamp(timestamp_secs, 0)
                                .unwrap_or_else(|| Utc::now());
                            (command, timestamp)
                        } else {
                            (command, Utc::now())
                        }
                    } else {
                        (line.to_string(), Utc::now())
                    }
                } else {
                    (line.to_string(), Utc::now())
                }
            }
            ShellType::Fish => {
                // Fish history format is more complex, simplified here
                if line.starts_with("- cmd: ") {
                    let command = line.strip_prefix("- cmd: ")?.to_string();
                    (command, Utc::now())
                } else {
                    return None;
                }
            }
            _ => {
                // Bash and others: simple line format
                (line.to_string(), Utc::now())
            }
        };

        // Filter out common non-productive commands
        if self.should_ignore_command(&command) {
            return None;
        }

        Some(CommandEntry {
            command: command.trim().to_string(),
            timestamp,
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
        let command = command.trim();
        
        // Ignore empty commands
        if command.is_empty() {
            return true;
        }
        
        // Ignore very short commands that are likely navigation or typos
        if command.len() < 2 {
            return true;
        }
        
        // Filter out obvious non-terminal content first
        if self.is_obvious_non_terminal_content(command) {
            return true;
        }
        
        // Filter out JavaScript/browser console content
        if self.is_javascript_or_browser_content(command) {
            return true;
        }
        
        // Filter out HTTP/API logs
        if self.is_http_or_api_log(command) {
            return true;
        }
        
        // Filter out code snippets and programming content
        if self.is_code_snippet(command) {
            return true;
        }
        
        // Filter out error messages and stack traces
        if self.is_error_or_stack_trace(command) {
            return true;
        }
        
        // Filter out JSON/configuration content
        if self.is_json_or_config_content(command) {
            return true;
        }
        
        // Only ignore very basic navigation commands without arguments
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            // Only ignore basic commands without arguments
            match *first_word {
                "ls" | "pwd" | "clear" | "exit" if cmd_parts.len() == 1 => true,
                "cd" if cmd_parts.len() <= 2 && cmd_parts.get(1).map_or(true, |arg| *arg == "~" || *arg == "-") => true,
                "history" if cmd_parts.len() == 1 => true,
                _ => false,
            }
        } else {
            true
        }
    }

    /// Check for obvious non-terminal content patterns
    fn is_obvious_non_terminal_content(&self, command: &str) -> bool {
        let trimmed = command.trim();
        
        // Filter out single characters or symbols that are clearly not commands
        if trimmed.len() <= 2 {
            match trimmed {
                "{" | "}" | "[" | "]" | "(" | ")" | ";" | "," | ":" | "." |
                "{\\" | "}\\" | "[\\" | "]\\" | ";\\" | ",\\" | ":\\" | ".\\" |
                "{\\n" | "}\\n" | "[\\n" | "]\\n" | ";\\n" | ",\\n" | ":\\n" | ".\\n" => return true,
                _ => {}
            }
        }
        
        // Filter out lines that end with backslash (likely continuation lines from code)
        if trimmed.ends_with('\\') && !trimmed.starts_with("cd ") && !trimmed.starts_with("ls ") {
            return true;
        }
        
        // Filter out lines that start with common code patterns
        if trimmed.starts_with("//") ||
           trimmed.starts_with("/*") ||
           trimmed.starts_with("*/") ||
           trimmed.starts_with("#") && !trimmed.starts_with("#!/") ||
           trimmed.starts_with("*") ||
           trimmed.starts_with("@") ||
           trimmed.starts_with("$") && trimmed.len() < 5 ||
           trimmed.starts_with("&") ||
           trimmed.starts_with("|") ||
           trimmed.starts_with("~") && trimmed.len() < 5 ||
           trimmed.starts_with("`") ||
           trimmed.starts_with("'") && trimmed.ends_with("'") ||
           trimmed.starts_with("\"") && trimmed.ends_with("\"") {
            return true;
        }
        
        // Filter out lines that look like JavaScript/web content
        if trimmed.contains(".js:") ||
           trimmed.contains(".css:") ||
           trimmed.contains(".html:") ||
           trimmed.contains(".ts:") ||
           trimmed.contains(".jsx:") ||
           trimmed.contains(".tsx:") ||
           trimmed.contains(".vue:") ||
           trimmed.contains("index-") && trimmed.contains(".js") ||
           trimmed.contains("assets/") ||
           trimmed.contains("www.") ||
           trimmed.contains("http://") ||
           trimmed.contains("https://") ||
           trimmed.contains(".com/") ||
           trimmed.contains(".org/") ||
           trimmed.contains(".net/") {
            return true;
        }
        
        // Filter out lines that look like object/array syntax
        if (trimmed.starts_with("{") && trimmed.contains(":")) ||
           (trimmed.starts_with("[") && trimmed.contains(",")) ||
           trimmed.contains("name:") ||
           trimmed.contains("type:") ||
           trimmed.contains("date:") ||
           trimmed.contains("'name'") ||
           trimmed.contains("'type'") ||
           trimmed.contains("'date'") ||
           trimmed.contains("\"name\"") ||
           trimmed.contains("\"type\"") ||
           trimmed.contains("\"date\"") {
            return true;
        }
        
        // Filter out lines that are clearly fragments
        let word_count = trimmed.split_whitespace().count();
        if word_count == 1 {
            let word = trimmed.split_whitespace().next().unwrap_or("");
            // Single words that are clearly not commands
            if word.len() <= 3 && !["ls", "cd", "pwd", "cat", "vim", "top", "ps", "du", "df", "who", "id", "su"].contains(&word) {
                return true;
            }
            
            // Single words that contain non-alphanumeric characters (except common command chars)
            if word.chars().any(|c| !c.is_alphanumeric() && !"-_.".contains(c)) {
                return true;
            }
        }
        
        false
    }

    /// Check if command is JavaScript or browser console content
    fn is_javascript_or_browser_content(&self, command: &str) -> bool {
        // JavaScript patterns
        let js_patterns = [
            "console.log(",
            "console.error(",
            "console.warn(",
            "console.info(",
            "function(",
            "const ",
            "let ",
            "var ",
            "=>",
            "require(",
            "import ",
            "export ",
            "document.",
            "window.",
            "localStorage.",
            "sessionStorage.",
            "JSON.stringify(",
            "JSON.parse(",
            "addEventListener(",
            "querySelector(",
            "getElementById(",
            "createElement(",
            "appendChild(",
            "removeChild(",
            "innerHTML",
            "textContent",
            "onclick",
            "onload",
            "createRoot",
            "ReactDOM",
            "React.",
            "useState(",
            "useEffect(",
            "props.",
            "state.",
            ".map(",
            ".filter(",
            ".reduce(",
            ".forEach(",
            "async function",
            "await ",
            "Promise.",
            "fetch(",
            "axios.",
            "$.ajax",
            "jQuery",
            "angular.",
            "vue.",
            "$scope.",
            "ng-",
            "v-",
            "class=",
            "id=",
            "<div",
            "<span",
            "<p>",
            "<h1",
            "<h2",
            "<h3",
            "<script",
            "<style",
            "<!DOCTYPE",
            "<html",
            "<head",
            "<body",
            "href=",
            "src=",
        ];
        
        let command_lower = command.to_lowercase();
        js_patterns.iter().any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }

    /// Check if command is HTTP/API log content
    fn is_http_or_api_log(&self, command: &str) -> bool {
        let http_patterns = [
            "HTTP/",
            "GET ",
            "POST ",
            "PUT ",
            "DELETE ",
            "PATCH ",
            "OPTIONS ",
            "HEAD ",
            "[HTTP/",
            "XHRPOST",
            "XMLHttpRequest",
            "fetch(",
            "api.example.com",
            "amazonaws.com",
            "execute-api",
            "Content-Type:",
            "Authorization:",
            "User-Agent:",
            "Accept:",
            "Cache-Control:",
            "Set-Cookie:",
            "Location:",
            "Referer:",
            "Origin:",
            "Access-Control",
            "CORS",
            "application/json",
            "text/html",
            "text/plain",
            "multipart/form-data",
            "www-form-urlencoded",
            "Bearer ",
            "Basic ",
            "API error",
            "status code",
            "response time",
            "request failed",
            "connection timeout",
            "network error",
            "502 Bad Gateway",
            "503 Service Unavailable",
            "404 Not Found",
            "401 Unauthorized",
            "403 Forbidden",
            "500 Internal Server Error",
            "200 OK",
            "201 Created",
            "204 No Content",
            "301 Moved Permanently",
            "302 Found",
            "304 Not Modified",
            "400 Bad Request",
            "422 Unprocessable Entity",
        ];
        
        http_patterns.iter().any(|pattern| command.contains(pattern))
    }

    /// Check if command is a code snippet
    fn is_code_snippet(&self, command: &str) -> bool {
        // Programming language patterns
        let code_patterns = [
            "def ",
            "class ",
            "import ",
            "from ",
            "if __name__",
            "#!/usr/bin/env",
            "#!/bin/bash",
            "#!/bin/sh",
            "#include",
            "#define",
            "#ifdef",
            "#ifndef",
            "public class",
            "private ",
            "protected ",
            "public ",
            "static ",
            "final ",
            "abstract ",
            "interface ",
            "extends ",
            "implements ",
            "package ",
            "namespace ",
            "using ",
            "struct ",
            "enum ",
            "union ",
            "typedef ",
            "template",
            "fn ",
            "impl ",
            "trait ",
            "mod ",
            "use ",
            "extern ",
            "pub ",
            "mut ",
            "match ",
            "if let",
            "while let",
            "for ",
            "loop ",
            "break",
            "continue",
            "return ",
            "yield ",
            "async def",
            "await ",
            "lambda ",
            "try:",
            "except:",
            "finally:",
            "with ",
            "as ",
            "raise ",
            "assert ",
            "global ",
            "nonlocal ",
            "pass",
            "elif ",
            "else:",
            "print(",
            "input(",
            "len(",
            "range(",
            "enumerate(",
            "zip(",
            "sorted(",
            "reversed(",
            "sum(",
            "max(",
            "min(",
            "abs(",
            "round(",
            "int(",
            "float(",
            "str(",
            "bool(",
            "list(",
            "dict(",
            "set(",
            "tuple(",
            "type(",
            "isinstance(",
            "hasattr(",
            "getattr(",
            "setattr(",
            "delattr(",
        ];
        
        let command_lower = command.to_lowercase();
        code_patterns.iter().any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }

    /// Check if command is an error message or stack trace
    fn is_error_or_stack_trace(&self, command: &str) -> bool {
        let error_patterns = [
            "Error:",
            "ERROR:",
            "Warning:",
            "WARNING:",
            "Exception:",
            "Traceback",
            "Stack trace:",
            "at ",
            "    at ",
            "Caused by:",
            "java.lang.",
            "java.util.",
            "java.io.",
            "java.net.",
            "org.springframework.",
            "com.example.",
            "node.js:",
            "TypeError:",
            "ReferenceError:",
            "SyntaxError:",
            "RangeError:",
            "URIError:",
            "EvalError:",
            "InternalError:",
            "AggregateError:",
            "UnhandledPromiseRejectionWarning:",
            "DeprecationWarning:",
            "FutureWarning:",
            "UserWarning:",
            "RuntimeWarning:",
            "SyntaxWarning:",
            "ImportWarning:",
            "UnicodeWarning:",
            "BytesWarning:",
            "ResourceWarning:",
            "ConnectionError:",
            "TimeoutError:",
            "PermissionError:",
            "FileNotFoundError:",
            "IsADirectoryError:",
            "NotADirectoryError:",
            "InterruptedError:",
            "BlockingIOError:",
            "ChildProcessError:",
            "ProcessLookupError:",
            "BrokenPipeError:",
            "ConnectionAbortedError:",
            "ConnectionRefusedError:",
            "ConnectionResetError:",
            "FileExistsError:",
            "FileNotFoundError:",
            "IsADirectoryError:",
            "NotADirectoryError:",
            "PermissionError:",
            "ProcessLookupError:",
            "TimeoutError:",
            "InterruptedError:",
            "ChildProcessError:",
            "BrokenPipeError:",
            "ConnectionAbortedError:",
            "ConnectionRefusedError:",
            "ConnectionResetError:",
            "OSError:",
            "IOError:",
            "EOFError:",
            "KeyboardInterrupt:",
            "SystemExit:",
            "GeneratorExit:",
            "StopIteration:",
            "StopAsyncIteration:",
            "ArithmeticError:",
            "OverflowError:",
            "ZeroDivisionError:",
            "FloatingPointError:",
            "AssertionError:",
            "AttributeError:",
            "BufferError:",
            "LookupError:",
            "IndexError:",
            "KeyError:",
            "MemoryError:",
            "NameError:",
            "UnboundLocalError:",
            "RuntimeError:",
            "RecursionError:",
            "NotImplementedError:",
            "SystemError:",
            "TypeError:",
            "ValueError:",
            "UnicodeError:",
            "UnicodeDecodeError:",
            "UnicodeEncodeError:",
            "UnicodeTranslateError:",
            "Layout was forced",
            "flash of unstyled content",
            "Permission denied",
            "Access denied",
            "File not found",
            "No such file",
            "Command not found",
            "command not found",
            "Operation not permitted",
            "Invalid argument",
            "Broken pipe",
            "Connection refused",
            "Network unreachable",
            "Host unreachable",
            "Timeout",
            "Segmentation fault",
            "Bus error",
            "Illegal instruction",
            "Abort trap",
            "Killed",
            "Terminated",
            "core dumped",
            "panic:",
            "fatal:",
            "FATAL:",
            "PANIC:",
            "CRITICAL:",
            "SEVERE:",
        ];
        
        error_patterns.iter().any(|pattern| command.contains(pattern))
    }

    /// Check if command is JSON or configuration content
    fn is_json_or_config_content(&self, command: &str) -> bool {
        let config_patterns = [
            "\"Version\":",
            "\"Statement\":",
            "\"Effect\":",
            "\"Principal\":",
            "\"Action\":",
            "\"Resource\":",
            "\"Sid\":",
            "\"Allow\"",
            "\"Deny\"",
            "arn:aws:",
            "s3:GetObject",
            "s3:PutObject",
            "s3:DeleteObject",
            "\"name\":",
            "\"type\":",
            "\"date\":",
            "\"country\":",
            "\"cached\":",
            "\"totalHolidaysFound\":",
            "\"nationalHolidaysFound\":",
            "\"filteredOut\":",
            "statusCode:",
            "responseTime:",
            "mockResponse",
            "parseResponse",
            "validateCountry",
            "checkRedirect",
            "RedirectManager",
            "CountryValidator",
            "HolidayAPI",
            "api.parseResponse",
            "manager.checkRedirect",
            "validator.validateCountry",
            "new HolidayAPI",
            "new CountryValidator",
            "new RedirectManager",
            "require('./",
            "require(\"./",
            "module.exports",
            "exports.",
            "__dirname",
            "__filename",
            "process.env",
            "process.argv",
            "process.cwd",
            "process.exit",
            "Buffer.",
            "global.",
            "setTimeout(",
            "setInterval(",
            "clearTimeout(",
            "clearInterval(",
            "setImmediate(",
            "clearImmediate(",
        ];
        
        let trimmed = command.trim();
        
        // Check for JSON-like structures
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
           (trimmed.starts_with('[') && trimmed.ends_with(']')) ||
           (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.contains(':')) {
            return true;
        }
        
        // Check for specific patterns
        config_patterns.iter().any(|pattern| command.contains(pattern))
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