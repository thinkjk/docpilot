pub mod markdown;
pub mod codeblock;

#[cfg(test)]
#[path = "markdown.test.rs"]
mod markdown_test;

#[cfg(test)]
#[path = "markdown_formatting_demo.test.rs"]
mod markdown_formatting_demo_test;

pub use markdown::{MarkdownGenerator, MarkdownTemplate, MarkdownConfig};
pub use codeblock::{CodeBlockGenerator, CodeBlockConfig, CodeBlock, CodeBlockType};

use anyhow::Result;
use crate::session::manager::Session;
use std::path::Path;

/// Generate documentation from a session and save to file
pub async fn generate_documentation(session: &Session, output_path: &Path, template: &str) -> Result<()> {
    // Create markdown generator based on template
    let generator = match template.to_lowercase().as_str() {
        "minimal" => MarkdownGenerator::with_config(MarkdownGenerator::minimal_config()),
        "comprehensive" => MarkdownGenerator::with_config(MarkdownGenerator::comprehensive_config()),
        "hierarchical" => MarkdownGenerator::with_config(MarkdownGenerator::hierarchical_config()),
        "professional" => MarkdownGenerator::with_config(MarkdownGenerator::professional_config()),
        "compact" => MarkdownGenerator::with_config(MarkdownGenerator::compact_config()),
        "rich" => MarkdownGenerator::with_config(MarkdownGenerator::rich_config()),
        "technical" => MarkdownGenerator::with_config(MarkdownGenerator::technical_config()),
        "github" => MarkdownGenerator::with_config(MarkdownGenerator::github_config()),
        "ai-enhanced" => MarkdownGenerator::with_config(MarkdownGenerator::ai_enhanced_config()),
        _ => MarkdownGenerator::new(), // Default to standard configuration
    };

    // Generate and save documentation
    generator.generate_to_file(session, output_path).await?;
    
    Ok(())
}