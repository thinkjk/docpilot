#[cfg(test)]
mod tests {
    use crate::output::markdown::{MarkdownGenerator, MarkdownConfig, TemplateOptions, OutputTheme, VerbosityLevel, DocumentSection, MarkdownExtension};
    use crate::session::manager::{Session, AnnotationType};
    use crate::terminal::CommandEntry;
    use chrono::Utc;

    fn create_test_session() -> Session {
        Session::new("Test session".to_string(), None).unwrap()
    }

    fn create_test_command() -> CommandEntry {
        CommandEntry {
            command: "cargo test".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/tmp".to_string(),
            shell: "bash".to_string(),
            output: Some("test result: ok".to_string()),
            error: None,
        }
    }

    #[tokio::test]
    async fn test_formatting_options_creation() {
        // Test that we can create formatting options with new enums
        let options = TemplateOptions {
            title: Some("Test Documentation".to_string()),
            include_toc: true,
            group_by_time: false,
            group_by_directory: false,
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            theme: OutputTheme::Professional,
            verbosity_level: VerbosityLevel::Detailed,
            section_order: vec![
                DocumentSection::TableOfContents,
                DocumentSection::Commands,
                DocumentSection::Annotations,
            ],
            markdown_extensions: vec![MarkdownExtension::Tables, MarkdownExtension::Footnotes],
            use_emoji_indicators: true,
            include_performance_metrics: true,
            include_environment_vars: true,
            include_duration: true,
            custom_header: None,
            custom_footer: None,
            ..Default::default()
        };

        // Test that enums work correctly
        assert!(matches!(options.theme, OutputTheme::Professional));
        assert!(matches!(options.verbosity_level, VerbosityLevel::Detailed));
        assert_eq!(options.section_order.len(), 3);
        assert_eq!(options.markdown_extensions.len(), 2);
    }

    #[tokio::test]
    async fn test_theme_variations() {
        let themes = vec![
            OutputTheme::Professional,
            OutputTheme::Compact,
            OutputTheme::Rich,
            OutputTheme::Technical,
            OutputTheme::GitHub,
            OutputTheme::Minimal,
            OutputTheme::Custom,
        ];

        for theme in themes {
            let theme_clone = theme.clone();
            let config = MarkdownConfig {
                template_options: TemplateOptions {
                    theme,
                    ..Default::default()
                },
                ..Default::default()
            };

            let generator = MarkdownGenerator::with_config(config);
            let session = create_test_session();
            
            // Should be able to generate with any theme
            let result = generator.generate_documentation(&session).await;
            assert!(result.is_ok(), "Failed to generate with theme: {:?}", theme_clone);
        }
    }

    #[tokio::test]
    async fn test_verbosity_levels() {
        let levels = vec![
            VerbosityLevel::Minimal,
            VerbosityLevel::Standard,
            VerbosityLevel::Detailed,
            VerbosityLevel::Verbose,
        ];

        for level in levels {
            let level_clone = level.clone();
            let config = MarkdownConfig {
                template_options: TemplateOptions {
                    verbosity_level: level,
                    ..Default::default()
                },
                ..Default::default()
            };

            let generator = MarkdownGenerator::with_config(config);
            let mut session = create_test_session();
            session.add_command(create_test_command());
            
            let result = generator.generate_documentation(&session).await;
            assert!(result.is_ok(), "Failed to generate with verbosity: {:?}", level_clone);
        }
    }

    #[tokio::test]
    async fn test_configuration_presets() {
        // Test that all the new configuration presets work
        use crate::output::markdown::{professional_config, compact_config, rich_config, technical_config, github_config};
        
        let configs = vec![
            ("professional", professional_config()),
            ("compact", compact_config()),
            ("rich", rich_config()),
            ("technical", technical_config()),
            ("github", github_config()),
        ];

        for (name, config) in configs {
            let generator = MarkdownGenerator::with_config(config);
            let session = create_test_session();
            
            let result = generator.generate_documentation(&session).await;
            assert!(result.is_ok(), "Failed to generate with {} config", name);
            
            let markdown = result.unwrap();
            assert!(!markdown.is_empty(), "{} config produced empty markdown", name);
            assert!(markdown.contains("#"), "{} config should contain headers", name);
        }
    }

    #[tokio::test]
    async fn test_formatting_options_integration() {
        let config = MarkdownConfig {
            template_options: TemplateOptions {
                theme: OutputTheme::Rich,
                verbosity_level: VerbosityLevel::Verbose,
                use_emoji_indicators: true,
                include_performance_metrics: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let generator = MarkdownGenerator::with_config(config);
        let mut session = create_test_session();
        
        // Add some test data
        session.add_command(create_test_command());
        session.add_annotation("Test annotation".to_string(), AnnotationType::Note);
        
        let result = generator.generate_documentation(&session).await;
        assert!(result.is_ok());
        
        let markdown = result.unwrap();
        assert!(!markdown.is_empty());
        
        // Should contain some basic markdown structure
        assert!(markdown.contains("#"));  // Headers
        assert!(markdown.contains("cargo test"));  // Our test command
    }

    #[test]
    fn test_new_enums_exist() {
        // Test that all the new enums and their variants exist
        let _theme = OutputTheme::Professional;
        let _verbosity = VerbosityLevel::Detailed;
        let _section = DocumentSection::Commands;
        let _extension = MarkdownExtension::Tables;
        
        // Test enum methods
        assert!(!format!("{:?}", OutputTheme::Professional).is_empty());
        assert!(!format!("{:?}", VerbosityLevel::Standard).is_empty());
        assert!(!format!("{:?}", DocumentSection::Commands).is_empty());
        assert!(!format!("{:?}", MarkdownExtension::Tables).is_empty());
    }
}