use anyhow::{Result, anyhow};
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    Unknown(String),
}

impl Platform {
    pub fn detect() -> Self {
        match env::consts::OS {
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            other => Platform::Unknown(other.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Unknown(name) => name,
        }
    }

    /// Get the default shell for the platform
    pub fn default_shell(&self) -> &str {
        match self {
            Platform::Linux => "bash",
            Platform::MacOS => "zsh", // macOS default since Catalina
            Platform::Unknown(_) => "sh",
        }
    }

    /// Get platform-specific terminal application paths
    pub fn terminal_apps(&self) -> Vec<&str> {
        match self {
            Platform::Linux => vec![
                "gnome-terminal",
                "konsole",
                "xterm",
                "alacritty",
                "kitty",
                "terminator",
            ],
            Platform::MacOS => vec![
                "Terminal.app",
                "iTerm.app",
                "Alacritty.app",
                "Kitty.app",
                "Hyper.app",
            ],
            Platform::Unknown(_) => vec!["xterm"],
        }
    }

    /// Get platform-specific process monitoring commands
    pub fn process_monitor_cmd(&self) -> (&str, Vec<&str>) {
        match self {
            Platform::Linux => ("ps", vec!["aux"]),
            Platform::MacOS => ("ps", vec!["aux"]),
            Platform::Unknown(_) => ("ps", vec!["aux"]),
        }
    }

    /// Get platform-specific shell configuration files
    pub fn shell_config_files(&self) -> Vec<PathBuf> {
        let home = match env::var("HOME") {
            Ok(h) => PathBuf::from(h),
            Err(_) => return vec![],
        };

        let mut configs = vec![
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".zshrc"),
            home.join(".profile"),
        ];

        match self {
            Platform::Linux => {
                configs.extend(vec![
                    home.join(".config/fish/config.fish"),
                    PathBuf::from("/etc/bash.bashrc"),
                    PathBuf::from("/etc/profile"),
                ]);
            }
            Platform::MacOS => {
                configs.extend(vec![
                    home.join(".config/fish/config.fish"),
                    PathBuf::from("/etc/bashrc"),
                    PathBuf::from("/etc/profile"),
                    PathBuf::from("/etc/zshrc"),
                ]);
            }
            Platform::Unknown(_) => {}
        }

        configs
    }

    /// Check if we have the necessary permissions for terminal monitoring
    pub fn check_permissions(&self) -> Result<()> {
        match self {
            Platform::Linux => {
                // Check if we can read /proc for process monitoring
                if !PathBuf::from("/proc").exists() {
                    return Err(anyhow!("Cannot access /proc filesystem for process monitoring"));
                }
                
                // Real-time monitoring only - no shell history file access needed
                println!("Initializing real-time monitoring for Linux");
                Ok(())
            }
            Platform::MacOS => {
                // Check if we can use system APIs
                let output = Command::new("ps")
                    .arg("aux")
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!("Cannot execute process monitoring commands"));
                }
                
                // Real-time monitoring only - no shell history file access needed
                println!("Initializing real-time monitoring for macOS");
                Ok(())
            }
            Platform::Unknown(os) => {
                Err(anyhow!("Unsupported platform: {}", os))
            }
        }
    }

    /// Get platform-specific terminal session detection method
    pub fn detect_terminal_session(&self) -> Result<Option<String>> {
        match self {
            Platform::Linux => {
                // Check common environment variables
                if let Ok(term) = env::var("TERM") {
                    if term != "dumb" {
                        return Ok(Some(term));
                    }
                }
                
                // Check if we're in a known terminal
                for var in &["GNOME_TERMINAL_SCREEN", "KONSOLE_VERSION", "ITERM_SESSION_ID"] {
                    if env::var(var).is_ok() {
                        return Ok(Some(var.to_string()));
                    }
                }
                
                Ok(None)
            }
            Platform::MacOS => {
                // Check for macOS-specific terminal indicators
                if let Ok(term_program) = env::var("TERM_PROGRAM") {
                    return Ok(Some(term_program));
                }
                
                if let Ok(iterm) = env::var("ITERM_SESSION_ID") {
                    return Ok(Some(format!("iTerm2:{}", iterm)));
                }
                
                if let Ok(term) = env::var("TERM") {
                    if term != "dumb" {
                        return Ok(Some(term));
                    }
                }
                
                Ok(None)
            }
            Platform::Unknown(_) => Ok(None),
        }
    }

    /// Get platform-specific installation instructions
    pub fn installation_method(&self) -> &str {
        match self {
            Platform::Linux => "Package manager (apt, yum, pacman) or cargo install",
            Platform::MacOS => "Homebrew: brew install docpilot, or cargo install",
            Platform::Unknown(_) => "cargo install",
        }
    }

    /// Check if the platform supports advanced terminal features
    pub fn supports_advanced_monitoring(&self) -> bool {
        match self {
            Platform::Linux | Platform::MacOS => true,
            Platform::Unknown(_) => false,
        }
    }
}

/// Platform-specific utilities for terminal integration
pub struct PlatformUtils;

impl PlatformUtils {
    /// Get the current platform
    pub fn current_platform() -> Platform {
        Platform::detect()
    }

    /// Initialize platform-specific monitoring
    pub fn initialize_monitoring() -> Result<()> {
        let platform = Platform::detect();
        
        println!("Initializing terminal monitoring for: {}", platform.name());
        
        // Check permissions
        platform.check_permissions()?;
        
        // Detect terminal session
        if let Some(session) = platform.detect_terminal_session()? {
            println!("Detected terminal session: {}", session);
        } else {
            println!("Warning: Could not detect terminal session");
        }
        
        if !platform.supports_advanced_monitoring() {
            return Err(anyhow!("Platform {} does not support advanced monitoring", platform.name()));
        }
        
        Ok(())
    }


    /// Check if running in a supported environment
    pub fn is_supported_environment() -> bool {
        let platform = Platform::detect();
        matches!(platform, Platform::Linux | Platform::MacOS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(!platform.name().is_empty());
    }

    #[test]
    fn test_platform_utils() {
        let platform = PlatformUtils::current_platform();
        assert!(!platform.name().is_empty());
        
        // Real-time monitoring test - no history paths needed
        assert!(PlatformUtils::is_supported_environment());
    }

    #[test]
    fn test_shell_config_files() {
        let platform = Platform::detect();
        let configs = platform.shell_config_files();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_terminal_apps() {
        let platform = Platform::detect();
        let apps = platform.terminal_apps();
        assert!(!apps.is_empty());
    }
}