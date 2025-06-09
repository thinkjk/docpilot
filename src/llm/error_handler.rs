use std::time::Duration;
use std::fmt;
use tokio::time::sleep;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub retry_after: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LlmError {
    RateLimited {
        provider: String,
        message: String,
        retry_after: Option<Duration>,
    },
    NetworkError {
        provider: String,
        error: String,
        retryable: bool,
    },
    ApiError {
        provider: String,
        message: String,
    },
    ParseError {
        provider: String,
        message: String,
    },
    AuthenticationError {
        provider: String,
        message: String,
    },
    QuotaExceeded {
        provider: String,
        reset_time: Option<Duration>,
    },
    InvalidRequest {
        provider: String,
        message: String,
    },
    ServiceUnavailable {
        provider: String,
        retry_after: Option<Duration>,
    },
    CircuitBreakerOpen {
        provider: String,
        reset_time: Duration,
    },
    MaxRetriesExceeded {
        provider: String,
        attempts: u32,
        last_error: String,
    },
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RateLimited { provider, message, retry_after } => {
                write!(f, "Rate limited by {}: {}. Retry after: {:?}", provider, message, retry_after)
            }
            LlmError::NetworkError { provider, error, retryable } => {
                write!(f, "Network error with {} (retryable: {}): {}", provider, retryable, error)
            }
            LlmError::ApiError { provider, message } => {
                write!(f, "API error from {}: {}", provider, message)
            }
            LlmError::ParseError { provider, message } => {
                write!(f, "Parse error from {}: {}", provider, message)
            }
            LlmError::AuthenticationError { provider, message } => {
                write!(f, "Authentication error with {}: {}", provider, message)
            }
            LlmError::QuotaExceeded { provider, reset_time } => {
                write!(f, "Quota exceeded for {}. Reset time: {:?}", provider, reset_time)
            }
            LlmError::InvalidRequest { provider, message } => {
                write!(f, "Invalid request to {}: {}", provider, message)
            }
            LlmError::ServiceUnavailable { provider, retry_after } => {
                write!(f, "Service unavailable for {}. Retry after: {:?}", provider, retry_after)
            }
            LlmError::CircuitBreakerOpen { provider, reset_time } => {
                write!(f, "Circuit breaker open for {}. Reset in: {:?}", provider, reset_time)
            }
            LlmError::MaxRetriesExceeded { provider, attempts, last_error } => {
                write!(f, "Max retries ({}) exceeded for {}: {}", attempts, provider, last_error)
            }
        }
    }
}

impl std::error::Error for LlmError {}

#[derive(Debug)]
struct CircuitBreakerState {
    failure_count: u32,
    reset_time: std::time::Instant,
}

impl CircuitBreakerState {
    fn is_open(&self) -> bool {
        self.failure_count >= 5 && std::time::Instant::now() < self.reset_time
    }
}

pub struct ErrorHandler {
    config: RetryConfig,
    circuit_breaker: Option<CircuitBreakerState>,
}

impl ErrorHandler {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            circuit_breaker: None,
        }
    }

    /// Execute a function with retry logic and error handling
    pub async fn execute_with_retry<F, Fut, T>(&mut self, mut operation: F) -> Result<T, LlmError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, LlmError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.config.max_retries {
            // Check circuit breaker
            if let Some(state) = &self.circuit_breaker {
                if state.is_open() {
                    return Err(LlmError::CircuitBreakerOpen {
                        provider: "unknown".to_string(),
                        reset_time: state.reset_time.saturating_duration_since(std::time::Instant::now()),
                    });
                }
            }

            match operation().await {
                Ok(result) => {
                    // Reset circuit breaker on success
                    self.circuit_breaker = None;
                    return Ok(result);
                }
                Err(error) => {
                    attempts += 1;
                    last_error = Some(error.clone());

                    // Handle rate limiting
                    if let LlmError::RateLimited { retry_after, .. } = &error {
                        if let Some(delay) = retry_after {
                            sleep(*delay).await;
                            continue;
                        }
                    }

                    // Check if error is retryable
                    if !self.is_retryable(&error) || attempts > self.config.max_retries {
                        // Update circuit breaker on non-retryable errors
                        if matches!(error, LlmError::ServiceUnavailable { .. }) {
                            self.circuit_breaker = Some(CircuitBreakerState {
                                failure_count: 1,
                                reset_time: std::time::Instant::now() + Duration::from_secs(60),
                            });
                        }
                        break;
                    }

                    // Calculate delay with exponential backoff
                    let delay = self.calculate_delay(attempts);
                    sleep(delay).await;
                }
            }
        }

        Err(LlmError::MaxRetriesExceeded {
            provider: "unknown".to_string(),
            attempts,
            last_error: last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Check if an error is retryable
    fn is_retryable(&self, error: &LlmError) -> bool {
        match error {
            LlmError::RateLimited { .. } => true,
            LlmError::NetworkError { retryable, .. } => *retryable,
            LlmError::ServiceUnavailable { .. } => true,
            LlmError::ApiError { .. } => false,
            LlmError::ParseError { .. } => false,
            LlmError::AuthenticationError { .. } => false,
            LlmError::QuotaExceeded { .. } => false,
            LlmError::InvalidRequest { .. } => false,
            LlmError::CircuitBreakerOpen { .. } => false,
            LlmError::MaxRetriesExceeded { .. } => false,
        }
    }

    /// Calculate delay for retry with exponential backoff
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay_ms = self.config.base_delay.as_millis() as f64;
        let delay_ms = base_delay_ms * self.config.backoff_multiplier.powi(attempt as i32 - 1);
        
        let final_delay_ms = if self.config.jitter {
            // Add jitter (±25% randomization)
            let jitter = rand::thread_rng().gen_range(-0.25..=0.25);
            delay_ms * (1.0 + jitter)
        } else {
            delay_ms
        };
        
        let delay = Duration::from_millis(final_delay_ms as u64);
        std::cmp::min(delay, self.config.max_delay)
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let handler = ErrorHandler::default();
        
        let rate_limit = LlmError::RateLimited {
            provider: "test".to_string(),
            message: "Rate limited".to_string(),
            retry_after: Some(Duration::from_secs(60)),
        };
        assert!(handler.is_retryable(&rate_limit));

        let auth_error = LlmError::AuthenticationError {
            provider: "test".to_string(),
            message: "Invalid key".to_string(),
        };
        assert!(!handler.is_retryable(&auth_error));
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(30),
            jitter: false,
            ..Default::default()
        };
        let handler = ErrorHandler::new(config);

        let delay1 = handler.calculate_delay(1);
        let delay2 = handler.calculate_delay(2);
        let delay3 = handler.calculate_delay(3);

        assert_eq!(delay1, Duration::from_millis(1000));
        assert_eq!(delay2, Duration::from_millis(2000));
        assert_eq!(delay3, Duration::from_millis(4000));
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let mut handler = ErrorHandler::new(RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        });

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
}