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
    let mut generator = match template.to_lowercase().as_str() {
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

    // Check if AI features should be enabled and LLM is configured
    if should_enable_ai(&generator, template) {
        if let Ok(llm_config) = crate::llm::LlmConfig::load() {
            if llm_config.is_configured() {
                println!("🤖 AI analysis enabled - generating enhanced documentation...");
                generator.enable_ai_analysis(llm_config);
            } else {
                println!("⚠️  AI template requested but no LLM provider configured.");
                println!("   Use 'docpilot config --provider <provider> --api-key <key>' to set up AI features.");
                println!("   Generating documentation without AI analysis...");
            }
        }
    }

    // Generate and save documentation
    generator.generate_to_file(session, output_path).await?;
    
    Ok(())
}

/// Check if AI features should be enabled based on template and configuration
fn should_enable_ai(generator: &MarkdownGenerator, template: &str) -> bool {
    // Enable AI for ai-enhanced template or if explicitly configured
    template.to_lowercase() == "ai-enhanced" ||
    generator.get_config().ai_analysis_config.enable_ai_explanations
}