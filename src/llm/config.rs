use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};

use super::client::LlmProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(default)]
    pub encrypted: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            providers: HashMap::new(),
            encryption_key: None,
        }
    }
}

impl LlmConfig {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: LlmConfig = serde_json::from_str(&content)?;
            
            // Decrypt API keys if they are encrypted
            config.decrypt_api_keys()?;
            
            Ok(config)
        } else {
            // Create default config and save it
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Clone config and encrypt API keys before saving
        let mut config_to_save = self.clone();
        config_to_save.encrypt_api_keys()?;

        let content = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&config_path, content)?;
        
        // Set restrictive permissions on the config file (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&config_path, perms)?;
        }

        Ok(())
    }

    /// Get the configuration file path
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            return Err(anyhow!("Cannot determine config directory"));
        };

        Ok(config_dir.join("docpilot").join("config.json"))
    }

    /// Set API key for a provider
    pub fn set_api_key(&mut self, provider: &str, api_key: String) -> Result<()> {
        let provider_config = self.providers.entry(provider.to_string()).or_insert_with(|| {
            ProviderConfig {
                api_key: String::new(),
                model: None,
                base_url: None,
                max_tokens: None,
                temperature: None,
                encrypted: false,
            }
        });

        provider_config.api_key = api_key;
        provider_config.encrypted = false; // Will be encrypted when saved
        
        Ok(())
    }

    /// Get API key for a provider
    pub fn get_api_key(&self, provider: &str) -> Option<&str> {
        self.providers.get(provider).map(|config| config.api_key.as_str())
    }

    /// Set default provider
    pub fn set_default_provider(&mut self, provider: String) -> Result<()> {
        // Validate that the provider is supported
        LlmProvider::from_str(&provider)?;
        self.default_provider = Some(provider);
        Ok(())
    }

    /// Get default provider
    pub fn get_default_provider(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// Set model for a provider
    pub fn set_model(&mut self, provider: &str, model: String) {
        let provider_config = self.providers.entry(provider.to_string()).or_insert_with(|| {
            ProviderConfig {
                api_key: String::new(),
                model: None,
                base_url: None,
                max_tokens: None,
                temperature: None,
                encrypted: false,
            }
        });

        provider_config.model = Some(model);
    }

    /// Get model for a provider
    pub fn get_model(&self, provider: &str) -> Option<&str> {
        self.providers.get(provider).and_then(|config| config.model.as_deref())
    }

    /// Set base URL for a provider (useful for Ollama or custom endpoints)
    pub fn set_base_url(&mut self, provider: &str, base_url: String) {
        let provider_config = self.providers.entry(provider.to_string()).or_insert_with(|| {
            ProviderConfig {
                api_key: String::new(),
                model: None,
                base_url: None,
                max_tokens: None,
                temperature: None,
                encrypted: false,
            }
        });

        provider_config.base_url = Some(base_url);
    }

    /// Get base URL for a provider
    pub fn get_base_url(&self, provider: &str) -> Option<&str> {
        self.providers.get(provider).and_then(|config| config.base_url.as_deref())
    }

    /// List configured providers
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, provider: &str) -> bool {
        self.providers.remove(provider).is_some()
    }

    /// Check if a provider is configured
    pub fn has_provider(&self, provider: &str) -> bool {
        self.providers.contains_key(provider) && 
        self.providers.get(provider).map_or(false, |config| !config.api_key.is_empty())
    }

    /// Generate or get encryption key
    fn get_encryption_key(&self) -> Result<String> {
        if let Some(key) = &self.encryption_key {
            Ok(key.clone())
        } else {
            // Generate a simple key based on machine-specific information
            let hostname = hostname::get()
                .map_err(|_| anyhow!("Cannot get hostname"))?
                .to_string_lossy()
                .to_string();
            
            let user = env::var("USER").or_else(|_| env::var("USERNAME"))
                .unwrap_or_else(|_| "unknown".to_string());
            
            let key_material = format!("docpilot-{}-{}", hostname, user);
            let encoded = general_purpose::STANDARD.encode(key_material.as_bytes());
            
            // Take first 32 characters for a consistent key
            Ok(encoded.chars().take(32).collect())
        }
    }

    /// Encrypt API keys in the configuration
    fn encrypt_api_keys(&mut self) -> Result<()> {
        let key = self.get_encryption_key()?;
        
        for (_, provider_config) in &mut self.providers {
            if !provider_config.encrypted && !provider_config.api_key.is_empty() {
                provider_config.api_key = Self::simple_encrypt(&provider_config.api_key, &key)?;
                provider_config.encrypted = true;
            }
        }
        
        Ok(())
    }

    /// Decrypt API keys in the configuration
    fn decrypt_api_keys(&mut self) -> Result<()> {
        let key = self.get_encryption_key()?;
        
        for (_, provider_config) in &mut self.providers {
            if provider_config.encrypted && !provider_config.api_key.is_empty() {
                provider_config.api_key = Self::simple_decrypt(&provider_config.api_key, &key)?;
                provider_config.encrypted = false;
            }
        }
        
        Ok(())
    }

    /// Simple XOR-based encryption (not cryptographically secure, but better than plaintext)
    fn simple_encrypt(data: &str, key: &str) -> Result<String> {
        let key_bytes = key.as_bytes();
        let encrypted: Vec<u8> = data.as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect();
        
        Ok(general_purpose::STANDARD.encode(&encrypted))
    }

    /// Simple XOR-based decryption
    fn simple_decrypt(encrypted_data: &str, key: &str) -> Result<String> {
        let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data)?;
        let key_bytes = key.as_bytes();
        
        let decrypted: Vec<u8> = encrypted_bytes
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
            .collect();
        
        String::from_utf8(decrypted).map_err(|e| anyhow!("Decryption failed: {}", e))
    }

    /// Load API key from environment variable as fallback
    pub fn get_api_key_with_fallback(&self, provider: &str) -> Option<String> {
        // First try to get from config
        if let Some(key) = self.get_api_key(provider) {
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }

        // Fallback to environment variables
        let env_var = match provider.to_lowercase().as_str() {
            "claude" => "ANTHROPIC_API_KEY",
            "chatgpt" | "openai" => "OPENAI_API_KEY",
            "gemini" | "google" => "GOOGLE_API_KEY",
            "ollama" => "OLLAMA_API_KEY", // Optional for Ollama
            _ => return None,
        };

        env::var(env_var).ok()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check if any providers are configured
        if self.providers.is_empty() {
            warnings.push("No LLM providers configured".to_string());
        }

        // Check each provider
        for (provider_name, config) in &self.providers {
            if config.api_key.is_empty() {
                warnings.push(format!("Provider '{}' has no API key", provider_name));
            }

            // Validate provider name
            if LlmProvider::from_str(provider_name).is_err() {
                warnings.push(format!("Unknown provider: '{}'", provider_name));
            }
        }

        // Check default provider
        if let Some(default) = &self.default_provider {
            if !self.providers.contains_key(default) {
                warnings.push(format!("Default provider '{}' is not configured", default));
            }
        }

        Ok(warnings)
    }

    /// Check if the configuration has at least one properly configured provider
    pub fn is_configured(&self) -> bool {
        // Check if we have a default provider that's properly configured
        if let Some(default_provider) = &self.default_provider {
            if self.has_provider(default_provider) {
                return true;
            }
        }
        
        // Check if any provider is configured with an API key
        self.providers.iter().any(|(_, config)| !config.api_key.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_creation() {
        let config = LlmConfig::default();
        assert!(config.providers.is_empty());
        assert!(config.default_provider.is_none());
    }

    #[test]
    fn test_api_key_management() {
        let mut config = LlmConfig::default();
        
        config.set_api_key("claude", "test-key".to_string()).unwrap();
        assert_eq!(config.get_api_key("claude"), Some("test-key"));
        assert_eq!(config.get_api_key("nonexistent"), None);
        
        assert!(config.has_provider("claude"));
        assert!(!config.has_provider("nonexistent"));
    }

    #[test]
    fn test_provider_management() {
        let mut config = LlmConfig::default();
        
        config.set_api_key("claude", "key1".to_string()).unwrap();
        config.set_api_key("chatgpt", "key2".to_string()).unwrap();
        
        let providers = config.list_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"claude"));
        assert!(providers.contains(&"chatgpt"));
        
        assert!(config.remove_provider("claude"));
        assert!(!config.remove_provider("nonexistent"));
        assert_eq!(config.list_providers().len(), 1);
    }

    #[test]
    fn test_default_provider() {
        let mut config = LlmConfig::default();
        
        assert!(config.set_default_provider("claude".to_string()).is_ok());
        assert_eq!(config.get_default_provider(), Some("claude"));
        
        assert!(config.set_default_provider("invalid".to_string()).is_err());
    }

    #[test]
    fn test_encryption_decryption() {
        let key = "test-key-12345678901234567890";
        let data = "secret-api-key";
        
        let encrypted = LlmConfig::simple_encrypt(data, key).unwrap();
        assert_ne!(encrypted, data);
        
        let decrypted = LlmConfig::simple_decrypt(&encrypted, key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_model_management() {
        let mut config = LlmConfig::default();
        
        config.set_model("claude", "claude-3-opus".to_string());
        assert_eq!(config.get_model("claude"), Some("claude-3-opus"));
        assert_eq!(config.get_model("nonexistent"), None);
    }

    #[test]
    fn test_base_url_management() {
        let mut config = LlmConfig::default();
        
        config.set_base_url("ollama", "http://localhost:11434".to_string());
        assert_eq!(config.get_base_url("ollama"), Some("http://localhost:11434"));
        assert_eq!(config.get_base_url("nonexistent"), None);
    }

    #[test]
    fn test_validation() {
        let mut config = LlmConfig::default();
        
        // Empty config should have warnings
        let warnings = config.validate().unwrap();
        assert!(!warnings.is_empty());
        
        // Add valid provider
        config.set_api_key("claude", "test-key".to_string()).unwrap();
        let warnings = config.validate().unwrap();
        assert!(warnings.len() < 2); // Should have fewer warnings
        
        // Add invalid provider
        config.providers.insert("invalid".to_string(), ProviderConfig {
            api_key: "key".to_string(),
            model: None,
            base_url: None,
            max_tokens: None,
            temperature: None,
            encrypted: false,
        });
        
        let warnings = config.validate().unwrap();
        assert!(warnings.iter().any(|w| w.contains("Unknown provider")));
    }
}