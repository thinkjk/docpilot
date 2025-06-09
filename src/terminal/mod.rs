pub mod monitor;
pub mod platform;

#[cfg(test)]
#[path = "monitor.test.rs"]
mod monitor_test;

pub use monitor::{TerminalMonitor, CommandEntry, ShellType};
pub use platform::{Platform, PlatformUtils};