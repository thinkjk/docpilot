#[cfg(test)]
mod tests {
    use super::super::monitor::*;
    use super::super::platform::*;
    use chrono::Utc;
    use std::env;

    #[test]
    fn test_shell_type_detection() {
        let shell = ShellType::detect();
        match shell {
            ShellType::Bash => assert_eq!(shell.name(), "bash"),
            ShellType::Zsh => assert_eq!(shell.name(), "zsh"),
            ShellType::Fish => assert_eq!(shell.name(), "fish"),
            ShellType::Unknown(name) => assert!(!name.is_empty()),
        }
    }

    #[test]
    fn test_shell_history_file_paths() {
        let shells = vec![
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
        ];

        for shell in shells {
            if let Some(history_path) = shell.history_file() {
                assert!(history_path.to_string_lossy().contains("history"));
            }
        }
    }

    #[test]
    fn test_command_entry_creation() {
        let entry = CommandEntry {
            command: "ls -la".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            output: Some("file1\nfile2".to_string()),
            error: None,
        };

        assert_eq!(entry.command, "ls -la");
        assert_eq!(entry.exit_code, Some(0));
        assert_eq!(entry.working_directory, "/home/user");
        assert_eq!(entry.shell, "bash");
        assert!(entry.output.is_some());
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_terminal_monitor_creation_on_supported_platform() {
        // This test will only pass on supported platforms (Linux/macOS)
        match TerminalMonitor::new("test-session".to_string()) {
            Ok(monitor) => {
                assert_eq!(monitor.session_id, "test-session");
                assert!(!monitor.is_monitoring());
                assert_eq!(monitor.get_commands().len(), 0);
                assert!(matches!(monitor.platform, Platform::Linux | Platform::MacOS));
            }
            Err(_) => {
                // Expected on unsupported platforms
                let platform = Platform::detect();
                assert!(matches!(platform, Platform::Unknown(_)));
            }
        }
    }

    #[test]
    fn test_command_filtering_logic() {
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            // Commands that should be ignored
            let ignore_commands = vec![
                "ls",
                "pwd",
                "cd /home",
                "clear",
                "history",
                "exit",
                "echo hello",
                "cat file.txt",
                "less file.txt",
                "more file.txt",
                "head file.txt",
                "tail file.txt",
            ];

            for cmd in ignore_commands {
                assert!(monitor.should_ignore_command(cmd), 
                       "Command '{}' should be ignored", cmd);
            }

            // Commands that should NOT be ignored
            let keep_commands = vec![
                "cargo build",
                "git commit -m 'test'",
                "npm install",
                "docker run ubuntu",
                "python script.py",
                "make install",
                "curl -X POST https://api.example.com",
                "ssh user@server",
            ];

            for cmd in keep_commands {
                assert!(!monitor.should_ignore_command(cmd), 
                       "Command '{}' should NOT be ignored", cmd);
            }
        }
    }

    #[test]
    fn test_bash_history_parsing() {
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            // Test bash history line parsing
            let bash_lines = vec![
                "ls -la",
                "cd /home/user",
                "git status",
                "",  // empty line should be ignored
                "   ",  // whitespace only should be ignored
            ];

            let mut valid_commands = 0;
            for line in bash_lines {
                if let Some(entry) = monitor.parse_history_line(line) {
                    assert!(!entry.command.is_empty());
                    assert_eq!(entry.shell, monitor.shell_type.name());
                    valid_commands += 1;
                }
            }

            // Should have parsed some valid commands (exact number depends on filtering)
            assert!(valid_commands > 0);
        }
    }

    #[test]
    fn test_zsh_history_parsing() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            // Simulate zsh shell
            monitor.shell_type = ShellType::Zsh;

            // Test zsh history format: ": timestamp:duration;command"
            let zsh_lines = vec![
                ": 1640995200:0;git status",
                ": 1640995201:5;cargo build",
                "regular command without timestamp",
                ": 1640995202:0;ls",  // should be filtered out
            ];

            let mut valid_commands = 0;
            for line in zsh_lines {
                if let Some(entry) = monitor.parse_history_line(line) {
                    assert!(!entry.command.is_empty());
                    assert_eq!(entry.shell, "zsh");
                    valid_commands += 1;
                }
            }

            assert!(valid_commands > 0);
        }
    }

    #[test]
    fn test_fish_history_parsing() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            // Simulate fish shell
            monitor.shell_type = ShellType::Fish;

            // Test fish history format
            let fish_lines = vec![
                "- cmd: git status",
                "- cmd: cargo test",
                "- when: 1640995200",  // metadata line, should be ignored
                "- cmd: ls",  // should be filtered out
            ];

            let mut valid_commands = 0;
            for line in fish_lines {
                if let Some(entry) = monitor.parse_history_line(line) {
                    assert!(!entry.command.is_empty());
                    assert_eq!(entry.shell, "fish");
                    valid_commands += 1;
                }
            }

            assert!(valid_commands > 0);
        }
    }

    #[test]
    fn test_monitor_state_management() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            // Initial state
            assert!(!monitor.is_monitoring());

            // Start monitoring
            assert!(monitor.start_monitoring().is_ok());
            assert!(monitor.is_monitoring());

            // Try to start again (should fail)
            assert!(monitor.start_monitoring().is_err());

            // Stop monitoring
            assert!(monitor.stop_monitoring().is_ok());
            assert!(!monitor.is_monitoring());

            // Try to stop again (should fail)
            assert!(monitor.stop_monitoring().is_err());
        }
    }

    #[test]
    fn test_command_addition() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            assert_eq!(monitor.get_commands().len(), 0);

            let entry = CommandEntry {
                command: "test command".to_string(),
                timestamp: Utc::now(),
                exit_code: Some(0),
                working_directory: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                shell: "bash".to_string(),
                output: None,
                error: None,
            };

            monitor.add_command(entry);
            assert_eq!(monitor.get_commands().len(), 1);
            assert_eq!(monitor.get_commands()[0].command, "test command");
        }
    }

    #[tokio::test]
    async fn test_execute_and_capture() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            // Test executing a simple command
            let result = monitor.execute_and_capture("echo 'hello world'").await;
            
            match result {
                Ok(entry) => {
                    assert_eq!(entry.command, "echo 'hello world'");
                    assert!(entry.output.is_some());
                    assert!(entry.output.unwrap().contains("hello world"));
                    assert_eq!(entry.exit_code, Some(0));
                }
                Err(e) => {
                    // Command execution might fail in some test environments
                    println!("Command execution failed (expected in some environments): {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_execute_and_capture_with_error() {
        if let Ok(mut monitor) = TerminalMonitor::new("test".to_string()) {
            // Test executing a command that should fail
            let result = monitor.execute_and_capture("nonexistent_command_12345").await;
            
            match result {
                Ok(entry) => {
                    assert_eq!(entry.command, "nonexistent_command_12345");
                    assert!(entry.exit_code != Some(0)); // Should have non-zero exit code
                    assert!(entry.error.is_some() || entry.output.is_some());
                }
                Err(_) => {
                    // Command execution failure is also acceptable
                }
            }
        }
    }

    #[test]
    fn test_working_directory_capture() {
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            // Create a test command entry
            let entry = CommandEntry {
                command: "pwd".to_string(),
                timestamp: Utc::now(),
                exit_code: Some(0),
                working_directory: env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                shell: monitor.shell_type.name().to_string(),
                output: None,
                error: None,
            };

            assert!(!entry.working_directory.is_empty());
            assert_ne!(entry.working_directory, "unknown");
        }
    }

    #[test]
    fn test_timestamp_accuracy() {
        if let Ok(monitor) = TerminalMonitor::new("test".to_string()) {
            let before = Utc::now();
            
            let entry = CommandEntry {
                command: "test".to_string(),
                timestamp: Utc::now(),
                exit_code: Some(0),
                working_directory: "/test".to_string(),
                shell: monitor.shell_type.name().to_string(),
                output: None,
                error: None,
            };
            
            let after = Utc::now();

            assert!(entry.timestamp >= before);
            assert!(entry.timestamp <= after);
        }
    }
}