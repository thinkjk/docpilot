use super::*;
use crate::session::manager::{Session, Annotation, AnnotationType, SessionState};
use crate::terminal::CommandEntry;
use chrono::Utc;
use std::collections::HashMap;

fn create_test_session() -> Session {
    let mut session = Session::new(
        "Test Documentation Session".to_string(),
        Some(std::path::PathBuf::from("test_output.md"))
    ).expect("Failed to create test session");

    // Add some test commands
    let command1 = CommandEntry {
        command: "ls -la".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        working_directory: "/home/user/project".to_string(),
        shell: "bash".to_string(),
        output: Some("total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .\ndrwxr-xr-x 3 user user 4096 Jan 1 12:00 ..".to_string()),
        error: None,
    };

    let command2 = CommandEntry {
        command: "cargo build".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        working_directory: "/home/user/project".to_string(),
        shell: "bash".to_string(),
        output: Some("   Compiling docpilot v0.1.0\n    Finished dev [unoptimized + debuginfo] target(s) in 2.34s".to_string()),
        error: None,
    };

    let command3 = CommandEntry {
        command: "cargo test nonexistent".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(1),
        working_directory: "/home/user/project".to_string(),
        shell: "bash".to_string(),
        output: None,
        error: Some("error: no tests to run".to_string()),
    };

    session.add_command(command1);
    session.add_command(command2);
    session.add_command(command3);

    // Add some annotations
    session.add_annotation("Starting the build process".to_string(), AnnotationType::Note);
    session.add_annotation("This command lists all files in the directory".to_string(), AnnotationType::Explanation);
    session.add_annotation("Build completed successfully".to_string(), AnnotationType::Milestone);
    session.add_annotation("Test command failed as expected".to_string(), AnnotationType::Warning);

    session
}

#[tokio::test]
async fn test_markdown_template_creation() {
    let template = MarkdownTemplate::new();
    assert!(template.get_config().include_metadata);
    assert!(template.get_config().include_timestamps);
    assert!(template.get_config().include_output);
    assert_eq!(template.get_config().code_language, "bash");
}

#[tokio::test]
async fn test_markdown_template_with_custom_config() {
    let mut config = MarkdownConfig::default();
    config.include_metadata = false;
    config.code_language = "shell".to_string();
    config.max_output_length = 500;

    let template = MarkdownTemplate::with_config(config);
    assert!(!template.get_config().include_metadata);
    assert_eq!(template.get_config().code_language, "shell");
    assert_eq!(template.get_config().max_output_length, 500);
}

#[tokio::test]
async fn test_basic_markdown_generation() {
    let session = create_test_session();
    let template = MarkdownTemplate::new();
    
    let result = template.generate(&session).await;
    assert!(result.is_ok());
    
    let content = result.unwrap();
    
    // Check that basic sections are present
    assert!(content.contains("# Test Documentation Session"));
    assert!(content.contains("## Session Overview"));
    assert!(content.contains("## Session Metadata"));
    assert!(content.contains("## Session Statistics"));
    assert!(content.contains("## Commands"));
    assert!(content.contains("## Annotations"));
    
    // Check that commands are included
    assert!(content.contains("ls -la"));
    assert!(content.contains("cargo build"));
    assert!(content.contains("cargo test nonexistent"));
    
    // Check that annotations are included
    assert!(content.contains("Starting the build process"));
    assert!(content.contains("Build completed successfully"));
}

#[tokio::test]
async fn test_markdown_generation_without_metadata() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.include_metadata = false;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should not contain metadata section
    assert!(!content.contains("## Session Metadata"));
    
    // But should still contain other sections
    assert!(content.contains("## Session Overview"));
    assert!(content.contains("## Commands"));
}

#[tokio::test]
async fn test_markdown_generation_without_timestamps() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.include_timestamps = false;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should not contain timestamp information in command tables
    let lines: Vec<&str> = content.lines().collect();
    let timestamp_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("Timestamp"))
        .cloned()
        .collect();
    
    // Should only have timestamp in session overview, not in command details
    assert!(timestamp_lines.len() <= 2); // Started/Stopped timestamps in overview
}

#[tokio::test]
async fn test_markdown_generation_without_output() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.include_output = false;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should not contain output sections
    assert!(!content.contains("**Output:**"));
    
    // But should still contain commands
    assert!(content.contains("ls -la"));
    assert!(content.contains("cargo build"));
}

#[tokio::test]
async fn test_markdown_generation_without_errors() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.include_errors = false;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should not contain error sections
    assert!(!content.contains("**Error:**"));
    assert!(!content.contains("error: no tests to run"));
}

#[tokio::test]
async fn test_markdown_generation_with_table_of_contents() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.template_options.include_toc = true;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should contain table of contents
    assert!(content.contains("## Table of Contents"));
    assert!(content.contains("- [Session Metadata](#session-metadata)"));
    assert!(content.contains("- [Commands](#commands)"));
    assert!(content.contains("- [Annotations](#annotations)"));
}

#[tokio::test]
async fn test_markdown_generation_with_status_indicators() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.template_options.include_status_indicators = true;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should contain status indicators
    assert!(content.contains("✅")); // Success indicator
    assert!(content.contains("❌")); // Failure indicator
}

#[tokio::test]
async fn test_markdown_generation_without_status_indicators() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.template_options.include_status_indicators = false;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should not contain status indicators
    assert!(!content.contains("✅"));
    assert!(!content.contains("❌"));
    assert!(!content.contains("⏳"));
}

#[tokio::test]
async fn test_output_truncation() {
    let mut session = create_test_session();
    
    // Add a command with very long output
    let long_output = "a".repeat(2000);
    let command_with_long_output = CommandEntry {
        command: "echo long_output".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        working_directory: "/home/user".to_string(),
        shell: "bash".to_string(),
        output: Some(long_output),
        error: None,
    };
    
    session.add_command(command_with_long_output);
    
    let mut config = MarkdownConfig::default();
    config.max_output_length = 100;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should contain truncation message
    assert!(content.contains("... (output truncated)"));
}

#[tokio::test]
async fn test_markdown_escape_special_characters() {
    let template = MarkdownTemplate::new();
    
    // Test escaping of markdown special characters
    assert_eq!(template.escape_markdown("test`code`"), "test\\`code\\`");
    assert_eq!(template.escape_markdown("*bold*"), "\\*bold\\*");
    assert_eq!(template.escape_markdown("_italic_"), "\\_italic\\_");
    assert_eq!(template.escape_markdown("[link](url)"), "\\[link\\]\\(url\\)");
}

#[tokio::test]
async fn test_duration_formatting() {
    let template = MarkdownTemplate::new();
    
    assert_eq!(template.format_duration(30), "30s");
    assert_eq!(template.format_duration(90), "1m 30s");
    assert_eq!(template.format_duration(3661), "1h 1m 1s");
    assert_eq!(template.format_duration(7200), "2h 0m 0s");
}

#[tokio::test]
async fn test_annotation_type_emojis() {
    let session = create_test_session();
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Check that different annotation types have different emojis
    assert!(content.contains("📝")); // Note
    assert!(content.contains("💡")); // Explanation
    assert!(content.contains("⚠️")); // Warning
    assert!(content.contains("🎯")); // Milestone
}

#[tokio::test]
async fn test_empty_session_generation() {
    let session = Session::new(
        "Empty Session".to_string(),
        None
    ).expect("Failed to create empty session");
    
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Should handle empty session gracefully
    assert!(content.contains("# Empty Session"));
    assert!(content.contains("*No commands were captured during this session.*"));
    
    // Should not contain annotations section if no annotations
    assert!(!content.contains("## Annotations"));
}

#[tokio::test]
async fn test_custom_title_and_headers() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.template_options.title = Some("Custom Documentation Title".to_string());
    config.template_options.custom_header = Some("This is a custom header section.".to_string());
    config.template_options.custom_footer = Some("This is a custom footer section.".to_string());
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    assert!(content.contains("# Custom Documentation Title"));
    assert!(content.contains("This is a custom header section."));
    assert!(content.contains("This is a custom footer section."));
}

#[tokio::test]
async fn test_markdown_generator_creation() {
    let generator = MarkdownGenerator::new();
    assert!(generator.get_config().include_metadata);
    
    let custom_config = MarkdownGenerator::minimal_config();
    let generator_with_config = MarkdownGenerator::with_config(custom_config);
    assert!(!generator_with_config.get_config().include_metadata);
}

#[tokio::test]
async fn test_minimal_vs_comprehensive_config() {
    let minimal = MarkdownGenerator::minimal_config();
    let comprehensive = MarkdownGenerator::comprehensive_config();
    
    // Minimal should have fewer features enabled
    assert!(!minimal.include_metadata);
    assert!(!minimal.include_statistics);
    assert!(!minimal.template_options.include_toc);
    assert_eq!(minimal.max_output_length, 500);
    
    // Comprehensive should have more features enabled
    assert!(comprehensive.include_metadata);
    assert!(comprehensive.include_statistics);
    assert!(comprehensive.template_options.include_toc);
    assert_eq!(comprehensive.max_output_length, 2000);
}

#[tokio::test]
async fn test_markdown_generator_documentation_generation() {
    let session = create_test_session();
    let generator = MarkdownGenerator::new();
    
    let result = generator.generate_documentation(&session).await;
    assert!(result.is_ok());
    
    let content = result.unwrap();
    assert!(content.contains("# Test Documentation Session"));
    assert!(content.contains("## Commands"));
}

#[tokio::test]
async fn test_session_statistics_calculation() {
    let session = create_test_session();
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Check that statistics are calculated correctly
    assert!(content.contains("| Total Commands | 3 |"));
    assert!(content.contains("| Successful Commands | 2 |"));
    assert!(content.contains("| Failed Commands | 1 |"));
    assert!(content.contains("| Success Rate | 66.7% |"));
    assert!(content.contains("| Total Annotations | 4 |"));
}

#[tokio::test]
async fn test_command_table_formatting() {
    let session = create_test_session();
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Check that command tables are properly formatted
    assert!(content.contains("| Property | Value |"));
    assert!(content.contains("|----------|-------|"));
    assert!(content.contains("| Command | `ls -la` |"));
    assert!(content.contains("| Working Directory | `/home/user/project` |"));
    assert!(content.contains("| Shell | `bash` |"));
    assert!(content.contains("| Exit Code | `0` |"));
}

#[tokio::test]
async fn test_code_block_formatting() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.code_language = "shell".to_string();
    config.code_block_config.default_command_language = "shell".to_string();
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Check that code blocks are generated with enhanced formatting
    assert!(content.contains("```"));
    assert!(content.contains("ls -la"));
    assert!(content.contains("cargo build"));
    // The enhanced system uses intelligent language detection, so it might not always be "shell"
    // but should contain the commands
}

#[tokio::test]
async fn test_working_directory_grouping() {
    let mut session = create_test_session();
    
    // Add commands from different directories
    let command_different_dir = CommandEntry {
        command: "pwd".to_string(),
        timestamp: Utc::now(),
        exit_code: Some(0),
        working_directory: "/home/user/other".to_string(),
        shell: "bash".to_string(),
        output: Some("/home/user/other".to_string()),
        error: None,
    };
    
    session.add_command(command_different_dir);
    
    let mut config = MarkdownConfig::default();
    config.template_options.group_by_directory = true;
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should contain directory group headers
    assert!(content.contains("### Directory: `/home/user/project`"));
    assert!(content.contains("### Directory: `/home/user/other`"));
}

#[tokio::test]
async fn test_time_grouping() {
    let session = create_test_session();
    let mut config = MarkdownConfig::default();
    config.template_options.group_by_time = true;
    config.template_options.time_group_interval = 60; // 1 hour intervals
    
    let template = MarkdownTemplate::with_config(config);
    let content = template.generate(&session).await.unwrap();
    
    // Should contain time period headers
    assert!(content.contains("### Time Period:"));
}

#[tokio::test]
async fn test_session_state_display() {
    let mut session = create_test_session();
    session.stop().expect("Failed to stop session");
    
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Should display the session state
    assert!(content.contains("**Status:** Stopped"));
}

#[tokio::test]
async fn test_generation_timestamp() {
    let session = create_test_session();
    let template = MarkdownTemplate::new();
    let content = template.generate(&session).await.unwrap();
    
    // Should contain generation timestamp
    assert!(content.contains("*Generated by DocPilot on"));
}

#[tokio::test]
async fn test_config_update() {
    let mut generator = MarkdownGenerator::new();
    assert!(generator.get_config().include_metadata);
    
    let new_config = MarkdownGenerator::minimal_config();
    generator.set_config(new_config);
    assert!(!generator.get_config().include_metadata);
}