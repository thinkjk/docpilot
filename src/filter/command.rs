//! Command filtering and success/failure detection
//! 
//! This module provides functionality to detect command success/failure,
//! filter out failed commands, and validate command sequences.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

use crate::terminal::monitor::CommandEntry;

/// Criteria for filtering commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCriteria {
    /// Filter out commands that failed (non-zero exit code)
    pub exclude_failed: bool,
    /// Filter out commands with specific exit codes
    pub exclude_exit_codes: HashSet<i32>,
    /// Filter out commands that match certain patterns (typos, etc.)
    pub exclude_patterns: Vec<String>,
    /// Include only commands that succeeded (exit code 0)
    pub only_successful: bool,
    /// Maximum execution time before considering a command as potentially problematic
    pub max_execution_time: Option<Duration>,
    /// Enable command deduplication
    pub enable_deduplication: bool,
    /// Time window for considering commands as duplicates (in seconds)
    pub deduplication_window: u64,
    /// Enable workflow optimization
    pub enable_workflow_optimization: bool,
    /// Minimum command frequency to be considered for optimization
    pub min_frequency_for_optimization: usize,
    /// Enable privacy filtering for sensitive commands
    pub enable_privacy_filtering: bool,
    /// Privacy filtering mode (strict, moderate, lenient)
    pub privacy_mode: PrivacyMode,
    /// Custom sensitive patterns to filter
    pub custom_sensitive_patterns: Vec<String>,
    /// Enable command sequence validation
    pub enable_sequence_validation: bool,
    /// Validate command dependencies and prerequisites
    pub validate_dependencies: bool,
    /// Suggest fixes for broken command sequences
    pub suggest_fixes: bool,
}

/// Privacy filtering modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyMode {
    /// Strict: Filter out all potentially sensitive information
    Strict,
    /// Moderate: Filter out obvious sensitive data but allow some context
    Moderate,
    /// Lenient: Only filter out clearly sensitive patterns like passwords
    Lenient,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        PrivacyMode::Moderate
    }
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            exclude_failed: true,
            exclude_exit_codes: HashSet::from([1, 2, 126, 127, 130]), // Common error codes
            exclude_patterns: vec![
                // Common typos and mistakes
                "sl".to_string(),     // typo for 'ls'
                "gti".to_string(),    // typo for 'git'
                "cd..".to_string(),   // missing space
                "..".to_string(),     // incomplete command
                "q".to_string(),      // accidental quit
                "x".to_string(),      // accidental command
                // Additional common typos
                "claer".to_string(),  // typo for 'clear'
                "exot".to_string(),   // typo for 'exit'
                "grpe".to_string(),   // typo for 'grep'
                "mkdri".to_string(),  // typo for 'mkdir'
                "tial".to_string(),   // typo for 'tail'
                "ehco".to_string(),   // typo for 'echo'
                "cta".to_string(),    // typo for 'cat'
                "mvoe".to_string(),   // typo for 'move'
                "cpoy".to_string(),   // typo for 'copy'
                "sudp".to_string(),   // typo for 'sudo'
                "whihc".to_string(),  // typo for 'which'
                "finde".to_string(),  // typo for 'find'
                "killl".to_string(),  // typo for 'kill'
                "pign".to_string(),   // typo for 'ping'
                "wgte".to_string(),   // typo for 'wget'
                "curll".to_string(),  // typo for 'curl'
                "vmi".to_string(),    // typo for 'vim'
                "naon".to_string(),   // typo for 'nano'
                "emcas".to_string(),  // typo for 'emacs'
                "tpo".to_string(),    // typo for 'top'
                "htpo".to_string(),   // typo for 'htop'
                "pws".to_string(),    // typo for 'ps'
                "duf".to_string(),    // typo for 'du'
                "fre".to_string(),    // typo for 'free'
                "upitme".to_string(), // typo for 'uptime'
                "histroy".to_string(), // typo for 'history'
                "alais".to_string(),  // typo for 'alias'
                "soruce".to_string(), // typo for 'source'
                "exprot".to_string(), // typo for 'export'
                "unste".to_string(),  // typo for 'unset'
                "chmdo".to_string(),  // typo for 'chmod'
                "chonw".to_string(),  // typo for 'chown'
                "tarr".to_string(),   // typo for 'tar'
                "ziip".to_string(),   // typo for 'zip'
                "unzpi".to_string(),  // typo for 'unzip'
                "sssh".to_string(),   // typo for 'ssh'
                "scpp".to_string(),   // typo for 'scp'
                "rsyncc".to_string(), // typo for 'rsync'
                "moutnt".to_string(), // typo for 'mount'
                "umoutnt".to_string(), // typo for 'umount'
                "fdiksk".to_string(), // typo for 'fdisk'
                "lsblkk".to_string(), // typo for 'lsblk'
                "systemclt".to_string(), // typo for 'systemctl'
                "servicce".to_string(), // typo for 'service'
                "aptget".to_string(), // missing space in 'apt-get'
                "yumm".to_string(),   // typo for 'yum'
                "dnff".to_string(),   // typo for 'dnf'
                "pacmna".to_string(), // typo for 'pacman'
                "breww".to_string(),  // typo for 'brew'
                "snapp".to_string(),  // typo for 'snap'
                "dockerr".to_string(), // typo for 'docker'
                "kubectll".to_string(), // typo for 'kubectl'
                "helmm".to_string(),  // typo for 'helm'
                "terrafrm".to_string(), // typo for 'terraform'
                "ansibel".to_string(), // typo for 'ansible'
                "vagrnant".to_string(), // typo for 'vagrant'
                "nodee".to_string(),  // typo for 'node'
                "npmm".to_string(),   // typo for 'npm'
                "yarnn".to_string(),  // typo for 'yarn'
                "piip".to_string(),   // typo for 'pip'
                "condaa".to_string(), // typo for 'conda'
                "cargoo".to_string(), // typo for 'cargo'
                "rustcc".to_string(), // typo for 'rustc'
                "pythno".to_string(), // typo for 'python'
                "rubby".to_string(),  // typo for 'ruby'
                "goo".to_string(),    // typo for 'go'
                "gcccc".to_string(),  // typo for 'gcc'
                "makee".to_string(),  // typo for 'make'
                "cmakee".to_string(), // typo for 'cmake'
                "ninaj".to_string(),  // typo for 'ninja'
                "baezl".to_string(),  // typo for 'bazel'
                "gradel".to_string(), // typo for 'gradle'
                "mavne".to_string(),  // typo for 'maven'
                "antt".to_string(),   // typo for 'ant'
            ],
            only_successful: false,
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes
            enable_deduplication: true,
            deduplication_window: 300, // 5 minutes
            enable_workflow_optimization: true,
            min_frequency_for_optimization: 3,
            enable_privacy_filtering: true,
            privacy_mode: PrivacyMode::default(),
            custom_sensitive_patterns: Vec::new(),
            enable_sequence_validation: true,
            validate_dependencies: true,
            suggest_fixes: true,
        }
    }
}

/// Result of command filtering
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub should_include: bool,
    pub reason: String,
    pub confidence: f32, // 0.0 to 1.0
}

/// Command dependency information
#[derive(Debug, Clone)]
pub struct CommandDependency {
    pub command_pattern: String,
    pub required_files: Vec<String>,
    pub required_commands: Vec<String>,
    pub required_environment: Vec<String>,
    pub description: String,
}

/// Validation result for command sequences
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub missing_dependencies: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub confidence: f32,
    pub validation_type: ValidationType,
}

/// Types of validation performed
#[derive(Debug, Clone)]
pub enum ValidationType {
    DependencyCheck,
    SequenceValidation,
    PrerequisiteValidation,
    EnvironmentValidation,
}

/// Command sequence validation error
#[derive(Debug, Clone)]
pub struct SequenceValidationError {
    pub command: String,
    pub error_type: ValidationErrorType,
    pub description: String,
    pub suggested_fix: Option<String>,
    pub confidence: f32,
}

/// Types of validation errors
#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    MissingFile,
    MissingCommand,
    MissingEnvironment,
    BrokenSequence,
    InvalidPrerequisite,
}

/// Command filter for success/failure detection and validation
#[derive(Debug)]
pub struct CommandFilter {
    criteria: FilterCriteria,
}

impl CommandFilter {
    /// Create a new command filter with default criteria
    pub fn new() -> Self {
        Self {
            criteria: FilterCriteria::default(),
        }
    }

    /// Create a new command filter with custom criteria
    pub fn with_criteria(criteria: FilterCriteria) -> Self {
        Self { criteria }
    }

    /// Update filter criteria
    pub fn set_criteria(&mut self, criteria: FilterCriteria) {
        self.criteria = criteria;
    }

    /// Get current filter criteria
    pub fn get_criteria(&self) -> &FilterCriteria {
        &self.criteria
    }

    /// Filter a single command entry
    pub fn filter_command(&self, command: &CommandEntry) -> FilterResult {
        // Check exit code if available
        if let Some(exit_code) = command.exit_code {
            // Check only_successful first (more specific)
            if self.criteria.only_successful && exit_code != 0 {
                return FilterResult {
                    should_include: false,
                    reason: "Only successful commands are included".to_string(),
                    confidence: 1.0,
                };
            }

            // Check exclude_failed (unless only_successful is already handling it)
            if self.criteria.exclude_failed && !self.criteria.only_successful && exit_code != 0 {
                return FilterResult {
                    should_include: false,
                    reason: format!("Command failed with exit code {}", exit_code),
                    confidence: 1.0,
                };
            }

            if self.criteria.exclude_exit_codes.contains(&exit_code) {
                return FilterResult {
                    should_include: false,
                    reason: format!("Exit code {} is in exclusion list", exit_code),
                    confidence: 1.0,
                };
            }
        }

        // Check for pattern matches (typos, etc.)
        let command_lower = command.command.to_lowercase();
        for pattern in &self.criteria.exclude_patterns {
            if command_lower.contains(&pattern.to_lowercase()) {
                return FilterResult {
                    should_include: false,
                    reason: format!("Command matches exclusion pattern: {}", pattern),
                    confidence: 0.8,
                };
            }
        }

        // Check for common command failure indicators in output/error
        if let Some(error) = &command.error {
            if self.contains_failure_indicators(error) {
                return FilterResult {
                    should_include: false,
                    reason: "Command output contains failure indicators".to_string(),
                    confidence: 0.7,
                };
            }
        }

        // Also check output field for failure indicators
        if let Some(output) = &command.output {
            if self.contains_failure_indicators(output) {
                return FilterResult {
                    should_include: false,
                    reason: "Command output contains failure indicators".to_string(),
                    confidence: 0.7,
                };
            }
        }

        // Check for suspicious command patterns
        if self.is_suspicious_command(&command.command) {
            return FilterResult {
                should_include: false,
                reason: "Command appears to be a typo or mistake".to_string(),
                confidence: 0.6,
            };
        }

        FilterResult {
            should_include: true,
            reason: "Command passed all filters".to_string(),
            confidence: 1.0,
        }
    }

    /// Sanitize a command by redacting sensitive information
    pub fn sanitize_command(&self, command: &CommandEntry) -> CommandEntry {
        if !self.criteria.enable_privacy_filtering {
            return command.clone();
        }

        let mut sanitized = command.clone();
        sanitized.command = self.redact_sensitive_data(&command.command);
        
        // Also sanitize output and error if they contain sensitive data
        if let Some(output) = &command.output {
            sanitized.output = Some(self.redact_sensitive_data(output));
        }
        if let Some(error) = &command.error {
            sanitized.error = Some(self.redact_sensitive_data(error));
        }

        sanitized
    }

    /// Redact sensitive data from a text string
    fn redact_sensitive_data(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Apply privacy filtering based on mode
        match self.criteria.privacy_mode {
            PrivacyMode::Strict => {
                result = self.apply_strict_privacy_filtering(&result);
            }
            PrivacyMode::Moderate => {
                result = self.apply_moderate_privacy_filtering(&result);
            }
            PrivacyMode::Lenient => {
                result = self.apply_lenient_privacy_filtering(&result);
            }
        }

        // Apply custom sensitive patterns
        for pattern in &self.criteria.custom_sensitive_patterns {
            result = self.redact_pattern(&result, pattern);
        }

        result
    }

    /// Apply strict privacy filtering (redacts most potentially sensitive data)
    fn apply_strict_privacy_filtering(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Apply all moderate and lenient filters
        result = self.apply_moderate_privacy_filtering(&result);
        
        // Additional strict patterns
        result = self.redact_ip_addresses(&result);
        result = self.redact_file_paths(&result);
        result = self.redact_usernames(&result);
        result = self.redact_hostnames(&result);
        
        result
    }

    /// Apply moderate privacy filtering (redacts obvious sensitive data)
    fn apply_moderate_privacy_filtering(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Apply all lenient filters
        result = self.apply_lenient_privacy_filtering(&result);
        
        // Additional moderate patterns
        result = self.redact_api_keys(&result);
        result = self.redact_tokens(&result);
        result = self.redact_secrets(&result);
        result = self.redact_credentials(&result);
        
        result
    }

    /// Apply lenient privacy filtering (only redacts obvious passwords and keys)
    fn apply_lenient_privacy_filtering(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Basic password patterns
        result = self.redact_passwords(&result);
        result = self.redact_ssh_keys(&result);
        result = self.redact_certificates(&result);
        
        result
    }

    /// Redact password patterns
    fn redact_passwords(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            // Command line password arguments
            r"(-p|--password|--pass)\s+\S+",
            r"(password=|pwd=|pass=)\S+",
            // Environment variables with passwords
            r"(PASSWORD|PWD|PASS)=[^\s]+",
            // MySQL/PostgreSQL connection strings
            r"mysql://[^:]+:[^@]+@",
            r"postgresql://[^:]+:[^@]+@",
            // Generic password patterns in URLs
            r"://[^:]+:[^@]+@",
            // curl -u user:password format - match the whole thing
            r"-u\s+[^:\s]+:[^\s]+",
            // JSON password patterns
            r#""password"\s*:\s*"[^"]+""#,
            // Text patterns with password
            r"(?i)password\s+[^\s]+",
            r"(?i)with\s+password\s+[^\s]+",
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact SSH key patterns
    fn redact_ssh_keys(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            // Specific SSH key patterns (not generic BEGIN/END blocks)
            r"ssh-rsa [A-Za-z0-9+/=]+",
            r"ssh-ed25519 [A-Za-z0-9+/=]+",
            r"ssh-dss [A-Za-z0-9+/=]+",
            r"ssh-ecdsa [A-Za-z0-9+/=]+",
            // SSH private key patterns (more specific)
            r"(?s)-----BEGIN OPENSSH PRIVATE KEY-----.*?-----END OPENSSH PRIVATE KEY-----",
            r"(?s)-----BEGIN RSA PRIVATE KEY-----.*?-----END RSA PRIVATE KEY-----",
            r"(?s)-----BEGIN DSA PRIVATE KEY-----.*?-----END DSA PRIVATE KEY-----",
            r"(?s)-----BEGIN EC PRIVATE KEY-----.*?-----END EC PRIVATE KEY-----",
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[SSH_KEY_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact certificate patterns
    fn redact_certificates(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r"(?s)-----BEGIN CERTIFICATE-----.*?-----END CERTIFICATE-----",
            r"(?s)-----BEGIN PRIVATE KEY-----.*?-----END PRIVATE KEY-----",
            r"(?s)-----BEGIN RSA PRIVATE KEY-----.*?-----END RSA PRIVATE KEY-----",
            r"(?s)-----BEGIN PUBLIC KEY-----.*?-----END PUBLIC KEY-----",
            // More flexible patterns for test certificates
            r"(?s)-----BEGIN.*?CERTIFICATE.*?-----.*?-----END.*?CERTIFICATE.*?-----",
            r"(?s)-----BEGIN.*?KEY.*?-----.*?-----END.*?KEY.*?-----",
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[CERTIFICATE_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact API key patterns
    fn redact_api_keys(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            // Generic API key patterns
            r#"(api[_-]?key|apikey)[:=]\s*['"]?([A-Za-z0-9_-]{3,})['"]?"#,
            r#"(access[_-]?key|accesskey)[:=]\s*['"]?([A-Za-z0-9_-]{3,})['"]?"#,
            // AWS keys
            r"AKIA[0-9A-Z]{16}",
            r#"aws[_-]?secret[_-]?access[_-]?key[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#,
            // GitHub tokens
            r"ghp_[A-Za-z0-9]{36}",
            r#"github[_-]?token[:=]\s*['"]?([A-Za-z0-9_-]{40})['"]?"#,
            // Google API keys
            r"AIza[0-9A-Za-z_-]{35}",
            // Simple API_KEY=value pattern
            r"API_KEY=[A-Za-z0-9_-]+",
            // JSON API key patterns
            r#""api_key"\s*:\s*"[^"]+""#,
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[API_KEY_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact token patterns
    fn redact_tokens(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r#"(?i)(token|bearer)[:=]\s*['"]?([A-Za-z0-9_.-]{10,})['"]?"#,
            r"(?i)Authorization:\s*Bearer\s+([A-Za-z0-9_.-]+)",
            r#"(?i)(jwt|refresh[_-]?token)[:=]\s*['"]?([A-Za-z0-9_.-]{10,})['"]?"#,
            r#"(?i)(auth[_-]?token|api[_-]?token)[:=]\s*['"]?([A-Za-z0-9_.-]{10,})['"]?"#,
            // JSON token patterns
            r#""token"\s*:\s*"[^"]+""#,
            // GitHub tokens
            r"ghp_[A-Za-z0-9]{36}",
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[TOKEN_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact secret patterns
    fn redact_secrets(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r#"(secret|private[_-]?key)[:=]\s*['"]?([A-Za-z0-9_/+=]{20,})['"]?"#,
            r#"(client[_-]?secret|app[_-]?secret)[:=]\s*['"]?([A-Za-z0-9_-]{20,})['"]?"#,
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, |caps: &regex::Captures| {
                    if caps.len() > 1 {
                        format!("{}[SECRET_REDACTED]", caps.get(1).map(|m| m.as_str()).unwrap_or(""))
                    } else {
                        "[SECRET_REDACTED]".to_string()
                    }
                }).to_string();
            }
        }
        
        result
    }

    /// Redact credential patterns
    fn redact_credentials(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r#"(username|user)[:=]\s*['"]?([A-Za-z0-9_.-]+)['"]?\s+(password|pass)[:=]\s*['"]?([A-Za-z0-9_.-]+)['"]?"#,
            r#"(login|auth)[:=]\s*['"]?([A-Za-z0-9_.-]+:[A-Za-z0-9_.-]+)['"]?"#,
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[CREDENTIALS_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact IP addresses
    fn redact_ip_addresses(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b",  // IPv4
            r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b",  // IPv6 (simplified)
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[IP_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact file paths that might contain sensitive information
    fn redact_file_paths(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r"/home/[^/\s]+",  // Home directories
            r"/Users/[^/\s]+", // macOS home directories
            r"C:\\Users\\[^\\s]+", // Windows user directories
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[PATH_REDACTED]").to_string();
            }
        }
        
        result
    }

    /// Redact usernames
    fn redact_usernames(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r#"(user|username|login)[:=]\s*['"]?([A-Za-z0-9_.-]+)['"]?"#,
            r"@([A-Za-z0-9_.-]+)\s", // Email-like usernames
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, |caps: &regex::Captures| {
                    if caps.len() > 1 {
                        format!("{}[USERNAME_REDACTED]", caps.get(1).map(|m| m.as_str()).unwrap_or(""))
                    } else {
                        "[USERNAME_REDACTED]".to_string()
                    }
                }).to_string();
            }
        }
        
        result
    }

    /// Redact hostnames and server names
    fn redact_hostnames(&self, text: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r#"(host|hostname|server)[:=]\s*['"]?([A-Za-z0-9.-]+)['"]?"#,
            r"@([A-Za-z0-9.-]+):", // SSH-style host references
        ];

        let mut result = text.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, |caps: &regex::Captures| {
                    if caps.len() > 1 {
                        format!("{}[HOSTNAME_REDACTED]", caps.get(1).map(|m| m.as_str()).unwrap_or(""))
                    } else {
                        "[HOSTNAME_REDACTED]".to_string()
                    }
                }).to_string();
            }
        }
        
        result
    }

    /// Redact a custom pattern
    fn redact_pattern(&self, text: &str, pattern: &str) -> String {
        use regex::Regex;
        
        if let Ok(re) = Regex::new(pattern) {
            re.replace_all(text, "[CUSTOM_REDACTED]").to_string()
        } else {
            text.to_string()
        }
    }

    /// Filter a list of commands
    pub fn filter_commands(&self, commands: &[CommandEntry]) -> Vec<(CommandEntry, FilterResult)> {
        commands
            .iter()
            .map(|cmd| (cmd.clone(), self.filter_command(cmd)))
            .collect()
    }

    /// Get only the commands that should be included
    pub fn get_filtered_commands(&self, commands: &[CommandEntry]) -> Vec<CommandEntry> {
        self.filter_commands(commands)
            .into_iter()
            .filter_map(|(cmd, result)| {
                if result.should_include {
                    Some(cmd)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Detect command success/failure by attempting to re-execute it
    /// This is useful for commands without exit codes in history
    pub async fn detect_command_status(&self, command: &str) -> Result<bool> {
        // For safety, only test read-only commands
        if !self.is_safe_to_test(command) {
            return Err(anyhow!("Command is not safe to re-execute for testing"));
        }

        let result = timeout(
            Duration::from_secs(10), // 10 second timeout for testing
            async {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
            }
        ).await;

        match result {
            Ok(Ok(status)) => Ok(status.success()),
            Ok(Err(_)) => Ok(false), // Command execution failed
            Err(_) => Ok(false), // Timeout
        }
    }

    /// Check if a command contains failure indicators in its output
    fn contains_failure_indicators(&self, text: &str) -> bool {
        let failure_patterns = [
            "error:", "Error:", "ERROR:",
            "failed", "Failed", "FAILED",
            "not found", "Not found", "NOT FOUND",
            "permission denied", "Permission denied",
            "no such file", "No such file",
            "command not found",
            "syntax error",
            "invalid option",
            "cannot access",
            "operation not permitted",
        ];

        let text_lower = text.to_lowercase();
        failure_patterns.iter().any(|pattern| text_lower.contains(&pattern.to_lowercase()))
    }

    /// Check if a command appears to be suspicious (typo, mistake, etc.)
    fn is_suspicious_command(&self, command: &str) -> bool {
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            // Check for very short commands that might be typos
            if first_word.len() == 1 && !["l", "w", "q"].contains(first_word) {
                return true;
            }

            // Check for commands with unusual character patterns
            if first_word.chars().all(|c| c.is_ascii_punctuation()) {
                return true;
            }

            // Check for repeated characters (likely typos)
            if self.has_repeated_chars(first_word) {
                return true;
            }
        }

        false
    }

    /// Check if a command is safe to re-execute for testing purposes
    fn is_safe_to_test(&self, command: &str) -> bool {
        let safe_commands = [
            "ls", "pwd", "whoami", "date", "echo", "cat", "head", "tail",
            "grep", "find", "which", "type", "file", "stat", "wc",
        ];

        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            safe_commands.contains(first_word)
        } else {
            false
        }
    }

    /// Check if a string has suspicious repeated characters
    fn has_repeated_chars(&self, s: &str) -> bool {
        if s.len() < 3 {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();
        for i in 0..chars.len() - 2 {
            if chars[i] == chars[i + 1] && chars[i + 1] == chars[i + 2] {
                return true;
            }
        }

        false
    }

    /// Advanced typo detection using edit distance and common patterns
    pub fn is_likely_typo(&self, command: &str) -> bool {
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = cmd_parts.first() {
            // Check against common commands with edit distance
            let common_commands = [
                "ls", "cd", "pwd", "cat", "echo", "grep", "find", "which", "chmod", "chown",
                "cp", "mv", "rm", "mkdir", "rmdir", "touch", "head", "tail", "less", "more",
                "ps", "top", "kill", "jobs", "bg", "fg", "nohup", "screen", "tmux",
                "git", "svn", "hg", "bzr", "cvs", "make", "cmake", "gcc", "g++", "clang",
                "python", "python3", "node", "npm", "yarn", "pip", "cargo", "rustc",
                "java", "javac", "scala", "go", "ruby", "perl", "php", "bash", "zsh",
                "vim", "emacs", "nano", "code", "subl", "atom", "gedit",
                "ssh", "scp", "rsync", "wget", "curl", "ping", "telnet", "ftp", "sftp",
                "tar", "gzip", "gunzip", "zip", "unzip", "7z", "rar", "unrar",
                "mount", "umount", "df", "du", "free", "uptime", "uname", "whoami",
                "sudo", "su", "passwd", "useradd", "userdel", "usermod", "groups",
                "systemctl", "service", "crontab", "at", "batch", "watch",
                "apt", "apt-get", "yum", "dnf", "pacman", "brew", "snap", "flatpak",
                "docker", "kubectl", "helm", "terraform", "ansible", "vagrant",
                "history", "alias", "unalias", "source", "export", "unset", "env",
                "date", "cal", "bc", "expr", "seq", "sort", "uniq", "cut", "awk", "sed",
                "file", "stat", "lsof", "netstat", "ss", "iptables", "firewall-cmd",
                "fdisk", "lsblk", "blkid", "mkfs", "fsck", "badblocks",
            ];

            for correct_cmd in &common_commands {
                if self.edit_distance(first_word, correct_cmd) == 1 && first_word != correct_cmd {
                    return true;
                }
            }

            // Check for keyboard layout mistakes (qwerty adjacent keys)
            if self.is_keyboard_typo(first_word) {
                return true;
            }

            // Check for common character swaps
            if self.has_character_swaps(first_word) {
                return true;
            }
        }

        false
    }

    /// Calculate edit distance between two strings (Levenshtein distance)
    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[len1][len2]
    }

    /// Check for keyboard layout typos (adjacent key presses)
    fn is_keyboard_typo(&self, word: &str) -> bool {
        let qwerty_adjacent = [
            ('q', vec!['w', 'a', 's']),
            ('w', vec!['q', 'e', 'a', 's', 'd']),
            ('e', vec!['w', 'r', 's', 'd', 'f']),
            ('r', vec!['e', 't', 'd', 'f', 'g']),
            ('t', vec!['r', 'y', 'f', 'g', 'h']),
            ('y', vec!['t', 'u', 'g', 'h', 'j']),
            ('u', vec!['y', 'i', 'h', 'j', 'k']),
            ('i', vec!['u', 'o', 'j', 'k', 'l']),
            ('o', vec!['i', 'p', 'k', 'l']),
            ('p', vec!['o', 'l']),
            ('a', vec!['q', 'w', 's', 'z', 'x']),
            ('s', vec!['q', 'w', 'e', 'a', 'd', 'z', 'x', 'c']),
            ('d', vec!['w', 'e', 'r', 's', 'f', 'x', 'c', 'v']),
            ('f', vec!['e', 'r', 't', 'd', 'g', 'c', 'v', 'b']),
            ('g', vec!['r', 't', 'y', 'f', 'h', 'v', 'b', 'n']),
            ('h', vec!['t', 'y', 'u', 'g', 'j', 'b', 'n', 'm']),
            ('j', vec!['y', 'u', 'i', 'h', 'k', 'n', 'm']),
            ('k', vec!['u', 'i', 'o', 'j', 'l', 'm']),
            ('l', vec!['i', 'o', 'p', 'k']),
            ('z', vec!['a', 's', 'x']),
            ('x', vec!['a', 's', 'd', 'z', 'c']),
            ('c', vec!['s', 'd', 'f', 'x', 'v']),
            ('v', vec!['d', 'f', 'g', 'c', 'b']),
            ('b', vec!['f', 'g', 'h', 'v', 'n']),
            ('n', vec!['g', 'h', 'j', 'b', 'm']),
            ('m', vec!['h', 'j', 'k', 'n']),
        ];

        let adjacent_map: std::collections::HashMap<char, Vec<char>> = qwerty_adjacent.into_iter().collect();

        // Check if word could be formed by adjacent key mistakes
        let chars: Vec<char> = word.to_lowercase().chars().collect();
        if chars.len() < 2 {
            return false;
        }

        // Look for patterns where consecutive characters are keyboard-adjacent
        let mut adjacent_count = 0;
        for i in 0..chars.len() - 1 {
            if let Some(adjacent_keys) = adjacent_map.get(&chars[i]) {
                if adjacent_keys.contains(&chars[i + 1]) {
                    adjacent_count += 1;
                }
            }
        }

        // If more than half the character pairs are adjacent, likely a typo
        adjacent_count > chars.len() / 3
    }

    /// Check for common character swaps (transpositions)
    fn has_character_swaps(&self, word: &str) -> bool {
        let common_commands = ["ls", "cd", "git", "cat", "echo", "grep", "find", "chmod"];
        
        for correct_cmd in &common_commands {
            if self.is_transposition(word, correct_cmd) {
                return true;
            }
        }

        false
    }

    /// Check if one string is a transposition of another
    fn is_transposition(&self, s1: &str, s2: &str) -> bool {
        if s1.len() != s2.len() || s1.len() < 2 {
            return false;
        }

        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();

        let mut diff_positions = Vec::new();
        for i in 0..chars1.len() {
            if chars1[i] != chars2[i] {
                diff_positions.push(i);
            }
        }

        // Exactly two differences that are swapped
        if diff_positions.len() == 2 {
            let pos1 = diff_positions[0];
            let pos2 = diff_positions[1];
            return chars1[pos1] == chars2[pos2] && chars1[pos2] == chars2[pos1];
        }

        false
    }

    /// Enhanced failure detection with more patterns
    pub fn is_command_failed(&self, command: &CommandEntry) -> bool {
        // Check exit code first
        if let Some(exit_code) = command.exit_code {
            if exit_code != 0 {
                return true;
            }
        }

        // Check error output
        if let Some(error) = &command.error {
            if self.contains_failure_indicators(error) {
                return true;
            }
        }

        // Check output for failure patterns
        if let Some(output) = &command.output {
            if self.contains_failure_indicators(output) {
                return true;
            }
        }

        // Check for commands that typically indicate failure
        let failure_indicating_commands = [
            "command not found",
            "permission denied",
            "no such file",
            "cannot access",
            "operation not permitted",
            "invalid option",
            "syntax error",
            "segmentation fault",
            "core dumped",
            "killed",
            "terminated",
            "aborted",
            "timeout",
            "connection refused",
            "network unreachable",
            "host unreachable",
        ];

        let cmd_lower = command.command.to_lowercase();
        for pattern in &failure_indicating_commands {
            if cmd_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Get filtering statistics for a set of commands
    pub fn get_filtering_stats(&self, commands: &[CommandEntry]) -> FilteringStats {
        let mut stats = FilteringStats::default();
        stats.total_commands = commands.len();

        for command in commands {
            let result = self.filter_command(command);
            
            if result.should_include {
                stats.included_commands += 1;
            } else {
                stats.excluded_commands += 1;
                
                if result.reason.contains("failed") || result.reason.contains("exit code") {
                    stats.failed_commands += 1;
                } else if result.reason.contains("pattern") || result.reason.contains("typo") {
                    stats.typo_commands += 1;
                } else if result.reason.contains("suspicious") {
                    stats.suspicious_commands += 1;
                } else if result.reason.contains("failure indicators") {
                    stats.error_output_commands += 1;
                }
            }

            // Check if command contains sensitive data that would be privacy filtered
            if self.criteria.enable_privacy_filtering && self.contains_sensitive_data(&command.command) {
                stats.privacy_filtered_commands += 1;
            }
        }

        stats
    }

    /// Check if a command contains sensitive data
    fn contains_sensitive_data(&self, text: &str) -> bool {
        // Check if the command would be modified by privacy filtering
        let original = text.to_string();
        let sanitized = self.redact_sensitive_data(text);
        original != sanitized
    }

    /// Get filtered and sanitized commands
    pub fn get_filtered_and_sanitized_commands(&self, commands: &[CommandEntry]) -> Vec<CommandEntry> {
        self.filter_commands(commands)
            .into_iter()
            .filter_map(|(cmd, result)| {
                if result.should_include {
                    Some(self.sanitize_command(&cmd))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Process commands with full filtering, deduplication, optimization, and privacy filtering
    pub fn process_commands_with_privacy(&self, commands: &[CommandEntry]) -> ProcessedCommands {
        // First apply basic filtering
        let filtered = self.get_filtered_commands(commands);
        
        // Apply privacy filtering to sanitize sensitive data
        let sanitized: Vec<CommandEntry> = if self.criteria.enable_privacy_filtering {
            filtered.iter().map(|cmd| self.sanitize_command(cmd)).collect()
        } else {
            filtered
        };
        
        // Then deduplicate
        let deduplicated = self.deduplicate_commands(&sanitized);
        
        // Find optimizations
        let optimizations = self.optimize_workflow(&deduplicated);
        
        ProcessedCommands {
            original_count: commands.len(),
            filtered_commands: deduplicated,
            optimizations,
            stats: self.get_filtering_stats(commands),
        }
    }

    /// Deduplicate commands based on criteria
    pub fn deduplicate_commands(&self, commands: &[CommandEntry]) -> Vec<CommandEntry> {
        if !self.criteria.enable_deduplication {
            return commands.to_vec();
        }

        let mut deduplicated = Vec::new();
        let mut seen_commands = std::collections::HashMap::new();

        for command in commands {
            let key = self.create_deduplication_key(command);
            
            if let Some(last_timestamp) = seen_commands.get(&key) {
                // Check if within deduplication window
                let time_diff = command.timestamp.signed_duration_since(*last_timestamp);
                if time_diff.num_seconds() < self.criteria.deduplication_window as i64 {
                    continue; // Skip duplicate within window
                }
            }

            seen_commands.insert(key, command.timestamp);
            deduplicated.push(command.clone());
        }

        deduplicated
    }

    /// Create a key for deduplication based on command content
    fn create_deduplication_key(&self, command: &CommandEntry) -> String {
        // Normalize the command for deduplication
        let normalized = self.normalize_command(&command.command);
        format!("{}:{}", normalized, command.working_directory)
    }

    /// Normalize command for comparison (remove variable parts)
    fn normalize_command(&self, command: &str) -> String {
        let mut normalized = command.to_lowercase();
        
        // Remove common variable patterns
        normalized = self.remove_timestamps(&normalized);
        normalized = self.remove_file_paths(&normalized);
        normalized = self.remove_process_ids(&normalized);
        normalized = self.remove_port_numbers(&normalized);
        
        // Normalize whitespace
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        
        normalized
    }

    /// Remove timestamp patterns from commands
    fn remove_timestamps(&self, command: &str) -> String {
        use regex::Regex;
        
        // Common timestamp patterns
        let patterns = [
            r"\d{4}-\d{2}-\d{2}",           // YYYY-MM-DD
            r"\d{2}:\d{2}:\d{2}",           // HH:MM:SS
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}", // ISO format
            r"\d{10,13}",                   // Unix timestamps
        ];

        let mut result = command.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "TIMESTAMP").to_string();
            }
        }
        
        result
    }

    /// Remove file path patterns that might be variable
    fn remove_file_paths(&self, command: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r"/tmp/[^\s]+",                 // Temp files
            r"/var/log/[^\s]+",             // Log files
            r"\.log\.\d+",                  // Rotated logs
            r"/proc/\d+",                   // Process directories
            r"\.tmp\.\w+",                  // Temp files with random names
        ];

        let mut result = command.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "FILEPATH").to_string();
            }
        }
        
        result
    }

    /// Remove process ID patterns
    fn remove_process_ids(&self, command: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r"\bpid\s+\d+",                 // pid 1234
            r"\bkill\s+\d{3,6}\b",          // kill 1234 (more specific PID context)
            r"\bps\s+.*\b\d{4,6}\b",        // ps output with PIDs
        ];

        let mut result = command.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "PID").to_string();
            }
        }
        
        result
    }

    /// Remove port number patterns
    fn remove_port_numbers(&self, command: &str) -> String {
        use regex::Regex;
        
        let patterns = [
            r":\d{2,5}\b",                  // :8080, :3000, etc.
            r"port\s+\d+",                  // port 8080
        ];

        let mut result = command.to_string();
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, ":PORT").to_string();
            }
        }
        
        result
    }

    /// Optimize workflow by identifying and consolidating patterns
    pub fn optimize_workflow(&self, commands: &[CommandEntry]) -> Vec<WorkflowOptimization> {
        if !self.criteria.enable_workflow_optimization {
            return Vec::new();
        }

        let mut optimizations = Vec::new();
        
        // Find command patterns
        let patterns = self.find_command_patterns(commands);
        
        // Find redundant sequences
        let redundant = self.find_redundant_sequences(commands);
        
        // Find repeated directory changes
        let directory_optimizations = self.find_directory_optimizations(commands);
        
        optimizations.extend(patterns);
        optimizations.extend(redundant);
        optimizations.extend(directory_optimizations);
        
        optimizations
    }

    /// Find frequently repeated command patterns
    fn find_command_patterns(&self, commands: &[CommandEntry]) -> Vec<WorkflowOptimization> {
        let mut command_counts = std::collections::HashMap::new();
        let mut optimizations = Vec::new();

        // Count command frequencies
        for command in commands {
            let normalized = self.normalize_command(&command.command);
            *command_counts.entry(normalized).or_insert(0) += 1;
        }

        // Find commands that appear frequently
        for (command, count) in command_counts {
            if count >= self.criteria.min_frequency_for_optimization {
                optimizations.push(WorkflowOptimization {
                    optimization_type: OptimizationType::FrequentCommand,
                    description: format!("Command '{}' appears {} times - consider creating an alias", command, count),
                    original_commands: vec![command.clone()],
                    suggested_replacement: format!("alias frequent_cmd='{}'", command),
                    confidence: 0.8,
                });
            }
        }

        optimizations
    }

    /// Find redundant command sequences
    fn find_redundant_sequences(&self, commands: &[CommandEntry]) -> Vec<WorkflowOptimization> {
        let mut optimizations = Vec::new();
        
        // Look for sequences like: cd dir && ls && cd ..
        for i in 0..commands.len().saturating_sub(2) {
            if let (Some(cmd1), Some(cmd2), Some(cmd3)) = (
                commands.get(i),
                commands.get(i + 1),
                commands.get(i + 2),
            ) {
                if self.is_redundant_cd_sequence(cmd1, cmd2, cmd3) {
                    optimizations.push(WorkflowOptimization {
                        optimization_type: OptimizationType::RedundantSequence,
                        description: "Redundant directory change sequence detected".to_string(),
                        original_commands: vec![
                            cmd1.command.clone(),
                            cmd2.command.clone(),
                            cmd3.command.clone(),
                        ],
                        suggested_replacement: format!("(cd {} && {})",
                            self.extract_cd_target(&cmd1.command).unwrap_or("DIR"),
                            cmd2.command
                        ),
                        confidence: 0.9,
                    });
                }
            }
        }

        optimizations
    }

    /// Check if three commands form a redundant cd sequence
    fn is_redundant_cd_sequence(&self, cmd1: &CommandEntry, cmd2: &CommandEntry, cmd3: &CommandEntry) -> bool {
        cmd1.command.starts_with("cd ") &&
        !cmd2.command.starts_with("cd ") &&
        (cmd3.command == "cd .." || cmd3.command == "cd -")
    }

    /// Extract target directory from cd command
    fn extract_cd_target<'a>(&self, command: &'a str) -> Option<&'a str> {
        if command.starts_with("cd ") {
            command.strip_prefix("cd ").map(|s| s.trim())
        } else {
            None
        }
    }

    /// Find directory change optimizations
    fn find_directory_optimizations(&self, commands: &[CommandEntry]) -> Vec<WorkflowOptimization> {
        let mut optimizations = Vec::new();
        let mut directory_changes = Vec::new();

        // Collect all directory changes
        for command in commands {
            if command.command.starts_with("cd ") {
                directory_changes.push(command);
            }
        }

        // Look for back-and-forth directory changes
        for i in 0..directory_changes.len().saturating_sub(1) {
            if let (Some(cd1), Some(cd2)) = (
                directory_changes.get(i),
                directory_changes.get(i + 1),
            ) {
                if self.is_back_and_forth_cd(cd1, cd2) {
                    optimizations.push(WorkflowOptimization {
                        optimization_type: OptimizationType::DirectoryOptimization,
                        description: "Back-and-forth directory changes detected".to_string(),
                        original_commands: vec![cd1.command.clone(), cd2.command.clone()],
                        suggested_replacement: "Consider staying in one directory or using absolute paths".to_string(),
                        confidence: 0.7,
                    });
                }
            }
        }

        optimizations
    }

    /// Check if two cd commands are back-and-forth
    fn is_back_and_forth_cd(&self, cd1: &CommandEntry, cd2: &CommandEntry) -> bool {
        (cd2.command == "cd .." || cd2.command == "cd -") ||
        (cd1.command == "cd .." && cd2.command.starts_with("cd ") && cd2.command != "cd ..")
    }

    /// Apply deduplication and optimization to a command list
    pub fn process_commands(&self, commands: &[CommandEntry]) -> ProcessedCommands {
        // First apply basic filtering
        let filtered = self.get_filtered_commands(commands);
        
        // Then deduplicate
        let deduplicated = self.deduplicate_commands(&filtered);
        
        // Find optimizations
        let optimizations = self.optimize_workflow(&deduplicated);
        
        ProcessedCommands {
            original_count: commands.len(),
            filtered_commands: deduplicated,
            optimizations,
            stats: self.get_filtering_stats(commands),
        }
    }

    /// Validate command sequences and dependencies
    pub fn validate_command_sequences(&self, commands: &[CommandEntry]) -> Vec<SequenceValidationError> {
        if !self.criteria.enable_sequence_validation {
            return Vec::new();
        }

        let mut validation_errors = Vec::new();
        let dependencies = self.get_command_dependencies();

        for (i, command) in commands.iter().enumerate() {
            // Check for missing dependencies
            if let Some(dep_errors) = self.validate_command_dependencies(command, &dependencies) {
                validation_errors.extend(dep_errors);
            }

            // Check sequence validity with previous commands
            if let Some(seq_errors) = self.validate_command_sequence(command, &commands[..i]) {
                validation_errors.extend(seq_errors);
            }
        }

        validation_errors
    }

    /// Get known command dependencies
    fn get_command_dependencies(&self) -> Vec<CommandDependency> {
        vec![
            // Git commands
            CommandDependency {
                command_pattern: "git".to_string(),
                required_files: vec![".git".to_string()],
                required_commands: vec!["git".to_string()],
                required_environment: vec![],
                description: "Git commands require a git repository".to_string(),
            },
            // Make commands
            CommandDependency {
                command_pattern: "make".to_string(),
                required_files: vec!["Makefile".to_string(), "makefile".to_string()],
                required_commands: vec!["make".to_string()],
                required_environment: vec![],
                description: "Make commands require a Makefile".to_string(),
            },
            // NPM commands
            CommandDependency {
                command_pattern: "npm".to_string(),
                required_files: vec!["package.json".to_string()],
                required_commands: vec!["npm".to_string(), "node".to_string()],
                required_environment: vec![],
                description: "NPM commands require package.json and Node.js".to_string(),
            },
            // Yarn commands
            CommandDependency {
                command_pattern: "yarn".to_string(),
                required_files: vec!["package.json".to_string()],
                required_commands: vec!["yarn".to_string(), "node".to_string()],
                required_environment: vec![],
                description: "Yarn commands require package.json and Node.js".to_string(),
            },
            // Cargo commands
            CommandDependency {
                command_pattern: "cargo".to_string(),
                required_files: vec!["Cargo.toml".to_string()],
                required_commands: vec!["cargo".to_string(), "rustc".to_string()],
                required_environment: vec![],
                description: "Cargo commands require Cargo.toml and Rust".to_string(),
            },
            // Docker commands
            CommandDependency {
                command_pattern: "docker".to_string(),
                required_files: vec![],
                required_commands: vec!["docker".to_string()],
                required_environment: vec!["DOCKER_HOST".to_string()],
                description: "Docker commands require Docker daemon".to_string(),
            },
            // Kubectl commands
            CommandDependency {
                command_pattern: "kubectl".to_string(),
                required_files: vec![],
                required_commands: vec!["kubectl".to_string()],
                required_environment: vec!["KUBECONFIG".to_string()],
                description: "Kubectl commands require Kubernetes configuration".to_string(),
            },
            // Python pip commands
            CommandDependency {
                command_pattern: "pip".to_string(),
                required_files: vec![],
                required_commands: vec!["pip".to_string(), "python".to_string()],
                required_environment: vec![],
                description: "Pip commands require Python".to_string(),
            },
            // Conda commands
            CommandDependency {
                command_pattern: "conda".to_string(),
                required_files: vec![],
                required_commands: vec!["conda".to_string()],
                required_environment: vec!["CONDA_DEFAULT_ENV".to_string()],
                description: "Conda commands require Anaconda/Miniconda".to_string(),
            },
            // Terraform commands
            CommandDependency {
                command_pattern: "terraform".to_string(),
                required_files: vec!["main.tf".to_string(), "*.tf".to_string()],
                required_commands: vec!["terraform".to_string()],
                required_environment: vec![],
                description: "Terraform commands require .tf files".to_string(),
            },
            // Ansible commands
            CommandDependency {
                command_pattern: "ansible".to_string(),
                required_files: vec!["ansible.cfg".to_string(), "inventory".to_string()],
                required_commands: vec!["ansible".to_string()],
                required_environment: vec![],
                description: "Ansible commands require configuration and inventory".to_string(),
            },
            // Maven commands
            CommandDependency {
                command_pattern: "mvn".to_string(),
                required_files: vec!["pom.xml".to_string()],
                required_commands: vec!["mvn".to_string(), "java".to_string()],
                required_environment: vec!["JAVA_HOME".to_string()],
                description: "Maven commands require pom.xml and Java".to_string(),
            },
            // Gradle commands
            CommandDependency {
                command_pattern: "gradle".to_string(),
                required_files: vec!["build.gradle".to_string(), "build.gradle.kts".to_string()],
                required_commands: vec!["gradle".to_string(), "java".to_string()],
                required_environment: vec!["JAVA_HOME".to_string()],
                description: "Gradle commands require build.gradle and Java".to_string(),
            },
        ]
    }

    /// Validate dependencies for a single command
    fn validate_command_dependencies(&self, command: &CommandEntry, dependencies: &[CommandDependency]) -> Option<Vec<SequenceValidationError>> {
        if !self.criteria.validate_dependencies {
            return None;
        }

        let mut errors = Vec::new();
        let cmd_parts: Vec<&str> = command.command.split_whitespace().collect();
        let main_command = cmd_parts.first()?;

        for dep in dependencies {
            if main_command.contains(&dep.command_pattern) || command.command.starts_with(&dep.command_pattern) {
                // Check required files
                for required_file in &dep.required_files {
                    if !self.file_exists_in_context(required_file, &command.working_directory) {
                        errors.push(SequenceValidationError {
                            command: command.command.clone(),
                            error_type: ValidationErrorType::MissingFile,
                            description: format!("Missing required file: {}", required_file),
                            suggested_fix: Some(format!("Create {} or run command in correct directory", required_file)),
                            confidence: 0.8,
                        });
                    }
                }

                // Check required commands
                for required_cmd in &dep.required_commands {
                    if !self.command_available(required_cmd) {
                        errors.push(SequenceValidationError {
                            command: command.command.clone(),
                            error_type: ValidationErrorType::MissingCommand,
                            description: format!("Missing required command: {}", required_cmd),
                            suggested_fix: Some(format!("Install {} or add it to PATH", required_cmd)),
                            confidence: 0.9,
                        });
                    }
                }

                // Check required environment variables
                for env_var in &dep.required_environment {
                    if !self.environment_variable_set(env_var) {
                        errors.push(SequenceValidationError {
                            command: command.command.clone(),
                            error_type: ValidationErrorType::MissingEnvironment,
                            description: format!("Missing environment variable: {}", env_var),
                            suggested_fix: Some(format!("Set {} environment variable", env_var)),
                            confidence: 0.7,
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    /// Validate command sequence logic
    fn validate_command_sequence(&self, command: &CommandEntry, previous_commands: &[CommandEntry]) -> Option<Vec<SequenceValidationError>> {
        let mut errors = Vec::new();

        // Check for common sequence violations
        if let Some(error) = self.check_git_sequence_violations(command, previous_commands) {
            errors.push(error);
        }

        if let Some(error) = self.check_build_sequence_violations(command, previous_commands) {
            errors.push(error);
        }

        if let Some(error) = self.check_deployment_sequence_violations(command, previous_commands) {
            errors.push(error);
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    /// Check for Git workflow sequence violations
    fn check_git_sequence_violations(&self, command: &CommandEntry, previous_commands: &[CommandEntry]) -> Option<SequenceValidationError> {
        let cmd = &command.command;

        // Check git push without commit
        if cmd.starts_with("git push") {
            let has_recent_commit = previous_commands.iter().rev().take(10)
                .any(|prev| prev.command.starts_with("git commit"));
            
            if !has_recent_commit {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "git push without recent commit".to_string(),
                    suggested_fix: Some("Run 'git commit' before pushing".to_string()),
                    confidence: 0.8,
                });
            }
        }

        // Check git commit without add
        if cmd.starts_with("git commit") && !cmd.contains("-a") && !cmd.contains("--all") {
            let has_recent_add = previous_commands.iter().rev().take(5)
                .any(|prev| prev.command.starts_with("git add"));
            
            if !has_recent_add {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "git commit without staging files".to_string(),
                    suggested_fix: Some("Run 'git add' to stage files before committing".to_string()),
                    confidence: 0.7,
                });
            }
        }

        None
    }

    /// Check for build sequence violations
    fn check_build_sequence_violations(&self, command: &CommandEntry, previous_commands: &[CommandEntry]) -> Option<SequenceValidationError> {
        let cmd = &command.command;

        // Check make install without make
        if cmd.starts_with("make install") {
            let has_recent_make = previous_commands.iter().rev().take(5)
                .any(|prev| prev.command == "make" || prev.command.starts_with("make "));
            
            if !has_recent_make {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "make install without building first".to_string(),
                    suggested_fix: Some("Run 'make' before 'make install'".to_string()),
                    confidence: 0.9,
                });
            }
        }

        // Check npm start without install
        if cmd.starts_with("npm start") || cmd.starts_with("npm run") {
            let has_install = previous_commands.iter()
                .any(|prev| prev.command.starts_with("npm install") ||
                           prev.command.starts_with("npm i ") ||
                           prev.command == "npm i" ||
                           prev.command.starts_with("npm ci"));
            
            if !has_install {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "npm start/run without installing dependencies".to_string(),
                    suggested_fix: Some("Run 'npm install' first".to_string()),
                    confidence: 0.8,
                });
            }
        }

        None
    }

    /// Check for deployment sequence violations
    fn check_deployment_sequence_violations(&self, command: &CommandEntry, previous_commands: &[CommandEntry]) -> Option<SequenceValidationError> {
        let cmd = &command.command;

        // Check docker run without build
        if cmd.starts_with("docker run") && cmd.contains("localhost") {
            let has_recent_build = previous_commands.iter().rev().take(10)
                .any(|prev| prev.command.starts_with("docker build"));
            
            if !has_recent_build {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "docker run with local image without recent build".to_string(),
                    suggested_fix: Some("Run 'docker build' before running local image".to_string()),
                    confidence: 0.7,
                });
            }
        }

        // Check kubectl apply without context
        if cmd.starts_with("kubectl apply") {
            let has_context_set = previous_commands.iter().rev().take(5)
                .any(|prev| prev.command.contains("kubectl config"));
            
            if !has_context_set {
                return Some(SequenceValidationError {
                    command: command.command.clone(),
                    error_type: ValidationErrorType::BrokenSequence,
                    description: "kubectl apply without setting context".to_string(),
                    suggested_fix: Some("Set kubectl context with 'kubectl config use-context'".to_string()),
                    confidence: 0.6,
                });
            }
        }

        None
    }

    /// Check if a file exists in the given context (simplified check)
    fn file_exists_in_context(&self, _filename: &str, _working_dir: &str) -> bool {
        // In a real implementation, this would check the file system
        // For now, we'll assume files exist to avoid false positives
        true
    }

    /// Check if a command is available in PATH (simplified check)
    fn command_available(&self, _command: &str) -> bool {
        // In a real implementation, this would check PATH
        // For now, we'll assume commands are available to avoid false positives
        true
    }

    /// Check if an environment variable is set (simplified check)
    fn environment_variable_set(&self, _var_name: &str) -> bool {
        // In a real implementation, this would check environment variables
        // For now, we'll assume they're set to avoid false positives
        true
    }

    /// Generate suggestions for fixing broken command sequences
    pub fn suggest_sequence_fixes(&self, commands: &[CommandEntry]) -> Vec<WorkflowOptimization> {
        if !self.criteria.suggest_fixes {
            return Vec::new();
        }

        let validation_errors = self.validate_command_sequences(commands);
        let mut suggestions = Vec::new();

        for error in validation_errors {
            if let Some(fix) = error.suggested_fix {
                suggestions.push(WorkflowOptimization {
                    optimization_type: OptimizationType::SequenceValidation,
                    description: format!("Sequence validation: {}", error.description),
                    original_commands: vec![error.command],
                    suggested_replacement: fix,
                    confidence: error.confidence,
                });
            }
        }

        suggestions
    }

    /// Process commands with full validation, filtering, and optimization
    pub fn process_commands_with_validation(&self, commands: &[CommandEntry]) -> ProcessedCommands {
        // First apply basic filtering
        let filtered = self.get_filtered_commands(commands);
        
        // Apply privacy filtering to sanitize sensitive data
        let sanitized: Vec<CommandEntry> = if self.criteria.enable_privacy_filtering {
            filtered.iter().map(|cmd| self.sanitize_command(cmd)).collect()
        } else {
            filtered
        };
        
        // Then deduplicate
        let deduplicated = self.deduplicate_commands(&sanitized);
        
        // Find optimizations
        let mut optimizations = self.optimize_workflow(&deduplicated);
        
        // Add validation suggestions
        let validation_suggestions = self.suggest_sequence_fixes(&deduplicated);
        optimizations.extend(validation_suggestions);
        
        // Get enhanced stats with validation
        let mut stats = self.get_filtering_stats(commands);
        let validation_errors = self.validate_command_sequences(commands);
        stats.validation_errors = validation_errors.len();
        stats.missing_dependencies = validation_errors.iter()
            .filter(|e| matches!(e.error_type, ValidationErrorType::MissingFile | ValidationErrorType::MissingCommand | ValidationErrorType::MissingEnvironment))
            .count();
        stats.broken_sequences = validation_errors.iter()
            .filter(|e| matches!(e.error_type, ValidationErrorType::BrokenSequence | ValidationErrorType::InvalidPrerequisite))
            .count();
        
        ProcessedCommands {
            original_count: commands.len(),
            filtered_commands: deduplicated,
            optimizations,
            stats,
        }
    }
}

/// Types of workflow optimizations
#[derive(Debug, Clone)]
pub enum OptimizationType {
    FrequentCommand,
    RedundantSequence,
    DirectoryOptimization,
    CommandConsolidation,
    DependencyValidation,
    SequenceValidation,
}

/// Workflow optimization suggestion
#[derive(Debug, Clone)]
pub struct WorkflowOptimization {
    pub optimization_type: OptimizationType,
    pub description: String,
    pub original_commands: Vec<String>,
    pub suggested_replacement: String,
    pub confidence: f32,
}

/// Result of processing commands with filtering, deduplication, and optimization
#[derive(Debug, Clone)]
pub struct ProcessedCommands {
    pub original_count: usize,
    pub filtered_commands: Vec<CommandEntry>,
    pub optimizations: Vec<WorkflowOptimization>,
    pub stats: FilteringStats,
}

/// Statistics about command filtering results
#[derive(Debug, Default, Clone)]
pub struct FilteringStats {
    pub total_commands: usize,
    pub included_commands: usize,
    pub excluded_commands: usize,
    pub failed_commands: usize,
    pub typo_commands: usize,
    pub suspicious_commands: usize,
    pub error_output_commands: usize,
    pub privacy_filtered_commands: usize,
    pub validation_errors: usize,
    pub missing_dependencies: usize,
    pub broken_sequences: usize,
}

impl FilteringStats {
    /// Get the percentage of commands that were included
    pub fn inclusion_rate(&self) -> f32 {
        if self.total_commands == 0 {
            0.0
        } else {
            (self.included_commands as f32 / self.total_commands as f32) * 100.0
        }
    }

    /// Get the percentage of commands that were excluded
    pub fn exclusion_rate(&self) -> f32 {
        if self.total_commands == 0 {
            0.0
        } else {
            (self.excluded_commands as f32 / self.total_commands as f32) * 100.0
        }
    }
}

impl Default for CommandFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_command(command: &str, exit_code: Option<i32>) -> CommandEntry {
        CommandEntry {
            command: command.to_string(),
            timestamp: Utc::now(),
            exit_code,
            working_directory: "/test".to_string(),
            shell: "bash".to_string(),
            output: None,
            error: None,
        }
    }

    #[test]
    fn test_successful_command_filtering() {
        let filter = CommandFilter::new();
        let cmd = create_test_command("ls -la", Some(0));
        
        let result = filter.filter_command(&cmd);
        assert!(result.should_include);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_failed_command_filtering() {
        let filter = CommandFilter::new();
        let cmd = create_test_command("ls /nonexistent", Some(1));
        
        let result = filter.filter_command(&cmd);
        assert!(!result.should_include);
        assert!(result.reason.contains("failed with exit code 1"));
    }

    #[test]
    fn test_typo_filtering() {
        let filter = CommandFilter::new();
        let cmd = create_test_command("sl", None);
        
        let result = filter.filter_command(&cmd);
        assert!(!result.should_include);
        assert!(result.reason.contains("exclusion pattern"));
    }

    #[test]
    fn test_suspicious_command_detection() {
        let filter = CommandFilter::new();
        
        // Single character commands (except allowed ones)
        let cmd1 = create_test_command("x", None);
        let result1 = filter.filter_command(&cmd1);
        assert!(!result1.should_include);
        
        // Repeated characters
        let cmd2 = create_test_command("lllls", None);
        let result2 = filter.filter_command(&cmd2);
        assert!(!result2.should_include);
    }

    #[test]
    fn test_failure_indicator_detection() {
        let filter = CommandFilter::new();
        let mut cmd = create_test_command("some_command", None);
        cmd.error = Some("Error: file not found".to_string());
        
        let result = filter.filter_command(&cmd);
        assert!(!result.should_include);
        assert!(result.reason.contains("failure indicators"));
    }

    #[test]
    fn test_safe_command_detection() {
        let filter = CommandFilter::new();
        
        assert!(filter.is_safe_to_test("ls -la"));
        assert!(filter.is_safe_to_test("pwd"));
        assert!(!filter.is_safe_to_test("rm -rf /"));
        assert!(!filter.is_safe_to_test("sudo something"));
    }

    #[test]
    fn test_custom_criteria() {
        let mut criteria = FilterCriteria::default();
        criteria.only_successful = true;
        
        let filter = CommandFilter::with_criteria(criteria);
        let cmd = create_test_command("test_command", Some(2));
        
        let result = filter.filter_command(&cmd);
        assert!(!result.should_include);
        assert!(result.reason.contains("Only successful commands"));
    }
}

// Include the test module
#[cfg(test)]
#[path = "command.test.rs"]
mod command_test;