use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

use crate::session::manager::{Session, Annotation, AnnotationType};
use crate::terminal::CommandEntry;
use crate::llm::{AIAnalyzer, AnalysisResult, LlmConfig};
use std::cell::RefCell;
use super::codeblock::{CodeBlockGenerator, CodeBlockConfig};

/// Configuration for markdown output generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// Include session metadata in output
    pub include_metadata: bool,
    /// Include command timestamps
    pub include_timestamps: bool,
    /// Include command output
    pub include_output: bool,
    /// Include command errors
    pub include_errors: bool,
    /// Include annotations
    pub include_annotations: bool,
    /// Include session statistics
    pub include_statistics: bool,
    /// Maximum length for command output (0 = no limit)
    pub max_output_length: usize,
    /// Code block language for syntax highlighting
    pub code_language: String,
    /// Custom CSS classes for styling
    pub css_classes: HashMap<String, String>,
    /// Template customization options
    pub template_options: TemplateOptions,
    /// Code block generation configuration
    pub code_block_config: CodeBlockConfig,
    /// AI analysis integration configuration
    pub ai_analysis_config: AIAnalysisConfig,
}

/// Template customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOptions {
    /// Custom title for the document
    pub title: Option<String>,
    /// Include table of contents
    pub include_toc: bool,
    /// Group commands by working directory
    pub group_by_directory: bool,
    /// Group commands by time periods
    pub group_by_time: bool,
    /// Time grouping interval in minutes
    pub time_group_interval: u64,
    /// Include command success/failure indicators
    pub include_status_indicators: bool,
    /// Custom header content
    pub custom_header: Option<String>,
    /// Custom footer content
    pub custom_footer: Option<String>,
    /// Enable hierarchical documentation structure
    pub enable_hierarchical_structure: bool,
    /// Group commands by workflow phases
    pub group_by_workflow: bool,
    /// Group commands by command type/category
    pub group_by_command_type: bool,
    /// Maximum depth for hierarchical nesting
    pub max_hierarchy_depth: usize,
    /// Include workflow summary sections
    pub include_workflow_summaries: bool,
    /// Include command type explanations
    pub include_command_type_explanations: bool,
    
    // New formatting options for task 4.5
    /// Date format for timestamps (e.g., "%Y-%m-%d %H:%M:%S", "%B %d, %Y")
    pub date_format: String,
    /// Output theme/style preset
    pub theme: OutputTheme,
    /// Include command execution duration
    pub include_duration: bool,
    /// Include working directory for each command
    pub include_working_directory: bool,
    /// Include command exit codes
    pub include_exit_codes: bool,
    /// Include environment variables used
    pub include_environment_vars: bool,
    /// Maximum number of commands to include (0 = no limit)
    pub max_commands: usize,
    /// Include command numbering/indexing
    pub include_command_numbers: bool,
    /// Use collapsible sections for long output
    pub use_collapsible_sections: bool,
    /// Include session summary at the top
    pub include_session_summary: bool,
    /// Include command frequency statistics
    pub include_command_stats: bool,
    /// Custom section ordering
    pub section_order: Vec<DocumentSection>,
    /// Include breadcrumb navigation
    pub include_breadcrumbs: bool,
    /// Use emoji indicators for command types
    pub use_emoji_indicators: bool,
    /// Include performance metrics
    pub include_performance_metrics: bool,
    /// Custom markdown extensions to enable
    pub markdown_extensions: Vec<MarkdownExtension>,
    /// Output verbosity level
    pub verbosity_level: VerbosityLevel,
    /// Include command dependencies/relationships
    pub include_command_relationships: bool,
    /// Use compact formatting for space efficiency
    pub use_compact_formatting: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 1000,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions::default(),
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            title: None,
            include_toc: false,
            group_by_directory: false,
            group_by_time: false,
            time_group_interval: 30,
            include_status_indicators: true,
            custom_header: None,
            custom_footer: None,
            enable_hierarchical_structure: false,
            group_by_workflow: false,
            group_by_command_type: false,
            max_hierarchy_depth: 3,
            include_workflow_summaries: true,
            include_command_type_explanations: true,
            
            // New formatting options defaults
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            theme: OutputTheme::default(),
            include_duration: true,
            include_working_directory: false,
            include_exit_codes: true,
            include_environment_vars: false,
            max_commands: 0, // No limit
            include_command_numbers: false,
            use_collapsible_sections: false,
            include_session_summary: true,
            include_command_stats: false,
            section_order: vec![
                DocumentSection::SessionInfo,
                DocumentSection::TableOfContents,
                DocumentSection::Commands,
                DocumentSection::Statistics,
                DocumentSection::Analysis,
                DocumentSection::Annotations,
                DocumentSection::Performance,
                DocumentSection::Footer,
            ],
            include_breadcrumbs: false,
            use_emoji_indicators: false,
            include_performance_metrics: false,
            markdown_extensions: vec![
                MarkdownExtension::Tables,
                MarkdownExtension::SyntaxHighlighting,
            ],
            verbosity_level: VerbosityLevel::default(),
            include_command_relationships: false,
            use_compact_formatting: false,
        }
    }
}

/// Configuration for AI analysis integration in markdown output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisConfig {
    /// Enable AI-generated explanations for commands
    pub enable_ai_explanations: bool,
    /// Include detailed analysis in output
    pub include_detailed_analysis: bool,
    /// Include security analysis and warnings
    pub include_security_analysis: bool,
    /// Include alternative command suggestions
    pub include_alternatives: bool,
    /// Include context insights
    pub include_context_insights: bool,
    /// Include AI recommendations
    pub include_recommendations: bool,
    /// Maximum number of alternatives to show
    pub max_alternatives: usize,
    /// Maximum number of recommendations to show
    pub max_recommendations: usize,
    /// Minimum confidence score to include analysis (0.0-1.0)
    pub min_confidence_score: f32,
    /// Enable analysis caching for performance
    pub enable_caching: bool,
    /// Custom analysis prompt context
    pub custom_context: Option<String>,
}

impl Default for AIAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_ai_explanations: false, // Disabled by default to avoid API costs
            include_detailed_analysis: true,
            include_security_analysis: true,
            include_alternatives: true,
            include_context_insights: false, // Can be verbose
            include_recommendations: true,
            max_alternatives: 3,
            max_recommendations: 5,
            min_confidence_score: 0.7,
            enable_caching: true,
            custom_context: None,
        }
    }
}

/// Command type categories for hierarchical organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    /// File and directory operations
    FileSystem,
    /// Network and connectivity commands
    Network,
    /// System administration and configuration
    System,
    /// Development and build tools
    Development,
    /// Package management
    PackageManagement,
    /// Text processing and manipulation
    TextProcessing,
    /// Version control operations
    VersionControl,
    /// Database operations
    Database,
    /// Monitoring and diagnostics
    Monitoring,
    /// Security and permissions
    Security,
    /// Other/uncategorized commands
    Other,
}

impl CommandType {
    /// Get a human-readable description of the command type
    pub fn description(&self) -> &'static str {
        match self {
            CommandType::FileSystem => "File and directory operations including navigation, creation, deletion, and permissions",
            CommandType::Network => "Network connectivity, transfers, and communication commands",
            CommandType::System => "System administration, configuration, and service management",
            CommandType::Development => "Development tools, compilers, build systems, and testing",
            CommandType::PackageManagement => "Package installation, updates, and dependency management",
            CommandType::TextProcessing => "Text editing, searching, filtering, and manipulation",
            CommandType::VersionControl => "Git and other version control system operations",
            CommandType::Database => "Database queries, administration, and data management",
            CommandType::Monitoring => "System monitoring, process management, and diagnostics",
            CommandType::Security => "Security tools, encryption, and access control",
            CommandType::Other => "Miscellaneous commands that don't fit other categories",
        }
    }

    /// Get an emoji icon for the command type
    pub fn icon(&self) -> &'static str {
        match self {
            CommandType::FileSystem => "📁",
            CommandType::Network => "🌐",
            CommandType::System => "⚙️",
            CommandType::Development => "💻",
            CommandType::PackageManagement => "📦",
            CommandType::TextProcessing => "📝",
            CommandType::VersionControl => "🔀",
            CommandType::Database => "🗄️",
            CommandType::Monitoring => "📊",
            CommandType::Security => "🔒",
            CommandType::Other => "🔧",
        }
    }

    /// Classify a command based on its content
    pub fn classify_command(command: &str) -> Self {
        let cmd = command.trim().to_lowercase();
        let first_word = cmd.split_whitespace().next().unwrap_or("");

        match first_word {
            // File system operations
            "ls" | "dir" | "cd" | "pwd" | "mkdir" | "rmdir" | "rm" | "cp" | "mv" | "find" | "locate" | "which" | "whereis" | "chmod" | "chown" | "chgrp" | "ln" | "touch" | "stat" | "file" | "du" | "df" | "tree" => CommandType::FileSystem,
            
            // Network operations
            "ping" | "curl" | "wget" | "ssh" | "scp" | "rsync" | "ftp" | "sftp" | "telnet" | "nc" | "netcat" | "nslookup" | "dig" | "host" | "traceroute" | "netstat" | "ss" | "iptables" | "ufw" => CommandType::Network,
            
            // System administration
            "sudo" | "su" | "systemctl" | "service" | "ps" | "top" | "htop" | "kill" | "killall" | "jobs" | "bg" | "fg" | "nohup" | "crontab" | "mount" | "umount" | "fdisk" | "lsblk" | "free" | "uptime" | "uname" | "whoami" | "id" | "groups" | "passwd" | "useradd" | "userdel" | "usermod" => CommandType::System,
            
            // Development tools
            "gcc" | "g++" | "clang" | "make" | "cmake" | "cargo" | "npm" | "yarn" | "pip" | "python" | "python3" | "node" | "java" | "javac" | "rustc" | "go" | "docker" | "docker-compose" | "kubectl" | "helm" => CommandType::Development,
            
            // Package management
            "apt" | "apt-get" | "yum" | "dnf" | "pacman" | "brew" | "snap" | "flatpak" | "pip" | "conda" | "gem" | "composer" => CommandType::PackageManagement,
            
            // Text processing
            "cat" | "less" | "more" | "head" | "tail" | "grep" | "egrep" | "fgrep" | "sed" | "awk" | "cut" | "sort" | "uniq" | "wc" | "tr" | "tee" | "xargs" | "vim" | "nano" | "emacs" | "code" => CommandType::TextProcessing,
            
            // Version control
            "git" | "svn" | "hg" | "bzr" => CommandType::VersionControl,
            
            // Database
            "mysql" | "psql" | "sqlite3" | "mongo" | "redis-cli" => CommandType::Database,
            
            // Monitoring
            "iostat" | "vmstat" | "sar" | "lsof" | "strace" | "ltrace" | "tcpdump" | "wireshark" | "iftop" | "iotop" => CommandType::Monitoring,
            
            // Security
            "gpg" | "openssl" | "ssh-keygen" | "fail2ban" | "chkrootkit" | "rkhunter" | "lynis" => CommandType::Security,
            
            _ => CommandType::Other,
        }
    }
}

/// Workflow phase for organizing commands by development lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowPhase {
    /// Initial setup and preparation
    Setup,
    /// Development and coding
    Development,
    /// Building and compilation
    Build,
    /// Testing and validation
    Testing,
    /// Deployment and release
    Deployment,
    /// Maintenance and monitoring
    Maintenance,
    /// Debugging and troubleshooting
    Debugging,
    /// Other workflow activities
    Other,
}

impl WorkflowPhase {
    /// Get a human-readable description of the workflow phase
    pub fn description(&self) -> &'static str {
        match self {
            WorkflowPhase::Setup => "Initial project setup, environment configuration, and dependency installation",
            WorkflowPhase::Development => "Active development, coding, and feature implementation",
            WorkflowPhase::Build => "Compilation, building, and packaging of the application",
            WorkflowPhase::Testing => "Running tests, validation, and quality assurance",
            WorkflowPhase::Deployment => "Deployment, release, and production setup",
            WorkflowPhase::Maintenance => "Ongoing maintenance, updates, and system administration",
            WorkflowPhase::Debugging => "Troubleshooting, debugging, and issue resolution",
            WorkflowPhase::Other => "Miscellaneous workflow activities",
        }
    }

    /// Get an emoji icon for the workflow phase
    pub fn icon(&self) -> &'static str {
        match self {
            WorkflowPhase::Setup => "🔧",
            WorkflowPhase::Development => "💻",
            WorkflowPhase::Build => "🏗️",
            WorkflowPhase::Testing => "🧪",
            WorkflowPhase::Deployment => "🚀",
            WorkflowPhase::Maintenance => "🔄",
            WorkflowPhase::Debugging => "🐛",
            WorkflowPhase::Other => "📋",
        }
    }

    /// Classify a command based on its workflow context
    pub fn classify_command(command: &str, command_type: &CommandType) -> Self {
        let cmd = command.trim().to_lowercase();
        
        // Check for specific workflow indicators
        if cmd.contains("install") || cmd.contains("setup") || cmd.contains("init") || cmd.contains("clone") {
            return WorkflowPhase::Setup;
        }
        
        if cmd.contains("build") || cmd.contains("compile") || cmd.contains("make") {
            return WorkflowPhase::Build;
        }
        
        if cmd.contains("test") || cmd.contains("spec") || cmd.contains("check") {
            return WorkflowPhase::Testing;
        }
        
        if cmd.contains("deploy") || cmd.contains("release") || cmd.contains("publish") {
            return WorkflowPhase::Deployment;
        }
        
        if cmd.contains("debug") || cmd.contains("trace") || cmd.contains("log") {
            return WorkflowPhase::Debugging;
        }
        
        // Classify based on command type
        match command_type {
            CommandType::Development => WorkflowPhase::Development,
            CommandType::PackageManagement => WorkflowPhase::Setup,
            CommandType::System | CommandType::Monitoring => WorkflowPhase::Maintenance,
            _ => WorkflowPhase::Other,
        }
    }
}

/// Output theme/style presets for different formatting styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputTheme {
    /// Clean, minimal formatting
    Minimal,
    /// Professional documentation style
    Professional,
    /// Detailed technical documentation
    Technical,
    /// Compact format for space efficiency
    Compact,
    /// Rich formatting with emojis and visual elements
    Rich,
    /// GitHub-style markdown
    GitHub,
    /// Custom theme with user-defined styling
    Custom,
}

impl Default for OutputTheme {
    fn default() -> Self {
        OutputTheme::Professional
    }
}

/// Document sections that can be reordered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSection {
    /// Session metadata and summary
    SessionInfo,
    /// Table of contents
    TableOfContents,
    /// Command listing (main content)
    Commands,
    /// Session statistics
    Statistics,
    /// AI analysis and insights
    Analysis,
    /// Annotations and notes
    Annotations,
    /// Performance metrics
    Performance,
    /// Custom footer content
    Footer,
}

/// Markdown extensions to enable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkdownExtension {
    /// GitHub Flavored Markdown tables
    Tables,
    /// Task lists with checkboxes
    TaskLists,
    /// Strikethrough text
    Strikethrough,
    /// Footnotes
    Footnotes,
    /// Definition lists
    DefinitionLists,
    /// Math expressions
    Math,
    /// Mermaid diagrams
    Mermaid,
    /// Syntax highlighting
    SyntaxHighlighting,
}

/// Output verbosity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbosityLevel {
    /// Minimal output with only essential information
    Minimal,
    /// Standard output with common details
    Standard,
    /// Detailed output with additional context
    Detailed,
    /// Verbose output with all available information
    Verbose,
    /// Debug level with internal details
    Debug,
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        VerbosityLevel::Standard
    }
}

/// Hierarchical structure for organizing commands
#[derive(Debug, Clone)]
pub struct HierarchicalStructure {
    /// Commands grouped by workflow phase
    pub workflow_groups: HashMap<WorkflowPhase, Vec<CommandEntry>>,
    /// Commands grouped by command type
    pub type_groups: HashMap<CommandType, Vec<CommandEntry>>,
    /// Commands grouped by directory within each category
    pub nested_groups: HashMap<String, HashMap<String, Vec<CommandEntry>>>,
}

impl HierarchicalStructure {
    /// Create a new hierarchical structure from a list of commands
    pub fn new(commands: &[CommandEntry]) -> Self {
        let mut workflow_groups: HashMap<WorkflowPhase, Vec<CommandEntry>> = HashMap::new();
        let mut type_groups: HashMap<CommandType, Vec<CommandEntry>> = HashMap::new();
        let mut nested_groups: HashMap<String, HashMap<String, Vec<CommandEntry>>> = HashMap::new();

        for command in commands {
            let command_type = CommandType::classify_command(&command.command);
            let workflow_phase = WorkflowPhase::classify_command(&command.command, &command_type);

            // Group by workflow phase
            workflow_groups
                .entry(workflow_phase)
                .or_insert_with(Vec::new)
                .push(command.clone());

            // Group by command type
            type_groups
                .entry(command_type.clone())
                .or_insert_with(Vec::new)
                .push(command.clone());

            // Create nested grouping (type -> directory -> commands)
            let type_key = format!("{:?}", command_type);
            nested_groups
                .entry(type_key)
                .or_insert_with(HashMap::new)
                .entry(command.working_directory.clone())
                .or_insert_with(Vec::new)
                .push(command.clone());
        }

        Self {
            workflow_groups,
            type_groups,
            nested_groups,
        }
    }
}

/// Markdown template system for generating documentation
pub struct MarkdownTemplate {
    config: MarkdownConfig,
    code_block_generator: CodeBlockGenerator,
    ai_analyzer: Option<RefCell<AIAnalyzer>>,
}

impl MarkdownTemplate {
    /// Create a new markdown template with default configuration
    pub fn new() -> Self {
        let config = MarkdownConfig::default();
        let code_block_generator = CodeBlockGenerator::with_config(config.code_block_config.clone());
        Self {
            config,
            code_block_generator,
            ai_analyzer: None,
        }
    }

    /// Create a new markdown template with custom configuration
    pub fn with_config(config: MarkdownConfig) -> Self {
        let code_block_generator = CodeBlockGenerator::with_config(config.code_block_config.clone());
        Self {
            config,
            code_block_generator,
            ai_analyzer: None,
        }
    }

    /// Set up AI analyzer with LLM configuration
    pub fn with_ai_analyzer(mut self, llm_config: LlmConfig) -> Self {
        if self.config.ai_analysis_config.enable_ai_explanations {
            self.ai_analyzer = Some(RefCell::new(AIAnalyzer::new(llm_config)));
        }
        self
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: MarkdownConfig) {
        self.code_block_generator.set_config(config.code_block_config.clone());
        self.config = config;
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &MarkdownConfig {
        &self.config
    }

    /// Generate markdown content from a session
    pub async fn generate(&self, session: &Session) -> Result<String> {
        let mut content = String::new();

        // Generate document header
        self.write_header(&mut content, session)?;

        // Generate table of contents if enabled
        if self.config.template_options.include_toc {
            self.write_table_of_contents(&mut content, session)?;
        }

        // Generate session metadata
        if self.config.include_metadata {
            self.write_metadata(&mut content, session)?;
        }

        // Generate session statistics
        if self.config.include_statistics {
            self.write_statistics(&mut content, session)?;
        }

        // Generate commands section
        self.write_commands(&mut content, session).await?;

        // Generate annotations section
        if self.config.include_annotations && !session.annotations.is_empty() {
            self.write_annotations(&mut content, session)?;
        }

        // Generate document footer
        self.write_footer(&mut content, session)?;

        Ok(content)
    }

    /// Write the document header
    fn write_header(&self, content: &mut String, session: &Session) -> Result<()> {
        let title = self.config.template_options.title
            .as_ref()
            .unwrap_or(&session.description);

        writeln!(content, "# {}", title)?;
        writeln!(content)?;

        if let Some(custom_header) = &self.config.template_options.custom_header {
            writeln!(content, "{}", custom_header)?;
            writeln!(content)?;
        }

        // Add session overview
        writeln!(content, "## Session Overview")?;
        writeln!(content)?;
        writeln!(content, "**Session ID:** `{}`", session.id)?;
        writeln!(content, "**Description:** {}", session.description)?;
        writeln!(content, "**Status:** {:?}", session.state)?;
        
        if let Some(started_at) = session.started_at {
            writeln!(content, "**Started:** {}", self.format_timestamp(started_at))?;
        }
        
        if let Some(stopped_at) = session.stopped_at {
            writeln!(content, "**Stopped:** {}", self.format_timestamp(stopped_at))?;
        }

        if let Some(duration) = session.get_duration_seconds() {
            writeln!(content, "**Duration:** {}", self.format_duration(duration))?;
        }

        writeln!(content)?;

        Ok(())
    }

    /// Write table of contents
    fn write_table_of_contents(&self, content: &mut String, session: &Session) -> Result<()> {
        writeln!(content, "## Table of Contents")?;
        writeln!(content)?;

        if self.config.include_metadata {
            writeln!(content, "- [Session Metadata](#session-metadata)")?;
        }

        if self.config.include_statistics {
            writeln!(content, "- [Session Statistics](#session-statistics)")?;
        }

        writeln!(content, "- [Commands](#commands)")?;

        // Add hierarchical TOC entries if enabled
        if self.config.template_options.enable_hierarchical_structure && !session.commands.is_empty() {
            self.write_hierarchical_toc(content, session)?;
        }

        if self.config.include_annotations && !session.annotations.is_empty() {
            writeln!(content, "- [Annotations](#annotations)")?;
        }

        writeln!(content)?;
        Ok(())
    }

    /// Write hierarchical table of contents entries
    fn write_hierarchical_toc(&self, content: &mut String, session: &Session) -> Result<()> {
        let hierarchy = HierarchicalStructure::new(&session.commands);

        if self.config.template_options.group_by_workflow {
            self.write_workflow_toc(content, &hierarchy)?;
        } else if self.config.template_options.group_by_command_type {
            self.write_command_type_toc(content, &hierarchy)?;
        } else {
            // Default: workflow phases with nested command types
            self.write_workflow_with_types_toc(content, &hierarchy)?;
        }

        Ok(())
    }

    /// Write workflow-based TOC entries
    fn write_workflow_toc(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let workflow_order = [
            WorkflowPhase::Setup,
            WorkflowPhase::Development,
            WorkflowPhase::Build,
            WorkflowPhase::Testing,
            WorkflowPhase::Deployment,
            WorkflowPhase::Maintenance,
            WorkflowPhase::Debugging,
            WorkflowPhase::Other,
        ];

        for phase in &workflow_order {
            if let Some(commands) = hierarchy.workflow_groups.get(phase) {
                if !commands.is_empty() {
                    let phase_name = format!("{:?}", phase);
                    let anchor = phase_name.to_lowercase().replace(" ", "-");
                    writeln!(content, "  - [{} {} Phase](#{})", phase.icon(), phase_name, anchor)?;
                }
            }
        }

        Ok(())
    }

    /// Write command type-based TOC entries
    fn write_command_type_toc(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let type_order = [
            CommandType::FileSystem,
            CommandType::Development,
            CommandType::VersionControl,
            CommandType::PackageManagement,
            CommandType::System,
            CommandType::Network,
            CommandType::Database,
            CommandType::TextProcessing,
            CommandType::Monitoring,
            CommandType::Security,
            CommandType::Other,
        ];

        for cmd_type in &type_order {
            if let Some(commands) = hierarchy.type_groups.get(cmd_type) {
                if !commands.is_empty() {
                    let type_name = format!("{:?}", cmd_type).replace("_", " ");
                    let anchor = format!("{:?}", cmd_type).to_lowercase().replace("_", "-");
                    writeln!(content, "  - [{} {} Commands](#{}-commands)", cmd_type.icon(), type_name, anchor)?;
                }
            }
        }

        Ok(())
    }

    /// Write workflow with nested command types TOC entries
    fn write_workflow_with_types_toc(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let workflow_order = [
            WorkflowPhase::Setup,
            WorkflowPhase::Development,
            WorkflowPhase::Build,
            WorkflowPhase::Testing,
            WorkflowPhase::Deployment,
            WorkflowPhase::Maintenance,
            WorkflowPhase::Debugging,
            WorkflowPhase::Other,
        ];

        for phase in &workflow_order {
            if let Some(workflow_commands) = hierarchy.workflow_groups.get(phase) {
                if !workflow_commands.is_empty() {
                    let phase_name = format!("{:?}", phase);
                    let phase_anchor = phase_name.to_lowercase().replace(" ", "-");
                    writeln!(content, "  - [{} {} Phase](#{})", phase.icon(), phase_name, phase_anchor)?;

                    // Add nested command type entries if there are multiple types
                    let mut phase_type_groups: HashMap<CommandType, Vec<&CommandEntry>> = HashMap::new();
                    for command in workflow_commands {
                        let cmd_type = CommandType::classify_command(&command.command);
                        phase_type_groups
                            .entry(cmd_type)
                            .or_insert_with(Vec::new)
                            .push(command);
                    }

                    if phase_type_groups.len() > 1 {
                        for (cmd_type, type_commands) in &phase_type_groups {
                            if !type_commands.is_empty() {
                                let type_name = format!("{:?}", cmd_type).replace("_", " ");
                                let type_anchor = format!("{:?}", cmd_type).to_lowercase().replace("_", "-");
                                writeln!(content, "    - [{} {} Commands](#{})", cmd_type.icon(), type_name, type_anchor)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Write session metadata
    fn write_metadata(&self, content: &mut String, session: &Session) -> Result<()> {
        writeln!(content, "## Session Metadata")?;
        writeln!(content)?;

        writeln!(content, "| Property | Value |")?;
        writeln!(content, "|----------|-------|")?;
        writeln!(content, "| Working Directory | `{}` |", session.metadata.working_directory.display())?;
        writeln!(content, "| Shell Type | `{}` |", session.metadata.shell_type)?;
        writeln!(content, "| Platform | `{}` |", session.metadata.platform)?;
        writeln!(content, "| Hostname | `{}` |", session.metadata.hostname)?;
        
        if let Some(user) = &session.metadata.user {
            writeln!(content, "| User | `{}` |", user)?;
        }

        if let Some(llm_provider) = &session.metadata.llm_provider {
            writeln!(content, "| LLM Provider | `{}` |", llm_provider)?;
        }

        if !session.metadata.tags.is_empty() {
            let tags = session.metadata.tags.join(", ");
            writeln!(content, "| Tags | `{}` |", tags)?;
        }

        writeln!(content)?;
        Ok(())
    }

    /// Write session statistics
    fn write_statistics(&self, content: &mut String, session: &Session) -> Result<()> {
        writeln!(content, "## Session Statistics")?;
        writeln!(content)?;

        let stats = &session.stats;
        let success_rate = if stats.total_commands > 0 {
            (stats.successful_commands as f64 / stats.total_commands as f64) * 100.0
        } else {
            0.0
        };

        writeln!(content, "| Metric | Value |")?;
        writeln!(content, "|--------|-------|")?;
        writeln!(content, "| Total Commands | {} |", stats.total_commands)?;
        writeln!(content, "| Successful Commands | {} |", stats.successful_commands)?;
        writeln!(content, "| Failed Commands | {} |", stats.failed_commands)?;
        writeln!(content, "| Success Rate | {:.1}% |", success_rate)?;
        writeln!(content, "| Total Annotations | {} |", stats.total_annotations)?;
        writeln!(content, "| Pause/Resume Count | {} |", stats.pause_resume_count)?;

        if let Some(duration) = stats.duration_seconds {
            writeln!(content, "| Session Duration | {} |", self.format_duration(duration))?;
        }

        writeln!(content)?;
        Ok(())
    }

    /// Write commands section
    async fn write_commands(&self, content: &mut String, session: &Session) -> Result<()> {
        writeln!(content, "## Commands")?;
        writeln!(content)?;

        if session.commands.is_empty() {
            writeln!(content, "*No commands were captured during this session.*")?;
            writeln!(content)?;
            return Ok(());
        }

        // Use hierarchical structure if enabled
        if self.config.template_options.enable_hierarchical_structure {
            self.write_commands_hierarchical(content, session).await?;
        } else if self.config.template_options.group_by_directory {
            self.write_commands_grouped_by_directory(content, session).await?;
        } else if self.config.template_options.group_by_time {
            self.write_commands_grouped_by_time(content, session).await?;
        } else {
            self.write_commands_chronological(content, session).await?;
        }

        Ok(())
    }

    /// Write commands in chronological order
    async fn write_commands_chronological(&self, content: &mut String, session: &Session) -> Result<()> {
        for (index, command) in session.commands.iter().enumerate() {
            self.write_command(content, command, index + 1).await?;
        }
        Ok(())
    }

    /// Write commands grouped by working directory
    async fn write_commands_grouped_by_directory(&self, content: &mut String, session: &Session) -> Result<()> {
        let mut directory_groups: HashMap<String, Vec<&CommandEntry>> = HashMap::new();

        for command in &session.commands {
            directory_groups
                .entry(command.working_directory.clone())
                .or_insert_with(Vec::new)
                .push(command);
        }

        for (directory, commands) in directory_groups {
            writeln!(content, "### Directory: `{}`", directory)?;
            writeln!(content)?;

            for (index, command) in commands.iter().enumerate() {
                self.write_command(content, command, index + 1).await?;
            }
        }

        Ok(())
    }

    /// Write commands grouped by time periods
    async fn write_commands_grouped_by_time(&self, content: &mut String, session: &Session) -> Result<()> {
        let interval_minutes = self.config.template_options.time_group_interval;
        let mut time_groups: HashMap<String, Vec<&CommandEntry>> = HashMap::new();

        for command in &session.commands {
            let group_key = self.get_time_group_key(command.timestamp, interval_minutes);
            time_groups
                .entry(group_key)
                .or_insert_with(Vec::new)
                .push(command);
        }

        let mut sorted_groups: Vec<_> = time_groups.into_iter().collect();
        sorted_groups.sort_by_key(|(key, _)| key.clone());

        for (time_group, commands) in sorted_groups {
            writeln!(content, "### Time Period: {}", time_group)?;
            writeln!(content)?;

            for (index, command) in commands.iter().enumerate() {
                self.write_command(content, command, index + 1).await?;
            }
        }

        Ok(())
    }

    /// Write commands using hierarchical structure
    async fn write_commands_hierarchical(&self, content: &mut String, session: &Session) -> Result<()> {
        let hierarchy = HierarchicalStructure::new(&session.commands);
        
        if self.config.template_options.group_by_workflow {
            self.write_commands_by_workflow(content, &hierarchy).await?;
        } else if self.config.template_options.group_by_command_type {
            self.write_commands_by_type(content, &hierarchy).await?;
        } else {
            // Default hierarchical structure: workflow phases with nested command types
            self.write_commands_workflow_with_types(content, &hierarchy).await?;
        }
        
        Ok(())
    }

    /// Write commands grouped by workflow phases
    async fn write_commands_by_workflow(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let workflow_order = [
            WorkflowPhase::Setup,
            WorkflowPhase::Development,
            WorkflowPhase::Build,
            WorkflowPhase::Testing,
            WorkflowPhase::Deployment,
            WorkflowPhase::Maintenance,
            WorkflowPhase::Debugging,
            WorkflowPhase::Other,
        ];

        for phase in &workflow_order {
            if let Some(commands) = hierarchy.workflow_groups.get(phase) {
                if !commands.is_empty() {
                    writeln!(content, "### {} {} - {}", phase.icon(), format!("{:?}", phase), phase.description())?;
                    writeln!(content)?;

                    if self.config.template_options.include_workflow_summaries {
                        self.write_workflow_summary(content, phase, commands)?;
                    }

                    for (index, command) in commands.iter().enumerate() {
                        self.write_command(content, command, index + 1).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write commands grouped by command types
    async fn write_commands_by_type(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let type_order = [
            CommandType::FileSystem,
            CommandType::Development,
            CommandType::VersionControl,
            CommandType::PackageManagement,
            CommandType::System,
            CommandType::Network,
            CommandType::Database,
            CommandType::TextProcessing,
            CommandType::Monitoring,
            CommandType::Security,
            CommandType::Other,
        ];

        for cmd_type in &type_order {
            if let Some(commands) = hierarchy.type_groups.get(cmd_type) {
                if !commands.is_empty() {
                    writeln!(content, "### {} {} Commands", cmd_type.icon(), format!("{:?}", cmd_type).replace("_", " "))?;
                    writeln!(content)?;

                    if self.config.template_options.include_command_type_explanations {
                        writeln!(content, "*{}*", cmd_type.description())?;
                        writeln!(content)?;
                    }

                    for (index, command) in commands.iter().enumerate() {
                        self.write_command(content, command, index + 1).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write commands with workflow phases containing nested command types
    async fn write_commands_workflow_with_types(&self, content: &mut String, hierarchy: &HierarchicalStructure) -> Result<()> {
        let workflow_order = [
            WorkflowPhase::Setup,
            WorkflowPhase::Development,
            WorkflowPhase::Build,
            WorkflowPhase::Testing,
            WorkflowPhase::Deployment,
            WorkflowPhase::Maintenance,
            WorkflowPhase::Debugging,
            WorkflowPhase::Other,
        ];

        for phase in &workflow_order {
            if let Some(workflow_commands) = hierarchy.workflow_groups.get(phase) {
                if !workflow_commands.is_empty() {
                    writeln!(content, "### {} {} Phase", phase.icon(), format!("{:?}", phase))?;
                    writeln!(content)?;

                    if self.config.template_options.include_workflow_summaries {
                        writeln!(content, "*{}*", phase.description())?;
                        writeln!(content)?;
                    }

                    // Group commands within this workflow phase by type
                    let mut phase_type_groups: HashMap<CommandType, Vec<&CommandEntry>> = HashMap::new();
                    for command in workflow_commands {
                        let cmd_type = CommandType::classify_command(&command.command);
                        phase_type_groups
                            .entry(cmd_type)
                            .or_insert_with(Vec::new)
                            .push(command);
                    }

                    // Write each command type within this workflow phase
                    for (cmd_type, type_commands) in phase_type_groups {
                        if type_commands.len() > 1 || self.config.template_options.include_command_type_explanations {
                            writeln!(content, "#### {} {} Commands", cmd_type.icon(), format!("{:?}", cmd_type).replace("_", " "))?;
                            writeln!(content)?;
                        }

                        for (index, command) in type_commands.iter().enumerate() {
                            self.write_command(content, command, index + 1).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Write a workflow summary section
    fn write_workflow_summary(&self, content: &mut String, phase: &WorkflowPhase, commands: &[CommandEntry]) -> Result<()> {
        let total_commands = commands.len();
        let successful_commands = commands.iter().filter(|cmd| cmd.exit_code == Some(0)).count();
        let failed_commands = commands.iter().filter(|cmd| cmd.exit_code.is_some() && cmd.exit_code != Some(0)).count();

        writeln!(content, "**Phase Summary:** {} commands executed ({} successful, {} failed)",
                total_commands, successful_commands, failed_commands)?;
        writeln!(content)?;

        Ok(())
    }

    /// Write a single command entry
    async fn write_command(&self, content: &mut String, command: &CommandEntry, index: usize) -> Result<()> {
        // Command header with status indicator
        let status_indicator = if self.config.template_options.include_status_indicators {
            match command.exit_code {
                Some(0) => " ✅",
                Some(_) => " ❌",
                None => " ⏳",
            }
        } else {
            ""
        };

        writeln!(content, "### Command {}{}", index, status_indicator)?;
        writeln!(content)?;

        // Command details table
        writeln!(content, "| Property | Value |")?;
        writeln!(content, "|----------|-------|")?;
        writeln!(content, "| Command | `{}` |", self.escape_markdown(&command.command))?;
        
        if self.config.include_timestamps {
            writeln!(content, "| Timestamp | {} |", self.format_timestamp(command.timestamp))?;
        }

        writeln!(content, "| Working Directory | `{}` |", command.working_directory)?;
        writeln!(content, "| Shell | `{}` |", command.shell)?;

        if let Some(exit_code) = command.exit_code {
            writeln!(content, "| Exit Code | `{}` |", exit_code)?;
        }

        writeln!(content)?;

        // Generate enhanced command code block
        let command_block = self.code_block_generator.generate_command_block(command);
        let formatted_command = self.code_block_generator.format_code_block(&command_block);
        writeln!(content, "{}", formatted_command)?;

        // Command output with enhanced formatting
        if self.config.include_output {
            if let Some(output) = &command.output {
                if !output.trim().is_empty() {
                    let truncated_output = self.truncate_output(output);
                    let output_block = self.code_block_generator.generate_output_block(&truncated_output, &command.command);
                    let formatted_output = self.code_block_generator.format_code_block(&output_block);
                    writeln!(content, "{}", formatted_output)?;
                }
            }
        }

        // Command errors with enhanced formatting
        if self.config.include_errors {
            if let Some(error) = &command.error {
                if !error.trim().is_empty() {
                    let truncated_error = self.truncate_output(error);
                    let error_block = self.code_block_generator.generate_error_block(&truncated_error, &command.command);
                    let formatted_error = self.code_block_generator.format_code_block(&error_block);
                    writeln!(content, "{}", formatted_error)?;
                }
            }
        }

        // AI-generated analysis and explanations
        if self.config.ai_analysis_config.enable_ai_explanations {
            if let Some(ai_analysis) = self.generate_ai_analysis(command).await? {
                self.write_ai_analysis(content, &ai_analysis)?;
            }
        }

        Ok(())
    }

    /// Generate AI analysis for a command
    async fn generate_ai_analysis(&self, command: &CommandEntry) -> Result<Option<AnalysisResult>> {
        if let Some(analyzer_cell) = &self.ai_analyzer {
            let config = &self.config.ai_analysis_config;
            
            // Create analysis context from command
            let context = format!(
                "Command: {}\nWorking Directory: {}\nShell: {}\nExit Code: {:?}",
                command.command,
                command.working_directory,
                command.shell,
                command.exit_code
            );
            
            // Add output context if available
            let full_context = if let Some(output) = &command.output {
                format!("{}\nOutput: {}", context, self.truncate_output(output))
            } else {
                context
            };
            
            // Add error context if available
            let full_context = if let Some(error) = &command.error {
                format!("{}\nError: {}", full_context, self.truncate_output(error))
            } else {
                full_context
            };
            
            // Add custom context if provided
            let analysis_context = if let Some(custom_context) = &config.custom_context {
                format!("{}\n\nAdditional Context: {}", full_context, custom_context)
            } else {
                full_context
            };
            
            // Try to borrow mutably and perform analysis
            let analysis_result = {
                match analyzer_cell.try_borrow_mut() {
                    Ok(mut analyzer) => {
                        analyzer.analyze_command(command, Some(&analysis_context)).await
                    }
                    Err(_) => {
                        // RefCell is already borrowed, skip AI analysis for this command
                        eprintln!("AI analyzer is busy, skipping analysis for command: {}", command.command);
                        return Ok(None);
                    }
                }
            };

            match analysis_result {
                Ok(analysis) => {
                    // Filter analysis based on confidence score
                    if analysis.confidence_score >= config.min_confidence_score {
                        Ok(Some(analysis))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => {
                    // Log error but don't fail the entire markdown generation
                    eprintln!("AI analysis failed for command '{}': {}", command.command, e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Write AI analysis section to markdown
    fn write_ai_analysis(&self, content: &mut String, analysis: &AnalysisResult) -> Result<()> {
        let config = &self.config.ai_analysis_config;
        
        writeln!(content, "#### 🤖 AI Analysis")?;
        writeln!(content)?;
        
        // Main explanation (using summary)
        if !analysis.summary.is_empty() {
            writeln!(content, "**Summary:**")?;
            writeln!(content, "{}", analysis.summary)?;
            writeln!(content)?;
        }
        
        // Detailed analysis
        if config.include_detailed_analysis && !analysis.detailed_explanation.is_empty() {
            writeln!(content, "**Detailed Analysis:**")?;
            writeln!(content, "{}", analysis.detailed_explanation)?;
            writeln!(content)?;
        }
        
        // Security analysis (from issues)
        if config.include_security_analysis {
            let security_issues: Vec<_> = analysis.issues.iter()
                .filter(|issue| matches!(issue.category, crate::llm::analyzer::IssueCategory::Security))
                .collect();
            
            if !security_issues.is_empty() {
                writeln!(content, "**Security Analysis:**")?;
                for issue in security_issues {
                    writeln!(content, "⚠️ **{}**: {} - {}",
                            format!("{:?}", issue.severity),
                            issue.description,
                            issue.solution)?;
                }
                writeln!(content)?;
            }
        }
        
        // Context insights
        if config.include_context_insights && !analysis.context_insights.is_empty() {
            writeln!(content, "**Context Insights:**")?;
            for insight in &analysis.context_insights {
                writeln!(content, "💡 **{}**: {}",
                        format!("{:?}", insight.insight_type),
                        insight.description)?;
            }
            writeln!(content)?;
        }
        
        // Alternative suggestions
        if config.include_alternatives && !analysis.alternatives.is_empty() {
            writeln!(content, "**Alternative Commands:**")?;
            let max_alternatives = config.max_alternatives.min(analysis.alternatives.len());
            for (i, alternative) in analysis.alternatives.iter().take(max_alternatives).enumerate() {
                writeln!(content, "{}. `{}` - {} ({})",
                        i + 1,
                        alternative.command,
                        alternative.description,
                        format!("{:?}", alternative.complexity))?;
                if !alternative.advantages.is_empty() {
                    writeln!(content, "   - Advantages: {}", alternative.advantages.join(", "))?;
                }
            }
            writeln!(content)?;
        }
        
        // Recommendations
        if config.include_recommendations && !analysis.recommendations.is_empty() {
            writeln!(content, "**Recommendations:**")?;
            let max_recommendations = config.max_recommendations.min(analysis.recommendations.len());
            for (i, recommendation) in analysis.recommendations.iter().take(max_recommendations).enumerate() {
                writeln!(content, "{}. **{}** ({}): {}",
                        i + 1,
                        recommendation.title,
                        format!("{:?}", recommendation.priority),
                        recommendation.description)?;
                if !recommendation.implementation.is_empty() {
                    writeln!(content, "   - Implementation: {}", recommendation.implementation)?;
                }
            }
            writeln!(content)?;
        }
        
        // Confidence score (for debugging/transparency)
        if analysis.confidence_score < 1.0 {
            writeln!(content, "*Confidence: {:.1}%*", analysis.confidence_score * 100.0)?;
            writeln!(content)?;
        }
        
        Ok(())
    }

    /// Write annotations section
    fn write_annotations(&self, content: &mut String, session: &Session) -> Result<()> {
        writeln!(content, "## Annotations")?;
        writeln!(content)?;

        for (index, annotation) in session.annotations.iter().enumerate() {
            self.write_annotation(content, annotation, index + 1)?;
        }

        Ok(())
    }

    /// Write a single annotation
    fn write_annotation(&self, content: &mut String, annotation: &Annotation, index: usize) -> Result<()> {
        let type_emoji = match annotation.annotation_type {
            AnnotationType::Note => "📝",
            AnnotationType::Explanation => "💡",
            AnnotationType::Warning => "⚠️",
            AnnotationType::Milestone => "🎯",
        };

        writeln!(content, "### {} Annotation {}", type_emoji, index)?;
        writeln!(content)?;

        if self.config.include_timestamps {
            writeln!(content, "**Timestamp:** {}", self.format_timestamp(annotation.timestamp))?;
            writeln!(content)?;
        }

        writeln!(content, "{}", annotation.text)?;
        writeln!(content)?;

        Ok(())
    }

    /// Write document footer
    fn write_footer(&self, content: &mut String, session: &Session) -> Result<()> {
        if let Some(custom_footer) = &self.config.template_options.custom_footer {
            writeln!(content, "{}", custom_footer)?;
            writeln!(content)?;
        }

        writeln!(content, "---")?;
        writeln!(content)?;
        writeln!(content, "*Generated by DocPilot on {}*", self.format_timestamp(Utc::now()))?;

        Ok(())
    }

    /// Format a timestamp for display
    fn format_timestamp(&self, timestamp: DateTime<Utc>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Format a duration in seconds to human-readable format
    pub fn format_duration(&self, seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Get time group key for grouping commands by time periods
    fn get_time_group_key(&self, timestamp: DateTime<Utc>, interval_minutes: u64) -> String {
        let interval_seconds = interval_minutes * 60;
        let timestamp_seconds = timestamp.timestamp() as u64;
        let group_start = (timestamp_seconds / interval_seconds) * interval_seconds;
        let group_end = group_start + interval_seconds;

        let start_time = DateTime::from_timestamp(group_start as i64, 0)
            .unwrap_or_else(|| timestamp);
        let end_time = DateTime::from_timestamp(group_end as i64, 0)
            .unwrap_or_else(|| timestamp);

        format!("{} - {}", 
                start_time.format("%H:%M"),
                end_time.format("%H:%M"))
    }

    /// Escape markdown special characters
    pub fn escape_markdown(&self, text: &str) -> String {
        text.replace('`', "\\`")
            .replace('*', "\\*")
            .replace('_', "\\_")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }

    /// Truncate output if it exceeds the maximum length
    fn truncate_output(&self, output: &str) -> String {
        if self.config.max_output_length == 0 || output.len() <= self.config.max_output_length {
            output.to_string()
        } else {
            let truncated = &output[..self.config.max_output_length];
            format!("{}\n\n... (output truncated)", truncated)
        }
    }
}

impl Default for MarkdownTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// Main markdown generator that orchestrates the template system
pub struct MarkdownGenerator {
    template: MarkdownTemplate,
}

impl MarkdownGenerator {
    /// Create a new markdown generator with default template
    pub fn new() -> Self {
        Self {
            template: MarkdownTemplate::new(),
        }
    }

    /// Create a new markdown generator with custom configuration
    pub fn with_config(config: MarkdownConfig) -> Self {
        Self {
            template: MarkdownTemplate::with_config(config),
        }
    }

    /// Generate markdown documentation from a session
    pub async fn generate_documentation(&self, session: &Session) -> Result<String> {
        self.template.generate(session).await
    }

    /// Generate markdown documentation and write to file
    pub async fn generate_to_file(&self, session: &Session, output_path: &std::path::Path) -> Result<()> {
        let content = self.generate_documentation(session).await?;
        std::fs::write(output_path, content)?;
        Ok(())
    }

    /// Update the generator configuration
    pub fn set_config(&mut self, config: MarkdownConfig) {
        self.template.set_config(config);
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &MarkdownConfig {
        self.template.get_config()
    }

    /// Enable AI analysis with the provided LLM configuration
    pub fn enable_ai_analysis(&mut self, llm_config: crate::llm::LlmConfig) {
        // Update the AI analysis config to enable AI features
        let mut config = self.template.get_config().clone();
        config.ai_analysis_config.enable_ai_explanations = true;
        
        // Create a new template with AI analyzer
        self.template = MarkdownTemplate::with_config(config).with_ai_analyzer(llm_config);
    }

    /// Generate AI-enhanced documentation with post-processing
    pub async fn generate_ai_enhanced_documentation(&mut self, session: &Session) -> Result<String> {
        // First, validate and filter commands using AI
        if let Some(ai_analyzer_cell) = &self.template.ai_analyzer {
            let mut ai_analyzer = ai_analyzer_cell.borrow_mut();
            
            // Filter and validate commands
            let validated_commands = ai_analyzer.validate_and_enhance_commands(&session.commands).await?;
            
            // Create a temporary session with validated commands for generation
            let mut enhanced_session = session.clone();
            enhanced_session.commands = validated_commands;
            
            // Generate the base documentation
            let base_markdown = self.template.generate(&enhanced_session).await?;
            
            // Post-process the markdown using AI
            let enhanced_markdown = self.post_process_markdown_with_ai(&base_markdown, &enhanced_session).await?;
            
            Ok(enhanced_markdown)
        } else {
            // Fallback to regular generation if no AI analyzer
            self.template.generate(session).await
        }
    }

    /// Post-process generated markdown using AI to improve quality
    async fn post_process_markdown_with_ai(&self, markdown: &str, session: &Session) -> Result<String> {
        if let Some(ai_analyzer_cell) = &self.template.ai_analyzer {
            // Use try_borrow to avoid conflicts
            match ai_analyzer_cell.try_borrow() {
                Ok(_ai_analyzer) => {
                    // Use the prompt engine to create a markdown post-processing prompt
                    let prompt_engine = crate::llm::prompt::PromptEngine::new();
                    let (system_prompt, user_prompt) = prompt_engine.generate_markdown_processing_prompt(
                        markdown,
                        Some(&session.description),
                        Some("Development team")
                    )?;
                    
                    // Query the LLM to improve the markdown
                    let llm_response = self.query_llm_for_enhancement(&system_prompt, &user_prompt).await?;
                    
                    // Return the enhanced markdown or fall back to original if processing fails
                    if llm_response.len() > 100 && !llm_response.contains("Analysis unavailable") {
                        Ok(llm_response)
                    } else {
                        Ok(markdown.to_string())
                    }
                }
                Err(_) => {
                    // RefCell is already borrowed, skip post-processing
                    eprintln!("AI analyzer is busy, skipping markdown post-processing");
                    Ok(markdown.to_string())
                }
            }
        } else {
            Ok(markdown.to_string())
        }
    }

    /// Query LLM for markdown enhancement
    async fn query_llm_for_enhancement(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        if let Some(ai_analyzer_cell) = &self.template.ai_analyzer {
            // Try to borrow and get config
            let (provider_name, api_key) = match ai_analyzer_cell.try_borrow() {
                Ok(ai_analyzer) => {
                    // Get LLM configuration from the analyzer
                    let config = ai_analyzer.get_config();
                    
                    // Get default provider
                    let provider_name = config.get_default_provider()
                        .ok_or_else(|| anyhow!("No default LLM provider configured"))?;

                    // Get API key
                    let api_key = config.get_api_key_with_fallback(provider_name)
                        .ok_or_else(|| anyhow!("No API key found for provider: {}", provider_name))?;
                    
                    (provider_name.to_string(), api_key.to_string())
                }
                Err(_) => {
                    return Err(anyhow!("AI analyzer is busy, cannot perform enhancement"));
                }
            };

            // Create LLM client
            let provider = crate::llm::client::LlmProvider::from_str(&provider_name)?;
            let client = crate::llm::client::LlmClient::new(provider, api_key)?;

            // Create request with higher token limit for documentation processing
            let request = crate::llm::client::LlmRequest {
                prompt: user_prompt.to_string(),
                max_tokens: Some(4000), // Higher limit for documentation
                temperature: Some(0.2), // Lower temperature for consistent formatting
                system_prompt: Some(system_prompt.to_string()),
            };

            // Get response
            match client.generate(request).await {
                Ok(response) => Ok(response.content),
                Err(e) => {
                    eprintln!("LLM enhancement failed: {}", e);
                    Err(anyhow!("Failed to enhance markdown: {}", e))
                }
            }
        } else {
            Err(anyhow!("No AI analyzer available for enhancement"))
        }
    }

    /// Generate comprehensive documentation with AI assistance
    pub async fn generate_comprehensive_ai_documentation(&mut self, session: &Session) -> Result<String> {
        if let Some(ai_analyzer_cell) = &self.template.ai_analyzer {
            // Try to borrow and generate enhanced documentation
            let enhanced_doc = match ai_analyzer_cell.try_borrow_mut() {
                Ok(mut ai_analyzer) => {
                    // Use AI to generate enhanced documentation structure
                    let _commands: Vec<String> = session.commands.iter().map(|c| c.command.clone()).collect();
                    ai_analyzer.generate_enhanced_documentation(&session.commands, Some(&session.description)).await?
                }
                Err(_) => {
                    eprintln!("AI analyzer is busy, generating basic documentation instead");
                    return self.template.generate(session).await;
                }
            };
            
            // Combine with regular markdown generation for complete documentation
            let base_markdown = self.template.generate(session).await?;
            
            // Merge AI-generated content with template-generated content
            let combined_markdown = format!(
                "{}\n\n## AI-Enhanced Analysis\n\n{}\n\n## Detailed Command Log\n\n{}",
                self.generate_executive_summary(session),
                enhanced_doc,
                base_markdown
            );
            
            // Post-process the combined content
            self.post_process_markdown_with_ai(&combined_markdown, session).await
        } else {
            // Fallback to enhanced generation without AI analysis
            self.generate_ai_enhanced_documentation(session).await
        }
    }

    /// Generate executive summary for the documentation
    fn generate_executive_summary(&self, session: &Session) -> String {
        let mut summary = String::new();
        
        summary.push_str("# Executive Summary\n\n");
        summary.push_str(&format!("This documentation captures the **{}** workflow session.\n\n", session.description));
        
        // Add session statistics
        summary.push_str("## Session Overview\n\n");
        summary.push_str(&format!("- **Total Commands**: {}\n", session.stats.total_commands));
        summary.push_str(&format!("- **Successful Commands**: {}\n", session.stats.successful_commands));
        summary.push_str(&format!("- **Failed Commands**: {}\n", session.stats.failed_commands));
        summary.push_str(&format!("- **Annotations**: {}\n", session.stats.total_annotations));
        
        if let Some(duration) = session.get_duration_seconds() {
            let hours = duration / 3600;
            let minutes = (duration % 3600) / 60;
            if hours > 0 {
                summary.push_str(&format!("- **Duration**: {}h {}m\n", hours, minutes));
            } else if minutes > 0 {
                summary.push_str(&format!("- **Duration**: {}m\n", minutes));
            } else {
                summary.push_str(&format!("- **Duration**: {}s\n", duration));
            }
        }
        
        summary.push_str("\n");
        summary
    }

    /// Create a minimal configuration for quick documentation
    pub fn minimal_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: false,
            include_timestamps: false,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: false,
            max_output_length: 500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: None,
                include_toc: false,
                group_by_directory: false,
                group_by_time: false,
                time_group_interval: 30,
                include_status_indicators: false,
                custom_header: None,
                custom_footer: None,
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 2,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                ..TemplateOptions::default()
            },
            code_block_config: {
                let mut config = CodeBlockConfig::default();
                config.enable_block_titles = false;
                config.enable_collapsible_blocks = false;
                config
            },
            ai_analysis_config: AIAnalysisConfig {
                enable_ai_explanations: false,
                include_detailed_analysis: false,
                include_security_analysis: false,
                include_alternatives: false,
                include_context_insights: false,
                include_recommendations: false,
                max_alternatives: 1,
                max_recommendations: 2,
                min_confidence_score: 0.8,
                enable_caching: true,
                custom_context: None,
            },
        }
    }

    /// Create a comprehensive configuration for detailed documentation
    pub fn comprehensive_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 2000,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: None,
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 30,
                include_status_indicators: true,
                custom_header: None,
                custom_footer: None,
                enable_hierarchical_structure: true,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 3,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                ..TemplateOptions::default()
            },
            code_block_config: {
                let mut config = CodeBlockConfig::default();
                config.enable_block_titles = true;
                config.enable_collapsible_blocks = true;
                config.enable_syntax_hints = true;
                config.collapsible_threshold = 15;
                config
            },
            ai_analysis_config: AIAnalysisConfig {
                enable_ai_explanations: true,
                include_detailed_analysis: true,
                include_security_analysis: true,
                include_alternatives: true,
                include_context_insights: true,
                include_recommendations: true,
                max_alternatives: 3,
                max_recommendations: 5,
                min_confidence_score: 0.7,
                enable_caching: true,
                custom_context: None,
            },
        }
    }

    /// Create an AI-enhanced configuration for intelligent documentation
    pub fn ai_enhanced_config() -> MarkdownConfig {
        let mut config = Self::comprehensive_config();
        config.ai_analysis_config = AIAnalysisConfig {
            enable_ai_explanations: true,
            include_detailed_analysis: true,
            include_security_analysis: true,
            include_alternatives: true,
            include_context_insights: false, // Can be verbose
            include_recommendations: true,
            max_alternatives: 3,
            max_recommendations: 5,
            min_confidence_score: 0.7,
            enable_caching: true,
            custom_context: Some("Focus on practical insights and actionable recommendations for terminal commands.".to_string()),
        };
        config
    }

    /// Create a hierarchical configuration for organized documentation
    pub fn hierarchical_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 1500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: None,
                include_toc: true,
                group_by_directory: false,
                group_by_time: false,
                time_group_interval: 30,
                include_status_indicators: true,
                custom_header: None,
                custom_footer: None,
                enable_hierarchical_structure: true,
                group_by_workflow: true,
                group_by_command_type: true,
                max_hierarchy_depth: 4,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                ..TemplateOptions::default()
            },
            code_block_config: {
                let mut config = CodeBlockConfig::default();
                config.enable_block_titles = true;
                config.enable_collapsible_blocks = true;
                config.enable_syntax_hints = true;
                config.collapsible_threshold = 10;
                config
            },
            ai_analysis_config: AIAnalysisConfig {
                enable_ai_explanations: false, // Disabled by default for performance
                include_detailed_analysis: true,
                include_security_analysis: true,
                include_alternatives: true,
                include_context_insights: false,
                include_recommendations: true,
                max_alternatives: 2,
                max_recommendations: 3,
                min_confidence_score: 0.75,
                enable_caching: true,
                custom_context: Some("Focus on workflow organization and command categorization.".to_string()),
            },
        }
    }
    /// Create a professional configuration for business documentation
    pub fn professional_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: false, // Keep it clean for professional docs
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Professional Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Professional Terminal Session Documentation\n\nGenerated for business and technical review.\n".to_string()),
                custom_footer: Some("\n---\n*Generated by DocPilot Terminal Documentation Tool*".to_string()),
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 2,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // Professional formatting options
                date_format: "%B %d, %Y at %I:%M %p".to_string(),
                theme: OutputTheme::Professional,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: false,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Statistics,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: false,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                ],
                verbosity_level: VerbosityLevel::Standard,
                include_command_relationships: false,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a compact configuration for space-efficient documentation
    pub fn compact_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: false,
            include_timestamps: false,
            include_output: false,
            include_errors: true,
            include_annotations: false,
            include_statistics: false,
            max_output_length: 100,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Compact Documentation".to_string()),
                include_toc: false,
                group_by_directory: false,
                group_by_time: false,
                time_group_interval: 30,
                include_status_indicators: false,
                custom_header: None,
                custom_footer: None,
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 1,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // Compact formatting options
                date_format: "%m/%d %H:%M".to_string(),
                theme: OutputTheme::Compact,
                include_duration: false,
                include_working_directory: false,
                include_exit_codes: false,
                include_environment_vars: false,
                max_commands: 50, // Limit for compactness
                include_command_numbers: false,
                use_collapsible_sections: false,
                include_session_summary: false,
                include_command_stats: false,
                section_order: vec![
                    DocumentSection::Commands,
                ],
                include_breadcrumbs: false,
                use_emoji_indicators: false,
                include_performance_metrics: false,
                markdown_extensions: vec![
                    MarkdownExtension::SyntaxHighlighting,
                ],
                verbosity_level: VerbosityLevel::Minimal,
                include_command_relationships: false,
                use_compact_formatting: true,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a rich configuration with visual enhancements
    pub fn rich_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 2000,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("🚀 Rich Terminal Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: true,
                time_group_interval: 30,
                include_status_indicators: true,
                custom_header: Some("# 🚀 Rich Terminal Session Documentation\n\n> **Enhanced with visual elements and comprehensive details**\n\n".to_string()),
                custom_footer: Some("\n---\n\n🔧 *Generated by DocPilot Terminal Documentation Tool*  \n📅 *Documentation created with rich formatting*".to_string()),
                enable_hierarchical_structure: true,
                group_by_workflow: true,
                group_by_command_type: true,
                max_hierarchy_depth: 4,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                
                // Rich formatting options
                date_format: "📅 %A, %B %d, %Y at %I:%M:%S %p".to_string(),
                theme: OutputTheme::Rich,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: true,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Analysis,
                    DocumentSection::Statistics,
                    DocumentSection::Performance,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: true,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::Strikethrough,
                    MarkdownExtension::Footnotes,
                    MarkdownExtension::Mermaid,
                ],
                verbosity_level: VerbosityLevel::Verbose,
                include_command_relationships: true,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a technical configuration for detailed technical documentation
    pub fn technical_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 1500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Technical Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Technical Terminal Session Documentation\n\n**Detailed technical analysis and command documentation**\n\n".to_string()),
                custom_footer: Some("\n---\n\n**Technical Notes:**\n- All commands include exit codes and execution times\n- Environment variables are documented where relevant\n- Performance metrics are included for analysis\n\n*Generated by DocPilot Terminal Documentation Tool*".to_string()),
                enable_hierarchical_structure: true,
                group_by_workflow: true,
                group_by_command_type: true,
                max_hierarchy_depth: 3,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                
                // Technical formatting options
                date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
                theme: OutputTheme::Technical,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: true,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Analysis,
                    DocumentSection::Performance,
                    DocumentSection::Statistics,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: false,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::DefinitionLists,
                    MarkdownExtension::Math,
                ],
                verbosity_level: VerbosityLevel::Detailed,
                include_command_relationships: true,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a GitHub-style configuration for repository documentation
    pub fn github_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: false,
            max_output_length: 800,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Terminal Session Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Terminal Session Documentation\n\n> Documentation of terminal commands and their execution\n\n".to_string()),
                custom_footer: Some("\n---\n\n**Generated by [DocPilot](https://github.com/your-org/docpilot)**".to_string()),
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 2,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // GitHub-style formatting options
                date_format: "%Y-%m-%d %H:%M:%S".to_string(),
                theme: OutputTheme::GitHub,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: false,
                max_commands: 0,
                include_command_numbers: false,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: false,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: false,
                use_emoji_indicators: false,
                include_performance_metrics: false,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::Strikethrough,
                ],
                verbosity_level: VerbosityLevel::Standard,
                include_command_relationships: false,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }
}

impl Default for MarkdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod markdown_hierarchical_test {
    use super::*;
    use crate::session::manager::Session;
    use crate::terminal::monitor::CommandEntry;
    use chrono::{DateTime, Utc};

    fn create_test_session_with_hierarchical_commands() -> Session {
        let mut session = Session::new("Test Session".to_string(), None).unwrap();
        
        // Add commands that represent different workflow phases and types
        let commands = vec![
            // Setup phase - File System commands
            CommandEntry {
                command: "mkdir project".to_string(),
                working_directory: "/home/user".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:00:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            CommandEntry {
                command: "cd project".to_string(),
                working_directory: "/home/user".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:01:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            
            // Development phase - Development commands
            CommandEntry {
                command: "npm init -y".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:02:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("package.json created".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            CommandEntry {
                command: "git init".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:03:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("Initialized empty Git repository".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            
            // Build phase - Development commands
            CommandEntry {
                command: "npm install express".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:04:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("added 1 package".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            CommandEntry {
                command: "npm run build".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:05:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("Build completed successfully".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            
            // Testing phase - Development commands
            CommandEntry {
                command: "npm test".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:06:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("All tests passed".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            
            // Deployment phase - System commands
            CommandEntry {
                command: "docker build -t myapp .".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:07:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("Successfully built image".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
            
            // Monitoring phase - System commands
            CommandEntry {
                command: "ps aux | grep node".to_string(),
                working_directory: "/home/user/project".to_string(),
                timestamp: DateTime::parse_from_rfc3339("2023-01-01T10:08:00Z").unwrap().with_timezone(&Utc),
                exit_code: Some(0),
                output: Some("node process running".to_string()),
                error: None,
                shell: "bash".to_string(),
            },
        ];
        
        for command in commands {
            session.add_command(command);
        }
        
        session
    }

    #[tokio::test]
    async fn test_hierarchical_structure_creation() {
        let session = create_test_session_with_hierarchical_commands();
        let hierarchy = HierarchicalStructure::new(&session.commands);
        
        // Check that commands are properly grouped by workflow phase
        assert!(!hierarchy.workflow_groups.is_empty());
        assert!(hierarchy.workflow_groups.contains_key(&WorkflowPhase::Setup));
        assert!(hierarchy.workflow_groups.contains_key(&WorkflowPhase::Build));
        assert!(hierarchy.workflow_groups.contains_key(&WorkflowPhase::Testing));
        assert!(hierarchy.workflow_groups.contains_key(&WorkflowPhase::Maintenance));
        
        // Check that commands are properly grouped by type
        assert!(!hierarchy.type_groups.is_empty());
        assert!(hierarchy.type_groups.contains_key(&CommandType::FileSystem));
        assert!(hierarchy.type_groups.contains_key(&CommandType::Development));
        assert!(hierarchy.type_groups.contains_key(&CommandType::VersionControl));
        assert!(hierarchy.type_groups.contains_key(&CommandType::System));
    }

    #[tokio::test]
    async fn test_command_type_classification() {
        // Test file system commands
        assert_eq!(CommandType::classify_command("mkdir test"), CommandType::FileSystem);
        assert_eq!(CommandType::classify_command("cd /home"), CommandType::FileSystem);
        assert_eq!(CommandType::classify_command("ls -la"), CommandType::FileSystem);
        
        // Test development commands
        assert_eq!(CommandType::classify_command("npm install"), CommandType::Development);
        assert_eq!(CommandType::classify_command("git commit"), CommandType::VersionControl);
        assert_eq!(CommandType::classify_command("cargo build"), CommandType::Development);
        
        // Test system commands
        assert_eq!(CommandType::classify_command("ps aux"), CommandType::System);
        assert_eq!(CommandType::classify_command("docker run"), CommandType::Development);
        assert_eq!(CommandType::classify_command("systemctl start"), CommandType::System);
        
        // Test network commands
        assert_eq!(CommandType::classify_command("curl https://api.example.com"), CommandType::Network);
        assert_eq!(CommandType::classify_command("wget file.zip"), CommandType::Network);
        
        // Test text processing commands
        assert_eq!(CommandType::classify_command("grep pattern file.txt"), CommandType::TextProcessing);
        assert_eq!(CommandType::classify_command("sed 's/old/new/g'"), CommandType::TextProcessing);
    }

    #[tokio::test]
    async fn test_workflow_phase_classification() {
        // Test setup phase (commands with setup keywords)
        assert_eq!(
            WorkflowPhase::classify_command("npm init -y", &CommandType::Development),
            WorkflowPhase::Setup
        );
        assert_eq!(
            WorkflowPhase::classify_command("git init", &CommandType::VersionControl),
            WorkflowPhase::Setup
        );
        assert_eq!(
            WorkflowPhase::classify_command("npm install express", &CommandType::Development),
            WorkflowPhase::Setup
        );
        
        // Test build phase
        assert_eq!(
            WorkflowPhase::classify_command("cargo build", &CommandType::Development),
            WorkflowPhase::Build
        );
        assert_eq!(
            WorkflowPhase::classify_command("npm run build", &CommandType::Development),
            WorkflowPhase::Build
        );
        
        // Test testing phase
        assert_eq!(
            WorkflowPhase::classify_command("cargo test", &CommandType::Development),
            WorkflowPhase::Testing
        );
        assert_eq!(
            WorkflowPhase::classify_command("npm test", &CommandType::Development),
            WorkflowPhase::Testing
        );
        
        // Test deployment phase
        assert_eq!(
            WorkflowPhase::classify_command("docker build", &CommandType::Development),
            WorkflowPhase::Build
        );
        assert_eq!(
            WorkflowPhase::classify_command("kubectl apply", &CommandType::System),
            WorkflowPhase::Maintenance
        );
        
        // Test other phase (commands without specific keywords)
        assert_eq!(
            WorkflowPhase::classify_command("mkdir project", &CommandType::FileSystem),
            WorkflowPhase::Other
        );
        assert_eq!(
            WorkflowPhase::classify_command("ls -la", &CommandType::FileSystem),
            WorkflowPhase::Other
        );
    }

    #[tokio::test]
    async fn test_hierarchical_markdown_generation() {
        let session = create_test_session_with_hierarchical_commands();
        let config = MarkdownGenerator::hierarchical_config();
        let template = MarkdownTemplate::with_config(config);
        
        let result = template.generate(&session).await;
        assert!(result.is_ok());
        
        let markdown = result.unwrap();
        
        // Check that hierarchical structure is present (based on actual workflow phases)
        assert!(markdown.contains("🔧 Setup"));
        assert!(markdown.contains("🏗️ Build"));
        assert!(markdown.contains("🧪 Testing"));
        assert!(markdown.contains("🔄 Maintenance"));
        assert!(markdown.contains("📋 Other"));
        
        // Check that phase summaries are present
        assert!(markdown.contains("Phase Summary:"));
        assert!(markdown.contains("3 commands executed"));
        assert!(markdown.contains("2 commands executed"));
        assert!(markdown.contains("1 commands executed"));
        
        // Check that hierarchical TOC is present
        assert!(markdown.contains("[🔧 Setup Phase](#setup)"));
        assert!(markdown.contains("[🏗️ Build Phase](#build)"));
        assert!(markdown.contains("[🧪 Testing Phase](#testing)"));
        
        // Check that specific commands are present
        assert!(markdown.contains("mkdir project"));
        assert!(markdown.contains("npm init -y"));
        assert!(markdown.contains("docker build"));
    }

    #[tokio::test]
    async fn test_hierarchical_config_vs_minimal_config() {
        let session = create_test_session_with_hierarchical_commands();
        
        // Test minimal config (non-hierarchical)
        let minimal_config = MarkdownGenerator::minimal_config();
        let minimal_template = MarkdownTemplate::with_config(minimal_config);
        let minimal_result = minimal_template.generate(&session).await;
        assert!(minimal_result.is_ok());
        let minimal_markdown = minimal_result.unwrap();
        
        // Test hierarchical config
        let hierarchical_config = MarkdownGenerator::hierarchical_config();
        let hierarchical_template = MarkdownTemplate::with_config(hierarchical_config);
        let hierarchical_result = hierarchical_template.generate(&session).await;
        assert!(hierarchical_result.is_ok());
        let hierarchical_markdown = hierarchical_result.unwrap();
        
        // Hierarchical should be longer and more structured
        assert!(hierarchical_markdown.len() > minimal_markdown.len());
        
        // Hierarchical should have workflow phase headers
        assert!(hierarchical_markdown.contains("🔧 Setup"));
        assert!(!minimal_markdown.contains("🔧 Setup"));
        
        // Both should contain the actual commands
        assert!(minimal_markdown.contains("mkdir project"));
        assert!(hierarchical_markdown.contains("mkdir project"));
    }
}
    /// Create a professional configuration for business documentation
    pub fn professional_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: false, // Keep it clean for professional docs
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Professional Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Professional Terminal Session Documentation\n\nGenerated for business and technical review.\n".to_string()),
                custom_footer: Some("\n---\n*Generated by DocPilot Terminal Documentation Tool*".to_string()),
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 2,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // Professional formatting options
                date_format: "%B %d, %Y at %I:%M %p".to_string(),
                theme: OutputTheme::Professional,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: false,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Statistics,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: false,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                ],
                verbosity_level: VerbosityLevel::Standard,
                include_command_relationships: false,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a compact configuration for space-efficient documentation
    pub fn compact_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: false,
            include_timestamps: false,
            include_output: false,
            include_errors: true,
            include_annotations: false,
            include_statistics: false,
            max_output_length: 100,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Compact Documentation".to_string()),
                include_toc: false,
                group_by_directory: false,
                group_by_time: false,
                time_group_interval: 30,
                include_status_indicators: false,
                custom_header: None,
                custom_footer: None,
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 1,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // Compact formatting options
                date_format: "%m/%d %H:%M".to_string(),
                theme: OutputTheme::Compact,
                include_duration: false,
                include_working_directory: false,
                include_exit_codes: false,
                include_environment_vars: false,
                max_commands: 50, // Limit for compactness
                include_command_numbers: false,
                use_collapsible_sections: false,
                include_session_summary: false,
                include_command_stats: false,
                section_order: vec![
                    DocumentSection::Commands,
                ],
                include_breadcrumbs: false,
                use_emoji_indicators: false,
                include_performance_metrics: false,
                markdown_extensions: vec![
                    MarkdownExtension::SyntaxHighlighting,
                ],
                verbosity_level: VerbosityLevel::Minimal,
                include_command_relationships: false,
                use_compact_formatting: true,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a rich configuration with visual enhancements
    pub fn rich_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 2000,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("🚀 Rich Terminal Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: true,
                time_group_interval: 30,
                include_status_indicators: true,
                custom_header: Some("# 🚀 Rich Terminal Session Documentation\n\n> **Enhanced with visual elements and comprehensive details**\n\n".to_string()),
                custom_footer: Some("\n---\n\n🔧 *Generated by DocPilot Terminal Documentation Tool*  \n📅 *Documentation created with rich formatting*".to_string()),
                enable_hierarchical_structure: true,
                group_by_workflow: true,
                group_by_command_type: true,
                max_hierarchy_depth: 4,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                
                // Rich formatting options
                date_format: "📅 %A, %B %d, %Y at %I:%M:%S %p".to_string(),
                theme: OutputTheme::Rich,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: true,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Analysis,
                    DocumentSection::Statistics,
                    DocumentSection::Performance,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: true,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::Strikethrough,
                    MarkdownExtension::Footnotes,
                    MarkdownExtension::Mermaid,
                ],
                verbosity_level: VerbosityLevel::Verbose,
                include_command_relationships: true,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a technical configuration for detailed technical documentation
    pub fn technical_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: true,
            max_output_length: 1500,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Technical Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Technical Terminal Session Documentation\n\n**Detailed technical analysis and command documentation**\n\n".to_string()),
                custom_footer: Some("\n---\n\n**Technical Notes:**\n- All commands include exit codes and execution times\n- Environment variables are documented where relevant\n- Performance metrics are included for analysis\n\n*Generated by DocPilot Terminal Documentation Tool*".to_string()),
                enable_hierarchical_structure: true,
                group_by_workflow: true,
                group_by_command_type: true,
                max_hierarchy_depth: 3,
                include_workflow_summaries: true,
                include_command_type_explanations: true,
                
                // Technical formatting options
                date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
                theme: OutputTheme::Technical,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: true,
                max_commands: 0,
                include_command_numbers: true,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: true,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Analysis,
                    DocumentSection::Performance,
                    DocumentSection::Statistics,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: true,
                use_emoji_indicators: false,
                include_performance_metrics: true,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::DefinitionLists,
                    MarkdownExtension::Math,
                ],
                verbosity_level: VerbosityLevel::Detailed,
                include_command_relationships: true,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }

    /// Create a GitHub-style configuration for repository documentation
    pub fn github_config() -> MarkdownConfig {
        MarkdownConfig {
            include_metadata: true,
            include_timestamps: true,
            include_output: true,
            include_errors: true,
            include_annotations: true,
            include_statistics: false,
            max_output_length: 800,
            code_language: "bash".to_string(),
            css_classes: HashMap::new(),
            template_options: TemplateOptions {
                title: Some("Terminal Session Documentation".to_string()),
                include_toc: true,
                group_by_directory: true,
                group_by_time: false,
                time_group_interval: 60,
                include_status_indicators: true,
                custom_header: Some("# Terminal Session Documentation\n\n> Documentation of terminal commands and their execution\n\n".to_string()),
                custom_footer: Some("\n---\n\n**Generated by [DocPilot](https://github.com/your-org/docpilot)**".to_string()),
                enable_hierarchical_structure: false,
                group_by_workflow: false,
                group_by_command_type: false,
                max_hierarchy_depth: 2,
                include_workflow_summaries: false,
                include_command_type_explanations: false,
                
                // GitHub-style formatting options
                date_format: "%Y-%m-%d %H:%M:%S".to_string(),
                theme: OutputTheme::GitHub,
                include_duration: true,
                include_working_directory: true,
                include_exit_codes: true,
                include_environment_vars: false,
                max_commands: 0,
                include_command_numbers: false,
                use_collapsible_sections: true,
                include_session_summary: true,
                include_command_stats: false,
                section_order: vec![
                    DocumentSection::SessionInfo,
                    DocumentSection::TableOfContents,
                    DocumentSection::Commands,
                    DocumentSection::Annotations,
                    DocumentSection::Footer,
                ],
                include_breadcrumbs: false,
                use_emoji_indicators: false,
                include_performance_metrics: false,
                markdown_extensions: vec![
                    MarkdownExtension::Tables,
                    MarkdownExtension::SyntaxHighlighting,
                    MarkdownExtension::TaskLists,
                    MarkdownExtension::Strikethrough,
                ],
                verbosity_level: VerbosityLevel::Standard,
                include_command_relationships: false,
                use_compact_formatting: false,
            },
            code_block_config: CodeBlockConfig::default(),
            ai_analysis_config: AIAnalysisConfig::default(),
        }
    }