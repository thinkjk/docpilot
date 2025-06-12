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
    // Check if AI features can be enabled (try to load LLM config first)
    let ai_available = if let Ok(llm_config) = crate::llm::LlmConfig::load() {
        llm_config.is_configured()
    } else {
        false
    };

    // Create markdown generator based on template, defaulting to AI-enhanced when available
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
        "standard" => {
            // Standard template now defaults to AI-enhanced when available
            if ai_available {
                println!("🤖 Using AI-enhanced documentation for standard template (LLM configured)");
                MarkdownGenerator::with_config(MarkdownGenerator::ai_enhanced_config())
            } else {
                MarkdownGenerator::new() // Fallback to basic standard
            }
        },
        _ => {
            // Default behavior: use AI-enhanced if available, otherwise standard
            if ai_available {
                println!("🤖 Defaulting to AI-enhanced documentation (LLM configured)");
                MarkdownGenerator::with_config(MarkdownGenerator::ai_enhanced_config())
            } else {
                MarkdownGenerator::new() // Standard configuration
            }
        }
    };

    // Enable AI features if available and should be used
    if should_enable_ai(&generator, template, ai_available) {
        if let Ok(llm_config) = crate::llm::LlmConfig::load() {
            if llm_config.is_configured() {
                println!("🤖 AI analysis enabled - generating enhanced documentation...");
                generator.enable_ai_analysis(llm_config);
                
                // Use AI-enhanced generation for better quality
                match template.to_lowercase().as_str() {
                    "ai-enhanced" | "standard" => {
                        println!("🚀 Generating comprehensive AI-enhanced documentation...");
                        let content = generator.generate_comprehensive_ai_documentation(session).await?;
                        std::fs::write(output_path, content)?;
                        return Ok(());
                    }
                    _ => {
                        println!("🔍 Applying AI post-processing to improve documentation quality...");
                        let content = generator.generate_ai_enhanced_documentation(session).await?;
                        std::fs::write(output_path, content)?;
                        return Ok(());
                    }
                }
            } else {
                println!("⚠️  AI features requested but no LLM provider configured.");
                println!("   Use 'docpilot config --provider <provider> --api-key <key>' to set up AI features.");
                println!("   Generating documentation without AI analysis...");
            }
        } else {
            println!("⚠️  Could not load LLM configuration. Generating documentation without AI analysis...");
        }
    }

    // Generate and save documentation using standard method
    generator.generate_to_file(session, output_path).await?;
    
    Ok(())
}

/// Check if AI features should be enabled based on template and configuration
fn should_enable_ai(generator: &MarkdownGenerator, template: &str, ai_available: bool) -> bool {
    // Enable AI for most templates except minimal and compact (which are explicitly simple)
    let template_lower = template.to_lowercase();
    match template_lower.as_str() {
        "minimal" | "compact" => false, // Explicitly simple templates - never use AI
        "standard" => ai_available, // Standard uses AI when available
        _ => ai_available, // All other templates use AI when available
    }
}