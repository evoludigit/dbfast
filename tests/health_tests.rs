//! Comprehensive tests for the database health monitoring system

use dbfast::health::{
    ConnectivityMetrics, HealthCheckConfig, HealthIssue, HealthIssueType, HealthSeverity,
    HealthStatus, HealthThresholds, PerformanceMetrics, PoolStatistics,
};
use std::time::Duration;

#[test]
fn test_health_status_ordering() {
    assert!(HealthStatus::Critical > HealthStatus::Warning);
    assert!(HealthStatus::Warning > HealthStatus::Degraded);
    assert!(HealthStatus::Degraded > HealthStatus::Healthy);
}

#[test]
fn test_pool_statistics_creation() {
    let pool_stats = PoolStatistics {
        total_connections: 20,
        active_connections: 15,
        idle_connections: 5,
        max_connections: 20,
        utilization_percent: 75.0,
        exhaustion_count: 0,
        avg_acquire_time_ms: 50.0,
    };

    assert_eq!(pool_stats.total_connections, 20);
    assert_eq!(pool_stats.active_connections, 15);
    assert_eq!(pool_stats.idle_connections, 5);
    assert_eq!(pool_stats.utilization_percent, 75.0);
}

#[test]
fn test_performance_metrics_creation() {
    let perf_metrics = PerformanceMetrics {
        avg_query_time_ms: 150.0,
        p95_query_time_ms: 500.0,
        queries_per_second: 25.0,
        slow_query_count: 3,
        cpu_usage_percent: Some(45.0),
        memory_usage_percent: Some(60.0),
    };

    assert_eq!(perf_metrics.avg_query_time_ms, 150.0);
    assert_eq!(perf_metrics.p95_query_time_ms, 500.0);
    assert_eq!(perf_metrics.queries_per_second, 25.0);
    assert_eq!(perf_metrics.slow_query_count, 3);
}

#[test]
fn test_connectivity_metrics_creation() {
    let connectivity = ConnectivityMetrics {
        can_connect: true,
        latency_ms: Some(25.0),
        server_version: Some("PostgreSQL 15.0".to_string()),
        last_success: Some(chrono::Utc::now()),
        last_failure: None,
        recent_failures: 0,
    };

    assert!(connectivity.can_connect);
    assert_eq!(connectivity.latency_ms, Some(25.0));
    assert_eq!(connectivity.recent_failures, 0);
}

#[test]
fn test_health_thresholds() {
    let thresholds = HealthThresholds {
        pool_utilization_warning: 75.0,
        pool_utilization_critical: 90.0,
        latency_warning_ms: 100.0,
        latency_critical_ms: 500.0,
        slow_query_threshold_ms: 1000.0,
        failure_count_warning: 3,
    };

    assert!(90.0 > thresholds.pool_utilization_warning);
    assert!(80.0 > thresholds.pool_utilization_warning);
    assert!(200.0 > thresholds.latency_warning_ms);
}

#[test]
fn test_health_issue_creation() {
    let issue = HealthIssue {
        issue_type: HealthIssueType::SlowQueries,
        severity: HealthSeverity::Medium,
        description: "Average query time exceeded threshold".to_string(),
        recommendation: "Consider optimizing queries or adding indexes".to_string(),
        detected_at: chrono::Utc::now(),
    };

    assert_eq!(issue.issue_type, HealthIssueType::SlowQueries);
    assert_eq!(issue.severity, HealthSeverity::Medium);
    assert!(!issue.description.is_empty());
    assert!(!issue.recommendation.is_empty());
}

#[test]
fn test_health_issue_types() {
    let pool_issue = HealthIssueType::PoolExhaustion;
    let perf_issue = HealthIssueType::SlowQueries;
    let conn_issue = HealthIssueType::ConnectionTimeout;

    assert_eq!(pool_issue, HealthIssueType::PoolExhaustion);
    assert_eq!(perf_issue, HealthIssueType::SlowQueries);
    assert_eq!(conn_issue, HealthIssueType::ConnectionTimeout);
}

#[test]
fn test_health_severity_ordering() {
    assert!(HealthSeverity::Critical < HealthSeverity::High);
    assert!(HealthSeverity::High < HealthSeverity::Medium);
    assert!(HealthSeverity::Medium < HealthSeverity::Low);
}

#[test]
fn test_health_check_config_defaults() {
    let config = HealthCheckConfig::default();

    assert_eq!(config.check_interval, Duration::from_secs(30));
    assert_eq!(config.check_timeout, Duration::from_secs(5));
    assert_eq!(config.performance_history_size, 100);
}

#[test]
fn test_health_thresholds_defaults() {
    let thresholds = HealthThresholds::default();

    assert_eq!(thresholds.pool_utilization_warning, 70.0);
    assert_eq!(thresholds.pool_utilization_critical, 90.0);
    assert_eq!(thresholds.latency_warning_ms, 100.0);
    assert_eq!(thresholds.latency_critical_ms, 500.0);
    assert_eq!(thresholds.slow_query_threshold_ms, 1000.0);
    assert_eq!(thresholds.failure_count_warning, 3);
}

#[test]
fn test_pool_statistics_default() {
    let pool_stats = PoolStatistics::default();

    assert_eq!(pool_stats.total_connections, 0);
    assert_eq!(pool_stats.active_connections, 0);
    assert_eq!(pool_stats.idle_connections, 0);
    assert_eq!(pool_stats.max_connections, 10);
    assert_eq!(pool_stats.utilization_percent, 0.0);
    assert_eq!(pool_stats.exhaustion_count, 0);
    assert_eq!(pool_stats.avg_acquire_time_ms, 0.0);
}

#[test]
fn test_performance_metrics_default() {
    let perf_metrics = PerformanceMetrics::default();

    assert_eq!(perf_metrics.avg_query_time_ms, 0.0);
    assert_eq!(perf_metrics.p95_query_time_ms, 0.0);
    assert_eq!(perf_metrics.queries_per_second, 0.0);
    assert_eq!(perf_metrics.slow_query_count, 0);
    assert_eq!(perf_metrics.cpu_usage_percent, None);
    assert_eq!(perf_metrics.memory_usage_percent, None);
}

#[test]
fn test_connectivity_metrics_default() {
    let connectivity = ConnectivityMetrics::default();

    assert!(!connectivity.can_connect);
    assert_eq!(connectivity.latency_ms, None);
    assert_eq!(connectivity.server_version, None);
    assert_eq!(connectivity.last_success, None);
    assert_eq!(connectivity.last_failure, None);
    assert_eq!(connectivity.recent_failures, 0);
}
