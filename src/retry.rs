//! Graceful error recovery and retry mechanisms
//!
//! This module provides intelligent retry strategies for:
//! - Database connection failures
//! - Network timeouts
//! - Temporary resource exhaustion
//! - Transient errors

use crate::errors::{DbFastError, ErrorSeverity};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Backoff strategy to use
    pub backoff_strategy: BackoffStrategy,

    /// Whether to add jitter to delays
    pub jitter: bool,

    /// Predicate to determine if error is retryable
    pub retry_condition: fn(&DbFastError) -> bool,
}

/// Backoff strategies for retry delays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,

    /// Linear increase in delay
    Linear,

    /// Exponential backoff (delay doubles each time)
    Exponential,

    /// Fibonacci sequence backoff
    Fibonacci,
}

/// Result of a retry operation
#[derive(Debug)]
pub enum RetryResult<T> {
    /// Operation succeeded
    Success(T),

    /// Operation failed after all retries
    Failed {
        last_error: DbFastError,
        attempts: u32,
        total_duration: Duration,
    },
}

/// Retry context information
#[derive(Debug)]
pub struct RetryContext {
    /// Current attempt number (1-based)
    pub attempt: u32,

    /// Total time spent retrying so far
    pub elapsed: Duration,

    /// Last error encountered
    pub last_error: Option<DbFastError>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_strategy: BackoffStrategy::Exponential,
            jitter: true,
            retry_condition: default_retry_condition,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of attempts
    #[must_use]
    pub const fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set initial delay
    #[must_use]
    pub const fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    #[must_use]
    pub const fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff strategy
    #[must_use]
    pub const fn with_backoff_strategy(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff_strategy = strategy;
        self
    }

    /// Enable or disable jitter
    #[must_use]
    pub const fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Set custom retry condition
    #[must_use]
    pub fn with_retry_condition(mut self, condition: fn(&DbFastError) -> bool) -> Self {
        self.retry_condition = condition;
        self
    }

    /// Execute an operation with retry logic
    #[allow(clippy::future_not_send)]
    pub async fn execute<T, F, Fut>(&self, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, DbFastError>>,
    {
        let start_time = std::time::Instant::now();
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            let _attempt_start = std::time::Instant::now();

            debug!(
                "Attempting operation (attempt {}/{})",
                attempt, self.max_attempts
            );

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            "Operation succeeded on attempt {} after {:?}",
                            attempt,
                            start_time.elapsed()
                        );
                    }
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    let should_retry = (self.retry_condition)(&error);
                    last_error = Some(error.clone());

                    if attempt < self.max_attempts && should_retry {
                        let delay = self.calculate_delay(attempt);

                        warn!(
                            "Operation failed on attempt {} ({}), retrying in {:?}: {}",
                            attempt,
                            error.context().operation,
                            delay,
                            error
                        );

                        sleep(delay).await;
                    } else if !should_retry {
                        warn!("Operation failed with non-retryable error: {}", error);
                        break;
                    } else {
                        warn!("Operation failed on final attempt {}: {}", attempt, error);
                    }
                }
            }
        }

        RetryResult::Failed {
            last_error: last_error.unwrap(),
            attempts: self.max_attempts,
            total_duration: start_time.elapsed(),
        }
    }

    /// Execute with callback for each retry attempt
    #[allow(clippy::future_not_send)]
    pub async fn execute_with_callback<T, F, Fut, C>(
        &self,
        mut operation: F,
        mut callback: C,
    ) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, DbFastError>>,
        C: FnMut(RetryContext),
    {
        let start_time = std::time::Instant::now();
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            let context = RetryContext {
                attempt,
                elapsed: start_time.elapsed(),
                last_error: last_error.clone(),
            };

            callback(context);

            match operation().await {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    let should_retry = (self.retry_condition)(&error);
                    last_error = Some(error.clone());

                    if attempt < self.max_attempts && should_retry {
                        let delay = self.calculate_delay(attempt);
                        sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        RetryResult::Failed {
            last_error: last_error.unwrap(),
            attempts: self.max_attempts,
            total_duration: start_time.elapsed(),
        }
    }

    /// Calculate delay for a given attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = match self.backoff_strategy {
            BackoffStrategy::Fixed => self.initial_delay,
            BackoffStrategy::Linear => self.initial_delay * attempt,
            BackoffStrategy::Exponential => {
                let multiplier = 2_u32.pow(attempt.saturating_sub(1));
                self.initial_delay * multiplier
            }
            BackoffStrategy::Fibonacci => {
                let fib = fibonacci(attempt as usize);
                #[allow(clippy::cast_possible_truncation)]
                {
                    self.initial_delay * fib as u32
                }
            }
        };

        let mut delay = base_delay.min(self.max_delay);

        // Add jitter if enabled
        if self.jitter {
            #[allow(clippy::cast_precision_loss)]
            let jitter_amount = delay.as_millis() as f64 * 0.1; // 10% jitter
            let jitter = (fastrand::f64() * jitter_amount).mul_add(2.0, -jitter_amount);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let jitter_duration = Duration::from_millis(jitter as u64);

            if jitter >= 0.0 {
                delay += jitter_duration;
            } else {
                delay = delay.saturating_sub(jitter_duration);
            }
        }

        delay
    }
}

/// Predefined retry policies for common scenarios
impl RetryPolicy {
    /// Policy for database operations
    #[must_use]
    pub fn database_operations() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_strategy: BackoffStrategy::Exponential,
            jitter: true,
            retry_condition: |error| {
                matches!(
                    error,
                    DbFastError::Database { .. }
                        | DbFastError::Network { .. }
                        | DbFastError::Remote { .. }
                )
            },
        }
    }

    /// Policy for network operations
    #[must_use]
    pub fn network_operations() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_strategy: BackoffStrategy::Exponential,
            jitter: true,
            retry_condition: |error| {
                matches!(
                    error,
                    DbFastError::Network { .. } | DbFastError::Remote { .. }
                )
            },
        }
    }

    /// Policy for file operations
    #[must_use]
    pub fn file_operations() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
            backoff_strategy: BackoffStrategy::Linear,
            jitter: false,
            retry_condition: |error| {
                matches!(
                    error,
                    DbFastError::FileSystem { .. } | DbFastError::Resource { .. }
                )
            },
        }
    }

    /// Aggressive retry policy for critical operations
    #[must_use]
    pub fn critical_operations() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_strategy: BackoffStrategy::Fibonacci,
            jitter: true,
            retry_condition: |error| error.is_recoverable(),
        }
    }

    /// Fast retry policy for quick operations
    #[must_use]
    pub fn fast_operations() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Fixed,
            jitter: false,
            retry_condition: |error| {
                matches!(
                    error.context().severity,
                    ErrorSeverity::Low | ErrorSeverity::Medium
                )
            },
        }
    }
}

/// Default retry condition - retry if error is recoverable
const fn default_retry_condition(error: &DbFastError) -> bool {
    error.is_recoverable()
}

/// Calculate fibonacci number (used for fibonacci backoff)
fn fibonacci(n: usize) -> usize {
    if n <= 1 {
        n
    } else {
        let mut a = 0;
        let mut b = 1;
        for _ in 2..=n {
            let temp = a + b;
            a = b;
            b = temp;
        }
        b
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: CircuitState,

    /// Configuration
    config: CircuitBreakerConfig,

    /// Statistics
    stats: CircuitStats,

    /// When circuit was last opened
    last_failure_time: Option<std::time::Instant>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, operations proceed normally
    Closed,

    /// Circuit is open, operations fail fast
    Open,

    /// Circuit is half-open, testing if system recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,

    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,

    /// Time to wait before trying half-open
    pub timeout: Duration,

    /// Window size for failure rate calculation
    pub window_size: u32,
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
struct CircuitStats {
    /// Recent operation results (true = success, false = failure)
    recent_results: std::collections::VecDeque<bool>,

    /// Consecutive successes in half-open state
    consecutive_successes: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            window_size: 10,
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    #[must_use]
    pub fn new(config: Option<CircuitBreakerConfig>) -> Self {
        let config = config.unwrap_or_default();

        Self {
            state: CircuitState::Closed,
            config,
            stats: CircuitStats {
                recent_results: std::collections::VecDeque::new(),
                consecutive_successes: 0,
            },
            last_failure_time: None,
        }
    }

    /// Execute an operation with circuit breaker protection
    #[allow(clippy::future_not_send)]
    pub async fn execute<T, F, Fut>(&mut self, operation: F) -> Result<T, DbFastError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, DbFastError>>,
    {
        // Check if circuit should transition from open to half-open
        if self.state == CircuitState::Open {
            if let Some(last_failure) = self.last_failure_time {
                if last_failure.elapsed() > self.config.timeout {
                    self.transition_to_half_open();
                } else {
                    return Err(DbFastError::Resource {
                        source: crate::errors::ResourceError::ConnectionLimit,
                        context: Box::new(
                            crate::errors::ErrorContext::new("circuit_breaker", "retry")
                                .with_severity(ErrorSeverity::High),
                        ),
                    });
                }
            }
        }

        // Reject immediately if circuit is open
        if self.state == CircuitState::Open {
            return Err(DbFastError::Resource {
                source: crate::errors::ResourceError::ConnectionLimit,
                context: Box::new(
                    crate::errors::ErrorContext::new("circuit_breaker", "retry")
                        .with_severity(ErrorSeverity::High),
                ),
            });
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(error)
            }
        }
    }

    /// Record a successful operation
    fn record_success(&mut self) {
        self.stats.recent_results.push_back(true);

        if self.stats.recent_results.len() > self.config.window_size as usize {
            self.stats.recent_results.pop_front();
        }

        if self.state == CircuitState::HalfOpen {
            self.stats.consecutive_successes += 1;
            if self.stats.consecutive_successes >= self.config.success_threshold {
                self.transition_to_closed();
            }
        }
    }

    /// Record a failed operation
    fn record_failure(&mut self) {
        self.stats.recent_results.push_back(false);

        if self.stats.recent_results.len() > self.config.window_size as usize {
            self.stats.recent_results.pop_front();
        }

        self.stats.consecutive_successes = 0;
        self.last_failure_time = Some(std::time::Instant::now());

        // Check if we should open the circuit
        #[allow(clippy::cast_possible_truncation)]
        let failure_count = self.stats.recent_results.iter().filter(|&&r| !r).count() as u32;
        if failure_count >= self.config.failure_threshold {
            self.transition_to_open();
        }
    }

    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        info!("Circuit breaker transitioning to CLOSED state");
        self.state = CircuitState::Closed;
        self.stats.consecutive_successes = 0;
    }

    /// Transition to open state
    fn transition_to_open(&mut self) {
        warn!("Circuit breaker transitioning to OPEN state");
        self.state = CircuitState::Open;
        self.last_failure_time = Some(std::time::Instant::now());
    }

    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        info!("Circuit breaker transitioning to HALF-OPEN state");
        self.state = CircuitState::HalfOpen;
        self.stats.consecutive_successes = 0;
    }

    /// Get current circuit state
    #[must_use]
    pub const fn state(&self) -> CircuitState {
        self.state
    }

    /// Get failure rate
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        if self.stats.recent_results.is_empty() {
            return 0.0;
        }

        let failure_count = self.stats.recent_results.iter().filter(|&&r| !r).count();
        #[allow(clippy::cast_precision_loss)]
        {
            failure_count as f64 / self.stats.recent_results.len() as f64
        }
    }
}

/// Convenience macro for retry operations
#[macro_export]
macro_rules! retry_operation {
    ($policy:expr, $operation:expr) => {{
        $policy.execute(|| async { $operation }).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_policy_success_on_first_attempt() {
        let policy = RetryPolicy::new().with_max_attempts(3);

        let result = policy
            .execute(|| async { Ok::<i32, DbFastError>(42) })
            .await;

        match result {
            RetryResult::Success(value) => assert_eq!(value, 42),
            RetryResult::Failed { .. } => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_retry_policy_success_after_failures() {
        let policy = RetryPolicy::new().with_max_attempts(3);
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = policy
            .execute(move || {
                let attempts = attempts_clone.clone();
                async move {
                    let current_attempts =
                        attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if current_attempts < 3 {
                        Err(DbFastError::Network {
                            message: "Connection timeout".to_string(),
                            context: Box::new(crate::errors::ErrorContext::default()),
                        })
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        match result {
            RetryResult::Success(value) => assert_eq!(value, 42),
            RetryResult::Failed { .. } => panic!("Expected success after retries"),
        }
    }

    #[tokio::test]
    async fn test_fibonacci_calculation() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(8), 21);
    }

    #[test]
    fn test_circuit_breaker_closed_initially() {
        let breaker = CircuitBreaker::new(None);
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_failure_rate() {
        let mut breaker = CircuitBreaker::new(None);

        // Record some failures
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_success();

        let rate = breaker.failure_rate();
        assert!((rate - 0.666).abs() < 0.01); // Approximately 2/3
    }

    #[test]
    fn test_backoff_strategies() {
        let policy_fixed = RetryPolicy::new().with_backoff_strategy(BackoffStrategy::Fixed);
        let policy_linear = RetryPolicy::new().with_backoff_strategy(BackoffStrategy::Linear);
        let policy_exp = RetryPolicy::new().with_backoff_strategy(BackoffStrategy::Exponential);

        // Test that different strategies produce different delays
        let delay1 = policy_fixed.calculate_delay(2);
        let delay2 = policy_linear.calculate_delay(2);
        let delay3 = policy_exp.calculate_delay(2);

        // All should be different (with high probability due to jitter)
        assert!(delay1 != delay2 || delay2 != delay3);
    }
}
