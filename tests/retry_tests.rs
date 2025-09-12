//! Comprehensive tests for the retry and circuit breaker system

use dbfast::errors::{DatabaseError, DbFastError, ErrorContext, ErrorSeverity};
use dbfast::retry::{
    BackoffStrategy, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, RetryExecutor,
    RetryPolicy, RetryResult,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_retry_policy_creation() {
    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_initial_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(10))
        .with_backoff_strategy(BackoffStrategy::Exponential)
        .with_jitter(true);

    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.initial_delay, Duration::from_millis(100));
    assert_eq!(policy.max_delay, Duration::from_secs(10));
    assert!(matches!(
        policy.backoff_strategy,
        BackoffStrategy::Exponential
    ));
    assert!(policy.jitter);
}

#[test]
fn test_backoff_strategies() {
    let policy = RetryPolicy::new().with_initial_delay(Duration::from_millis(100));

    // Test different backoff strategies
    let fixed_policy = policy.clone().with_backoff_strategy(BackoffStrategy::Fixed);
    let linear_policy = policy
        .clone()
        .with_backoff_strategy(BackoffStrategy::Linear);
    let exponential_policy = policy
        .clone()
        .with_backoff_strategy(BackoffStrategy::Exponential);
    let fibonacci_policy = policy
        .clone()
        .with_backoff_strategy(BackoffStrategy::Fibonacci);

    // Fixed should always return the same delay
    assert_eq!(fixed_policy.calculate_delay(1), Duration::from_millis(100));
    assert_eq!(fixed_policy.calculate_delay(3), Duration::from_millis(100));

    // Linear should increase linearly
    assert_eq!(linear_policy.calculate_delay(1), Duration::from_millis(100));
    assert_eq!(linear_policy.calculate_delay(2), Duration::from_millis(200));
    assert_eq!(linear_policy.calculate_delay(3), Duration::from_millis(300));

    // Exponential should double each time
    assert_eq!(
        exponential_policy.calculate_delay(1),
        Duration::from_millis(100)
    );
    assert_eq!(
        exponential_policy.calculate_delay(2),
        Duration::from_millis(200)
    );
    assert_eq!(
        exponential_policy.calculate_delay(3),
        Duration::from_millis(400)
    );

    // Fibonacci should follow fibonacci sequence
    assert_eq!(
        fibonacci_policy.calculate_delay(1),
        Duration::from_millis(100)
    );
    assert_eq!(
        fibonacci_policy.calculate_delay(2),
        Duration::from_millis(100)
    );
    assert_eq!(
        fibonacci_policy.calculate_delay(3),
        Duration::from_millis(200)
    );
    assert_eq!(
        fibonacci_policy.calculate_delay(4),
        Duration::from_millis(300)
    );
}

#[tokio::test]
async fn test_retry_executor_success() {
    let policy = RetryPolicy::new()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(10));

    let executor = RetryExecutor::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = executor
        .execute(|| async move {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                // Fail first two attempts
                Err(create_retryable_error("Temporary failure"))
            } else {
                // Succeed on third attempt
                Ok("Success".to_string())
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(counter.load(Ordering::SeqCst), 3); // Should have tried 3 times
}

#[tokio::test]
async fn test_retry_executor_max_attempts_exceeded() {
    let policy = RetryPolicy::new()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(10));

    let executor = RetryExecutor::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = executor
        .execute(|| async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(create_retryable_error("Always fails"))
        })
        .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::SeqCst), 3); // Should have tried max attempts
}

#[tokio::test]
async fn test_retry_executor_non_retryable_error() {
    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_initial_delay(Duration::from_millis(10));

    let executor = RetryExecutor::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = executor
        .execute(|| async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(create_non_retryable_error("Permanent failure"))
        })
        .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::SeqCst), 1); // Should have tried only once
}

#[tokio::test]
async fn test_circuit_breaker_closed_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        half_open_max_calls: 1,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Initially should be closed
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::Closed);

    // Should allow calls
    assert!(circuit_breaker.call_allowed().await);
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        half_open_max_calls: 1,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Record failures to trigger opening
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;

    // Circuit should now be open
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::Open);

    // Should not allow calls
    assert!(!circuit_breaker.call_allowed().await);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_transition() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(50),
        half_open_max_calls: 1,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Open the circuit
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::Open);

    // Wait for timeout
    sleep(Duration::from_millis(60)).await;

    // Should transition to half-open and allow one call
    assert!(circuit_breaker.call_allowed().await);
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::HalfOpen);
}

#[tokio::test]
async fn test_circuit_breaker_recovery() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(50),
        half_open_max_calls: 3,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Open the circuit
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::Open);

    // Wait for timeout to transition to half-open
    sleep(Duration::from_millis(60)).await;
    circuit_breaker.call_allowed().await; // Force transition to half-open

    // Record successes to close the circuit
    circuit_breaker.record_success().await;
    circuit_breaker.record_success().await;

    // Circuit should be closed again
    assert_eq!(circuit_breaker.state().await, CircuitBreakerState::Closed);
    assert!(circuit_breaker.call_allowed().await);
}

#[tokio::test]
async fn test_retry_with_circuit_breaker() {
    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_initial_delay(Duration::from_millis(10));

    let circuit_config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(100),
        half_open_max_calls: 1,
    };

    let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_config));
    let executor = RetryExecutor::new(policy).with_circuit_breaker(circuit_breaker.clone());

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // First call should fail and open circuit after threshold
    let result1 = executor
        .execute(|| async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(create_retryable_error("Service unavailable"))
        })
        .await;

    assert!(result1.is_err());

    // Circuit should be open, subsequent calls should fail fast
    let start_time = std::time::Instant::now();
    let result2 = executor
        .execute(|| async move {
            panic!("Should not be called due to open circuit");
        })
        .await;

    let elapsed = start_time.elapsed();
    assert!(result2.is_err());
    assert!(elapsed < Duration::from_millis(50)); // Should fail fast
}

#[test]
fn test_fibonacci_calculation() {
    use dbfast::retry::fibonacci;

    assert_eq!(fibonacci(0), 0);
    assert_eq!(fibonacci(1), 1);
    assert_eq!(fibonacci(2), 1);
    assert_eq!(fibonacci(3), 2);
    assert_eq!(fibonacci(4), 3);
    assert_eq!(fibonacci(5), 5);
    assert_eq!(fibonacci(8), 21);
}

#[tokio::test]
async fn test_jitter_application() {
    let policy = RetryPolicy::new()
        .with_initial_delay(Duration::from_millis(100))
        .with_jitter(true);

    let delays: Vec<Duration> = (1..=10)
        .map(|attempt| policy.calculate_delay(attempt))
        .collect();

    // With jitter, delays should vary
    let unique_delays: std::collections::HashSet<_> = delays.into_iter().collect();
    assert!(
        unique_delays.len() > 1,
        "Jitter should create different delays"
    );
}

#[tokio::test]
async fn test_max_delay_enforcement() {
    let policy = RetryPolicy::new()
        .with_initial_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_millis(500))
        .with_backoff_strategy(BackoffStrategy::Exponential);

    // Even with exponential backoff, delay should not exceed max_delay
    let delay = policy.calculate_delay(10); // This would normally be very large
    assert!(delay <= Duration::from_millis(500));
}

#[tokio::test]
async fn test_retry_result_tracking() {
    let policy = RetryPolicy::new()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(10));

    let executor = RetryExecutor::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = executor
        .execute_with_result(|| async move {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(create_retryable_error("Temporary failure"))
            } else {
                Ok("Success".to_string())
            }
        })
        .await;

    match result {
        RetryResult::Success {
            value,
            attempts,
            total_duration,
        } => {
            assert_eq!(value, "Success");
            assert_eq!(attempts, 3);
            assert!(total_duration > Duration::from_millis(20)); // At least 2 retry delays
        }
        _ => panic!("Expected success result"),
    }
}

#[tokio::test]
async fn test_concurrent_circuit_breaker_operations() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_millis(100),
        half_open_max_calls: 2,
    };

    let circuit_breaker = Arc::new(CircuitBreaker::new(config));

    // Spawn multiple concurrent tasks
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let cb = circuit_breaker.clone();
            tokio::spawn(async move {
                if i % 2 == 0 {
                    cb.record_success().await;
                } else {
                    cb.record_failure().await;
                }
                cb.call_allowed().await
            })
        })
        .collect();

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All tasks should complete without panicking
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }
}

// Helper functions
fn create_retryable_error(message: &str) -> DbFastError {
    DbFastError::Database {
        source: DatabaseError::Connection {
            message: message.to_string(),
            host: Some("localhost".to_string()),
            port: Some(5432),
        },
        context: ErrorContext {
            operation: "test_operation".to_string(),
            component: "test_component".to_string(),
            user_message: None,
            details: HashMap::new(),
            severity: ErrorSeverity::Medium,
            recoverability: dbfast::errors::ErrorRecoverability::Recoverable,
            correlation_id: None,
        },
    }
}

fn create_non_retryable_error(message: &str) -> DbFastError {
    DbFastError::Database {
        source: DatabaseError::Query {
            query: "SELECT * FROM invalid".to_string(),
            message: message.to_string(),
            hint: None,
        },
        context: ErrorContext {
            operation: "test_operation".to_string(),
            component: "test_component".to_string(),
            user_message: None,
            details: HashMap::new(),
            severity: ErrorSeverity::High,
            recoverability: dbfast::errors::ErrorRecoverability::PermanentFailure,
            correlation_id: None,
        },
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_high_frequency_circuit_breaker_operations() {
        let config = CircuitBreakerConfig {
            failure_threshold: 100,
            success_threshold: 50,
            timeout: Duration::from_millis(1000),
            half_open_max_calls: 10,
        };

        let circuit_breaker = CircuitBreaker::new(config);
        let start = std::time::Instant::now();

        // Perform many operations
        for i in 0..1000 {
            circuit_breaker.call_allowed().await;
            if i % 2 == 0 {
                circuit_breaker.record_success().await;
            } else {
                circuit_breaker.record_failure().await;
            }
        }

        let duration = start.elapsed();
        println!(
            "1000 circuit breaker operations completed in {:?}",
            duration
        );

        // Should complete reasonably quickly
        assert!(duration < Duration::from_secs(1));
    }
}
