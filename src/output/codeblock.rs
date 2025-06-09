use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::terminal::CommandEntry;

/// Configuration for code block generation and syntax highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockConfig {
    /// Enable intelligent language detection
    pub enable_language_detection: bool,
    /// Default language for commands when detection fails
    pub default_command_language: String,
    /// Default language for output blocks
    pub default_output_language: String,
    /// Default language for error blocks
    pub default_error_language: String,
    /// Enable line numbers in code blocks
    pub enable_line_numbers: bool,
    /// Enable syntax highlighting hints
    pub enable_syntax_hints: bool,
    /// Custom language mappings for specific commands
    pub custom_language_mappings: HashMap<String, String>,
    /// Enable code block titles
    pub enable_block_titles: bool,
    /// Enable collapsible code blocks for long output
    pub enable_collapsible_blocks: bool,
    /// Threshold for making blocks collapsible (in lines)
    pub collapsible_threshold: usize,
}

impl Default for CodeBlockConfig {
    fn default() -> Self {
        let mut custom_mappings = HashMap::new();
        
        // Add common command-to-language mappings
        custom_mappings.insert("python".to_string(), "python".to_string());
        custom_mappings.insert("python3".to_string(), "python".to_string());
        custom_mappings.insert("node".to_string(), "javascript".to_string());
        custom_mappings.insert("npm".to_string(), "bash".to_string());
        custom_mappings.insert("yarn".to_string(), "bash".to_string());
        custom_mappings.insert("cargo".to_string(), "bash".to_string());
        custom_mappings.insert("rustc".to_string(), "bash".to_string());
        custom_mappings.insert("go".to_string(), "bash".to_string());
        custom_mappings.insert("java".to_string(), "bash".to_string());
        custom_mappings.insert("javac".to_string(), "bash".to_string());
        custom_mappings.insert("gcc".to_string(), "bash".to_string());
        custom_mappings.insert("clang".to_string(), "bash".to_string());
        custom_mappings.insert("make".to_string(), "makefile".to_string());
        custom_mappings.insert("cmake".to_string(), "bash".to_string());
        custom_mappings.insert("docker".to_string(), "dockerfile".to_string());
        custom_mappings.insert("kubectl".to_string(), "yaml".to_string());
        custom_mappings.insert("helm".to_string(), "yaml".to_string());
        custom_mappings.insert("terraform".to_string(), "hcl".to_string());
        custom_mappings.insert("ansible".to_string(), "yaml".to_string());
        custom_mappings.insert("ansible-playbook".to_string(), "yaml".to_string());
        
        Self {
            enable_language_detection: true,
            default_command_language: "bash".to_string(),
            default_output_language: "text".to_string(),
            default_error_language: "text".to_string(),
            enable_line_numbers: false,
            enable_syntax_hints: true,
            custom_language_mappings: custom_mappings,
            enable_block_titles: true,
            enable_collapsible_blocks: true,
            collapsible_threshold: 20,
        }
    }
}

/// Types of code blocks that can be generated
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockType {
    Command,
    Output,
    Error,
    Config,
    Script,
    Log,
}

/// Represents a formatted code block with metadata
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub content: String,
    pub language: String,
    pub block_type: CodeBlockType,
    pub title: Option<String>,
    pub line_count: usize,
    pub is_collapsible: bool,
    pub syntax_hints: Vec<String>,
}

/// Advanced code block generator with intelligent syntax highlighting
pub struct CodeBlockGenerator {
    config: CodeBlockConfig,
    language_patterns: HashMap<String, Regex>,
}

impl CodeBlockGenerator {
    /// Create a new code block generator with default configuration
    pub fn new() -> Self {
        Self::with_config(CodeBlockConfig::default())
    }

    /// Create a new code block generator with custom configuration
    pub fn with_config(config: CodeBlockConfig) -> Self {
        let mut generator = Self {
            config,
            language_patterns: HashMap::new(),
        };
        generator.initialize_patterns();
        generator
    }

    /// Initialize regex patterns for language detection
    fn initialize_patterns(&mut self) {
        // Command patterns for language detection
        let patterns = vec![
            ("python", r"^(python|python3|pip|pip3)\s"),
            ("javascript", r"^(node|npm|yarn|npx)\s"),
            ("rust", r"^(cargo|rustc|rustup)\s"),
            ("go", r"^(go)\s+(run|build|test|mod|get)"),
            ("java", r"^(java|javac|mvn|gradle)\s"),
            ("docker", r"^(docker|docker-compose)\s"),
            ("kubernetes", r"^(kubectl|helm|k9s)\s"),
            ("git", r"^(git)\s"),
            ("sql", r"^(mysql|psql|sqlite3|sqlcmd)\s"),
            ("shell", r"^(bash|zsh|fish|sh)\s"),
            ("makefile", r"^(make)\s"),
            ("terraform", r"^(terraform|tf)\s"),
            ("ansible", r"^(ansible|ansible-playbook)\s"),
            ("vim", r"^(vim|nvim|vi)\s"),
            ("emacs", r"^(emacs)\s"),
            ("curl", r"^(curl|wget)\s"),
            ("ssh", r"^(ssh|scp|rsync)\s"),
            ("systemctl", r"^(systemctl|service)\s"),
            ("package_manager", r"^(apt|yum|dnf|pacman|brew|choco)\s"),
        ];

        for (lang, pattern) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.language_patterns.insert(lang.to_string(), regex);
            }
        }
    }

    /// Generate a code block for a command
    pub fn generate_command_block(&self, command: &CommandEntry) -> CodeBlock {
        let language = self.detect_command_language(&command.command);
        let line_count = command.command.lines().count();
        let is_collapsible = self.config.enable_collapsible_blocks && 
                           line_count > self.config.collapsible_threshold;
        
        let title = if self.config.enable_block_titles {
            Some(format!("Command ({})", command.shell))
        } else {
            None
        };

        let syntax_hints = if self.config.enable_syntax_hints {
            self.generate_command_syntax_hints(&command.command, &language)
        } else {
            Vec::new()
        };

        CodeBlock {
            content: command.command.clone(),
            language,
            block_type: CodeBlockType::Command,
            title,
            line_count,
            is_collapsible,
            syntax_hints,
        }
    }

    /// Generate a code block for command output
    pub fn generate_output_block(&self, output: &str, command: &str) -> CodeBlock {
        let language = self.detect_output_language(output, command);
        let line_count = output.lines().count();
        let is_collapsible = self.config.enable_collapsible_blocks && 
                           line_count > self.config.collapsible_threshold;
        
        let title = if self.config.enable_block_titles {
            Some("Output".to_string())
        } else {
            None
        };

        let syntax_hints = if self.config.enable_syntax_hints {
            self.generate_output_syntax_hints(output, command)
        } else {
            Vec::new()
        };

        CodeBlock {
            content: output.to_string(),
            language,
            block_type: CodeBlockType::Output,
            title,
            line_count,
            is_collapsible,
            syntax_hints,
        }
    }

    /// Generate a code block for error output
    pub fn generate_error_block(&self, error: &str, command: &str) -> CodeBlock {
        let language = self.detect_error_language(error, command);
        let line_count = error.lines().count();
        let is_collapsible = self.config.enable_collapsible_blocks && 
                           line_count > self.config.collapsible_threshold;
        
        let title = if self.config.enable_block_titles {
            Some("Error".to_string())
        } else {
            None
        };

        let syntax_hints = if self.config.enable_syntax_hints {
            self.generate_error_syntax_hints(error, command)
        } else {
            Vec::new()
        };

        CodeBlock {
            content: error.to_string(),
            language,
            block_type: CodeBlockType::Error,
            title,
            line_count,
            is_collapsible,
            syntax_hints,
        }
    }

    /// Detect the appropriate language for a command
    fn detect_command_language(&self, command: &str) -> String {
        if !self.config.enable_language_detection {
            return self.config.default_command_language.clone();
        }

        // Check custom mappings first
        let first_word = command.split_whitespace().next().unwrap_or("");
        if let Some(language) = self.config.custom_language_mappings.get(first_word) {
            return language.clone();
        }

        // Check regex patterns
        for (language, pattern) in &self.language_patterns {
            if pattern.is_match(command) {
                return self.map_detected_language_to_highlight(language);
            }
        }

        // Special cases for complex commands
        if command.contains("#!/bin/bash") || command.contains("#!/bin/sh") {
            return "bash".to_string();
        }
        
        if command.contains("#!/usr/bin/env python") {
            return "python".to_string();
        }

        if command.contains("#!/usr/bin/env node") {
            return "javascript".to_string();
        }

        // Check for file extensions in the command
        if let Some(extension) = self.extract_file_extension(command) {
            return self.map_extension_to_language(&extension);
        }

        self.config.default_command_language.clone()
    }

    /// Detect the appropriate language for output
    fn detect_output_language(&self, output: &str, command: &str) -> String {
        if !self.config.enable_language_detection {
            return self.config.default_output_language.clone();
        }

        // JSON output detection
        if output.trim_start().starts_with('{') && output.trim_end().ends_with('}') {
            return "json".to_string();
        }
        
        if output.trim_start().starts_with('[') && output.trim_end().ends_with(']') {
            return "json".to_string();
        }

        // YAML output detection
        if output.contains("---") && (output.contains(": ") || output.contains("- ")) {
            return "yaml".to_string();
        }

        // XML output detection
        if output.trim_start().starts_with("<?xml") || 
           (output.trim_start().starts_with('<') && output.trim_end().ends_with('>')) {
            return "xml".to_string();
        }

        // CSV output detection
        if output.lines().take(5).all(|line| line.contains(',')) {
            return "csv".to_string();
        }

        // Log format detection
        if self.looks_like_log_output(output) {
            return "log".to_string();
        }

        // SQL output detection
        if command.contains("sql") && output.contains("|") && output.contains("+") {
            return "text".to_string(); // SQL table output
        }

        // Docker output detection
        if command.starts_with("docker") && output.contains("CONTAINER ID") {
            return "text".to_string();
        }

        // Git output detection
        if command.starts_with("git") {
            if output.contains("commit ") || output.contains("Author:") {
                return "diff".to_string();
            }
        }

        self.config.default_output_language.clone()
    }

    /// Detect the appropriate language for error output
    fn detect_error_language(&self, error: &str, command: &str) -> String {
        if !self.config.enable_language_detection {
            return self.config.default_error_language.clone();
        }

        // Compiler error detection
        if command.contains("rustc") || command.contains("cargo") {
            return "rust".to_string();
        }
        
        if command.contains("gcc") || command.contains("clang") {
            return "c".to_string();
        }
        
        if command.contains("javac") || command.contains("java") {
            return "java".to_string();
        }
        
        if command.contains("python") && error.contains("Traceback") {
            return "python".to_string();
        }
        
        if command.contains("node") && error.contains("Error:") {
            return "javascript".to_string();
        }

        // Stack trace detection
        if error.contains("at ") && error.contains("(") && error.contains(")") {
            return "stacktrace".to_string();
        }

        self.config.default_error_language.clone()
    }

    /// Generate syntax hints for commands
    fn generate_command_syntax_hints(&self, command: &str, language: &str) -> Vec<String> {
        let mut hints = Vec::new();

        // Add language-specific hints
        match language {
            "python" => {
                if command.contains("pip install") {
                    hints.push("Package installation".to_string());
                }
                if command.contains("python -m") {
                    hints.push("Module execution".to_string());
                }
            }
            "rust" => {
                if command.contains("cargo build") {
                    hints.push("Compilation".to_string());
                }
                if command.contains("cargo test") {
                    hints.push("Testing".to_string());
                }
            }
            "docker" => {
                if command.contains("docker run") {
                    hints.push("Container execution".to_string());
                }
                if command.contains("docker build") {
                    hints.push("Image building".to_string());
                }
            }
            "git" => {
                if command.contains("git commit") {
                    hints.push("Version control".to_string());
                }
                if command.contains("git push") {
                    hints.push("Remote synchronization".to_string());
                }
            }
            _ => {}
        }

        // Add general command hints
        if command.contains("sudo") {
            hints.push("Elevated privileges".to_string());
        }
        
        if command.contains("&&") || command.contains("||") {
            hints.push("Command chaining".to_string());
        }
        
        if command.contains("|") {
            hints.push("Pipe operation".to_string());
        }
        
        if command.contains(">") || command.contains(">>") {
            hints.push("Output redirection".to_string());
        }

        hints
    }

    /// Generate syntax hints for output
    fn generate_output_syntax_hints(&self, output: &str, command: &str) -> Vec<String> {
        let mut hints = Vec::new();

        if output.lines().count() > 50 {
            hints.push("Large output".to_string());
        }

        if command.contains("test") && output.contains("passed") {
            hints.push("Test results".to_string());
        }

        if command.contains("build") && output.contains("Finished") {
            hints.push("Build output".to_string());
        }

        if output.contains("WARNING") || output.contains("WARN") {
            hints.push("Contains warnings".to_string());
        }

        hints
    }

    /// Generate syntax hints for errors
    fn generate_error_syntax_hints(&self, error: &str, command: &str) -> Vec<String> {
        let mut hints = Vec::new();

        if error.contains("permission denied") {
            hints.push("Permission issue".to_string());
        }

        if error.contains("not found") || error.contains("No such file") {
            hints.push("Missing resource".to_string());
        }

        if error.contains("syntax error") {
            hints.push("Syntax issue".to_string());
        }

        if error.contains("timeout") {
            hints.push("Timeout error".to_string());
        }

        if command.contains("compile") && error.contains("error:") {
            hints.push("Compilation error".to_string());
        }

        hints
    }

    /// Format a code block as markdown
    pub fn format_code_block(&self, block: &CodeBlock) -> String {
        let mut result = String::new();

        // Add title if enabled
        if let Some(title) = &block.title {
            result.push_str(&format!("**{}**", title));
            if !block.syntax_hints.is_empty() {
                result.push_str(&format!(" *({})*", block.syntax_hints.join(", ")));
            }
            result.push_str("\n\n");
        }

        // Add collapsible wrapper if needed
        if block.is_collapsible {
            result.push_str("<details>\n");
            result.push_str(&format!("<summary>Show {} ({} lines)</summary>\n\n", 
                                   block.block_type.to_string().to_lowercase(), 
                                   block.line_count));
        }

        // Add the code block
        result.push_str(&format!("```{}", block.language));
        
        if self.config.enable_line_numbers && block.line_count > 5 {
            result.push_str(" {.line-numbers}");
        }
        
        result.push('\n');
        result.push_str(&block.content);
        result.push_str("\n```\n");

        // Close collapsible wrapper if needed
        if block.is_collapsible {
            result.push_str("\n</details>\n");
        }

        result
    }

    /// Extract file extension from command
    fn extract_file_extension(&self, command: &str) -> Option<String> {
        let words: Vec<&str> = command.split_whitespace().collect();
        for word in words {
            if let Some(dot_pos) = word.rfind('.') {
                let extension = &word[dot_pos + 1..];
                if extension.len() <= 5 && extension.chars().all(|c| c.is_alphanumeric()) {
                    return Some(extension.to_lowercase());
                }
            }
        }
        None
    }

    /// Map file extension to programming language
    fn map_extension_to_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "rust".to_string(),
            "py" => "python".to_string(),
            "js" | "mjs" => "javascript".to_string(),
            "ts" => "typescript".to_string(),
            "java" => "java".to_string(),
            "c" => "c".to_string(),
            "cpp" | "cc" | "cxx" => "cpp".to_string(),
            "h" | "hpp" => "c".to_string(),
            "go" => "go".to_string(),
            "rb" => "ruby".to_string(),
            "php" => "php".to_string(),
            "sh" | "bash" => "bash".to_string(),
            "zsh" => "zsh".to_string(),
            "fish" => "fish".to_string(),
            "ps1" => "powershell".to_string(),
            "sql" => "sql".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            "xml" => "xml".to_string(),
            "html" => "html".to_string(),
            "css" => "css".to_string(),
            "scss" | "sass" => "scss".to_string(),
            "md" => "markdown".to_string(),
            "toml" => "toml".to_string(),
            "ini" => "ini".to_string(),
            "conf" | "config" => "text".to_string(),
            "log" => "log".to_string(),
            _ => self.config.default_command_language.clone(),
        }
    }

    /// Map detected language patterns to syntax highlighting languages
    fn map_detected_language_to_highlight(&self, detected: &str) -> String {
        match detected {
            "package_manager" => "bash".to_string(),
            "systemctl" => "bash".to_string(),
            "curl" => "bash".to_string(),
            "ssh" => "bash".to_string(),
            "vim" | "emacs" => "bash".to_string(),
            "kubernetes" => "yaml".to_string(),
            _ => detected.to_string(),
        }
    }

    /// Check if output looks like log format
    fn looks_like_log_output(&self, output: &str) -> bool {
        let lines: Vec<&str> = output.lines().take(10).collect();
        if lines.len() < 3 {
            return false;
        }

        let timestamp_patterns = [
            r"\d{4}-\d{2}-\d{2}",  // YYYY-MM-DD
            r"\d{2}:\d{2}:\d{2}",  // HH:MM:SS
            r"\[\d+\]",            // [timestamp]
            r"\w{3}\s+\d{1,2}",    // Mon 12
        ];

        let log_level_patterns = [
            "INFO", "DEBUG", "WARN", "ERROR", "FATAL", "TRACE",
            "info", "debug", "warn", "error", "fatal", "trace",
        ];

        let mut timestamp_matches = 0;
        let mut level_matches = 0;

        for line in &lines {
            for pattern in &timestamp_patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(line) {
                        timestamp_matches += 1;
                        break;
                    }
                }
            }

            for level in &log_level_patterns {
                if line.contains(level) {
                    level_matches += 1;
                    break;
                }
            }
        }

        // Consider it a log if at least 40% of lines have timestamps or log levels
        (timestamp_matches as f64 / lines.len() as f64) >= 0.4 ||
        (level_matches as f64 / lines.len() as f64) >= 0.4
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: CodeBlockConfig) {
        self.config = config;
        self.initialize_patterns();
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &CodeBlockConfig {
        &self.config
    }
}

impl Default for CodeBlockGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CodeBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeBlockType::Command => write!(f, "Command"),
            CodeBlockType::Output => write!(f, "Output"),
            CodeBlockType::Error => write!(f, "Error"),
            CodeBlockType::Config => write!(f, "Config"),
            CodeBlockType::Script => write!(f, "Script"),
            CodeBlockType::Log => write!(f, "Log"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_command(command: &str, shell: &str) -> CommandEntry {
        CommandEntry {
            command: command.to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/test".to_string(),
            shell: shell.to_string(),
            output: None,
            error: None,
        }
    }

    #[test]
    fn test_language_detection() {
        let generator = CodeBlockGenerator::new();
        
        assert_eq!(generator.detect_command_language("python script.py"), "python");
        assert_eq!(generator.detect_command_language("cargo build"), "bash");
        assert_eq!(generator.detect_command_language("docker run nginx"), "dockerfile");
        assert_eq!(generator.detect_command_language("kubectl get pods"), "yaml");
        assert_eq!(generator.detect_command_language("git commit -m 'test'"), "git");
    }

    #[test]
    fn test_output_language_detection() {
        let generator = CodeBlockGenerator::new();
        
        let json_output = r#"{"name": "test", "value": 123}"#;
        assert_eq!(generator.detect_output_language(json_output, "curl"), "json");
        
        let yaml_output = "---\nname: test\nvalue: 123";
        assert_eq!(generator.detect_output_language(yaml_output, "kubectl"), "yaml");
        
        let xml_output = "<?xml version=\"1.0\"?><root></root>";
        assert_eq!(generator.detect_output_language(xml_output, "curl"), "xml");
    }

    #[test]
    fn test_command_block_generation() {
        let generator = CodeBlockGenerator::new();
        let command = create_test_command("python script.py", "bash");
        
        let block = generator.generate_command_block(&command);
        assert_eq!(block.language, "python");
        assert_eq!(block.block_type, CodeBlockType::Command);
        assert_eq!(block.content, "python script.py");
    }

    #[test]
    fn test_output_block_generation() {
        let generator = CodeBlockGenerator::new();
        let output = r#"{"result": "success"}"#;
        
        let block = generator.generate_output_block(output, "curl");
        assert_eq!(block.language, "json");
        assert_eq!(block.block_type, CodeBlockType::Output);
    }

    #[test]
    fn test_error_block_generation() {
        let generator = CodeBlockGenerator::new();
        let error = "Traceback (most recent call last):\n  File \"script.py\", line 1";
        
        let block = generator.generate_error_block(error, "python script.py");
        assert_eq!(block.language, "python");
        assert_eq!(block.block_type, CodeBlockType::Error);
    }

    #[test]
    fn test_syntax_hints() {
        let generator = CodeBlockGenerator::new();
        
        let hints = generator.generate_command_syntax_hints("sudo apt install package", "bash");
        assert!(hints.contains(&"Elevated privileges".to_string()));
        
        let hints = generator.generate_command_syntax_hints("ls | grep test", "bash");
        assert!(hints.contains(&"Pipe operation".to_string()));
    }

    #[test]
    fn test_collapsible_blocks() {
        let mut config = CodeBlockConfig::default();
        config.collapsible_threshold = 5;
        let generator = CodeBlockGenerator::with_config(config);
        
        let long_output = "line1\nline2\nline3\nline4\nline5\nline6\nline7";
        let block = generator.generate_output_block(long_output, "echo");
        assert!(block.is_collapsible);
    }

    #[test]
    fn test_file_extension_detection() {
        let generator = CodeBlockGenerator::new();
        
        assert_eq!(generator.extract_file_extension("vim script.py"), Some("py".to_string()));
        assert_eq!(generator.extract_file_extension("cat config.json"), Some("json".to_string()));
        assert_eq!(generator.extract_file_extension("ls -la"), None);
    }

    #[test]
    fn test_log_detection() {
        let generator = CodeBlockGenerator::new();
        
        let log_output = "2023-01-01 12:00:00 INFO Starting application\n2023-01-01 12:00:01 DEBUG Loading config\n2023-01-01 12:00:02 WARN Configuration issue";
        assert!(generator.looks_like_log_output(log_output));
        
        let normal_output = "Hello world\nThis is normal text";
        assert!(!generator.looks_like_log_output(normal_output));
    }

    #[test]
    fn test_code_block_formatting() {
        let generator = CodeBlockGenerator::new();
        let command = create_test_command("echo 'hello'", "bash");
        
        let block = generator.generate_command_block(&command);
        let formatted = generator.format_code_block(&block);
        
        assert!(formatted.contains("```bash"));
        assert!(formatted.contains("echo 'hello'"));
        assert!(formatted.contains("**Command (bash)**"));
    }
}