use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use crate::llm::error_handler::{ErrorHandler, LlmError, RetryConfig};

#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    Claude,
    ChatGpt,
    Gemini,
    Ollama,
}

impl LlmProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(LlmProvider::Claude),
            "chatgpt" | "openai" => Ok(LlmProvider::ChatGpt),
            "gemini" | "google" => Ok(LlmProvider::Gemini),
            "ollama" | "local" => Ok(LlmProvider::Ollama),
            _ => Err(anyhow!("Unsupported LLM provider: {}", s)),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            LlmProvider::Claude => "claude",
            LlmProvider::ChatGpt => "chatgpt",
            LlmProvider::Gemini => "gemini",
            LlmProvider::Ollama => "ollama",
        }
    }

    pub fn api_base_url(&self) -> &str {
        match self {
            LlmProvider::Claude => "https://api.anthropic.com/v1",
            LlmProvider::ChatGpt => "https://api.openai.com/v1",
            LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            LlmProvider::Ollama => "http://localhost:11434/api",
        }
    }

    pub fn default_model(&self) -> &str {
        match self {
            LlmProvider::Claude => "claude-3-5-sonnet-20241022",
            LlmProvider::ChatGpt => "gpt-4",
            LlmProvider::Gemini => "gemini-pro",
            LlmProvider::Ollama => "llama2", // Default Ollama model
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<Usage>,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct LlmClient {
    provider: LlmProvider,
    api_key: String,
    client: Client,
    model: String,
    error_handler: std::sync::Mutex<ErrorHandler>,
}

impl LlmClient {
    pub fn new(provider: LlmProvider, api_key: String) -> Result<Self> {
        if api_key.trim().is_empty() {
            return Err(anyhow!("API key cannot be empty"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let model = provider.default_model().to_string();

        let retry_config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let error_handler = std::sync::Mutex::new(ErrorHandler::new(retry_config));

        Ok(Self {
            provider,
            api_key,
            client,
            model,
            error_handler,
        })
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn error_handler(&self) -> &std::sync::Mutex<ErrorHandler> {
        &self.error_handler
    }

    pub fn provider(&self) -> &LlmProvider {
        &self.provider
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        let mut operation = || async {
            let result = match self.provider {
                LlmProvider::Claude => self.generate_claude_internal(request.clone()).await,
                LlmProvider::ChatGpt => self.generate_chatgpt_internal(request.clone()).await,
                LlmProvider::Gemini => self.generate_gemini_internal(request.clone()).await,
                LlmProvider::Ollama => self.generate_ollama_internal(request.clone()).await,
            };

            // Convert anyhow::Error to LlmError for error handler
            result.map_err(|e| {
                let error_str = e.to_string();
                let provider_name = self.provider.name().to_string();
                
                if error_str.contains("rate limit") || error_str.contains("429") {
                    LlmError::RateLimited {
                        provider: provider_name,
                        message: error_str,
                        retry_after: Some(Duration::from_secs(60))
                    }
                } else if error_str.contains("network") || error_str.contains("connection") {
                    LlmError::NetworkError {
                        provider: provider_name,
                        error: error_str,
                        retryable: true,
                    }
                } else if error_str.contains("parse") || error_str.contains("json") {
                    LlmError::ParseError {
                        provider: provider_name,
                        message: error_str,
                    }
                } else {
                    LlmError::ApiError {
                        provider: provider_name,
                        message: error_str,
                    }
                }
            })
        };

        let mut handler = self.error_handler.lock().unwrap();
        handler.execute_with_retry(operation).await
            .map_err(|e| anyhow!("LLM request failed: {}", e))
    }

    async fn generate_claude_internal(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/messages", self.provider.api_base_url());
        
        // Add user message (only user messages in the messages array for Claude)
        let messages = vec![json!({
            "role": "user",
            "content": request.prompt
        })];

        let mut payload = json!({
            "model": self.model,
            "max_tokens": request.max_tokens.unwrap_or(1000),
            "temperature": request.temperature.unwrap_or(0.7),
            "messages": messages
        });

        // Add system prompt as top-level parameter if provided
        if let Some(system) = &request.system_prompt {
            payload["system"] = json!(system);
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Claude API error: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Claude response format"))?
            .to_string();

        let usage = if let Some(usage_data) = response_json.get("usage") {
            Some(Usage {
                prompt_tokens: usage_data["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage_data["output_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: (usage_data["input_tokens"].as_u64().unwrap_or(0) + 
                              usage_data["output_tokens"].as_u64().unwrap_or(0)) as u32,
            })
        } else {
            None
        };

        Ok(LlmResponse {
            content,
            usage,
            model: self.model.clone(),
            provider: self.provider.name().to_string(),
        })
    }

    async fn generate_chatgpt_internal(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.provider.api_base_url());
        
        let mut messages = Vec::new();
        
        // Add system message if provided
        if let Some(system) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system
            }));
        }
        
        // Add user message
        messages.push(json!({
            "role": "user",
            "content": request.prompt
        }));

        let payload = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(1000),
            "temperature": request.temperature.unwrap_or(0.7)
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("ChatGPT API error: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid ChatGPT response format"))?
            .to_string();

        let usage = if let Some(usage_data) = response_json.get("usage") {
            Some(Usage {
                prompt_tokens: usage_data["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage_data["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage_data["total_tokens"].as_u64().unwrap_or(0) as u32,
            })
        } else {
            None
        };

        Ok(LlmResponse {
            content,
            usage,
            model: self.model.clone(),
            provider: self.provider.name().to_string(),
        })
    }

    async fn generate_gemini_internal(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/models/{}:generateContent?key={}", 
                         self.provider.api_base_url(), 
                         self.model, 
                         self.api_key);
        
        let mut parts = Vec::new();
        
        // Add system prompt if provided
        if let Some(system) = &request.system_prompt {
            parts.push(json!({
                "text": format!("System: {}\n\nUser: {}", system, request.prompt)
            }));
        } else {
            parts.push(json!({
                "text": request.prompt
            }));
        }

        let payload = json!({
            "contents": [{
                "parts": parts
            }],
            "generationConfig": {
                "maxOutputTokens": request.max_tokens.unwrap_or(1000),
                "temperature": request.temperature.unwrap_or(0.7)
            }
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Gemini response format"))?
            .to_string();

        // Gemini doesn't provide detailed usage stats in the same format
        let usage = if let Some(usage_data) = response_json.get("usageMetadata") {
            Some(Usage {
                prompt_tokens: usage_data["promptTokenCount"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage_data["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage_data["totalTokenCount"].as_u64().unwrap_or(0) as u32,
            })
        } else {
            None
        };

        Ok(LlmResponse {
            content,
            usage,
            model: self.model.clone(),
            provider: self.provider.name().to_string(),
        })
    }

    async fn generate_ollama_internal(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/generate", self.provider.api_base_url());
        
        // Combine system prompt and user prompt for Ollama
        let prompt = if let Some(system) = &request.system_prompt {
            format!("System: {}\n\nUser: {}", system, request.prompt)
        } else {
            request.prompt.clone()
        };

        let payload = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": request.max_tokens.unwrap_or(1000),
                "temperature": request.temperature.unwrap_or(0.7)
            }
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama API error: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        
        let content = response_json["response"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Ollama response format"))?
            .to_string();

        // Ollama provides some usage information
        let usage = if let Some(prompt_eval_count) = response_json.get("prompt_eval_count") {
            let prompt_tokens = prompt_eval_count.as_u64().unwrap_or(0) as u32;
            let completion_tokens = response_json["eval_count"].as_u64().unwrap_or(0) as u32;
            
            Some(Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        } else {
            None
        };

        Ok(LlmResponse {
            content,
            usage,
            model: self.model.clone(),
            provider: self.provider.name().to_string(),
        })
    }

    /// Test the connection to the LLM provider
    pub async fn test_connection(&self) -> Result<()> {
        let test_request = LlmRequest {
            prompt: "Hello, this is a test. Please respond with 'OK'.".to_string(),
            max_tokens: Some(10),
            temperature: Some(0.1),
            system_prompt: None,
        };

        let response = self.generate(test_request).await?;
        
        if response.content.trim().is_empty() {
            return Err(anyhow!("Empty response from LLM provider"));
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_string() {
        assert_eq!(LlmProvider::from_str("claude").unwrap(), LlmProvider::Claude);
        assert_eq!(LlmProvider::from_str("chatgpt").unwrap(), LlmProvider::ChatGpt);
        assert_eq!(LlmProvider::from_str("openai").unwrap(), LlmProvider::ChatGpt);
        assert_eq!(LlmProvider::from_str("gemini").unwrap(), LlmProvider::Gemini);
        assert_eq!(LlmProvider::from_str("google").unwrap(), LlmProvider::Gemini);
        assert_eq!(LlmProvider::from_str("ollama").unwrap(), LlmProvider::Ollama);
        assert_eq!(LlmProvider::from_str("local").unwrap(), LlmProvider::Ollama);
        
        assert!(LlmProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_provider_properties() {
        let claude = LlmProvider::Claude;
        assert_eq!(claude.name(), "claude");
        assert!(claude.api_base_url().contains("anthropic"));
        assert!(!claude.default_model().is_empty());

        let chatgpt = LlmProvider::ChatGpt;
        assert_eq!(chatgpt.name(), "chatgpt");
        assert!(chatgpt.api_base_url().contains("openai"));
        assert!(!chatgpt.default_model().is_empty());

        let gemini = LlmProvider::Gemini;
        assert_eq!(gemini.name(), "gemini");
        assert!(gemini.api_base_url().contains("googleapis"));
        assert!(!gemini.default_model().is_empty());

        let ollama = LlmProvider::Ollama;
        assert_eq!(ollama.name(), "ollama");
        assert!(ollama.api_base_url().contains("localhost"));
        assert!(!ollama.default_model().is_empty());
    }

    #[test]
    fn test_llm_client_creation() {
        let result = LlmClient::new(LlmProvider::Claude, "test-key".to_string());
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.provider(), &LlmProvider::Claude);
        assert_eq!(client.model(), LlmProvider::Claude.default_model());

        // Test empty API key
        let result = LlmClient::new(LlmProvider::Claude, "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_llm_request_creation() {
        let request = LlmRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.5),
            system_prompt: Some("You are a helpful assistant".to_string()),
        };

        assert_eq!(request.prompt, "Test prompt");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.5));
        assert!(request.system_prompt.is_some());
    }

    #[tokio::test]
    async fn test_llm_client_with_error_handler() {
        let client = LlmClient::new(LlmProvider::Claude, "test-key".to_string()).unwrap();
        
        // Test that error handler is properly initialized
        let error_handler = client.error_handler();
        assert!(error_handler.lock().is_ok());
    }

    #[test]
    fn test_llm_client_with_custom_model() {
        let client = LlmClient::new(LlmProvider::Claude, "test-key".to_string())
            .unwrap()
            .with_model("claude-3-opus-20240229".to_string());
        
        assert_eq!(client.model(), "claude-3-opus-20240229");
        assert_eq!(client.provider(), &LlmProvider::Claude);
    }

    #[test]
    fn test_llm_request_serialization() {
        let request = LlmRequest {
            prompt: "Test prompt".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            system_prompt: Some("System prompt".to_string()),
        };

        // Test that the request can be serialized and deserialized
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: LlmRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.prompt, deserialized.prompt);
        assert_eq!(request.max_tokens, deserialized.max_tokens);
        assert_eq!(request.temperature, deserialized.temperature);
        assert_eq!(request.system_prompt, deserialized.system_prompt);
    }

    #[test]
    fn test_llm_response_serialization() {
        let response = LlmResponse {
            content: "Test response".to_string(),
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
            model: "claude-3-sonnet".to_string(),
            provider: "claude".to_string(),
        };

        // Test that the response can be serialized and deserialized
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LlmResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(response.content, deserialized.content);
        assert_eq!(response.model, deserialized.model);
        assert_eq!(response.provider, deserialized.provider);
        assert!(deserialized.usage.is_some());
        
        let usage = deserialized.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_provider_case_insensitive() {
        assert_eq!(LlmProvider::from_str("CLAUDE").unwrap(), LlmProvider::Claude);
        assert_eq!(LlmProvider::from_str("ChatGPT").unwrap(), LlmProvider::ChatGpt);
        assert_eq!(LlmProvider::from_str("GEMINI").unwrap(), LlmProvider::Gemini);
        assert_eq!(LlmProvider::from_str("OLLAMA").unwrap(), LlmProvider::Ollama);
    }

    #[test]
    fn test_provider_aliases() {
        assert_eq!(LlmProvider::from_str("openai").unwrap(), LlmProvider::ChatGpt);
        assert_eq!(LlmProvider::from_str("google").unwrap(), LlmProvider::Gemini);
        assert_eq!(LlmProvider::from_str("local").unwrap(), LlmProvider::Ollama);
    }
}