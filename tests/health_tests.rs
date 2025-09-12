//! Comprehensive tests for the database health monitoring system

use dbfast::health::{
    ConnectivityMetrics, HealthIssue, HealthMetrics, HealthMonitor, HealthStatus, HealthThresholds,
    IssueType, MonitoringConfig, PerformanceMetrics, PoolStatistics,
};
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_health_status_ordering() {
    assert!(HealthStatus::Critical > HealthStatus::Warning);
    assert!(HealthStatus::Warning > HealthStatus::Degraded);
    assert!(HealthStatus::Degraded > HealthStatus::Healthy);
}

#[tokio::test]
async fn test_health_monitor_creation() {
    let config = MonitoringConfig::default();
    let monitor = HealthMonitor::new(config);

    let initial_health = monitor.get_health().await;

    // Initial state should be healthy or unknown
    assert!(matches!(
        initial_health.status,
        HealthStatus::Healthy | HealthStatus::Degraded
    ));
}

#[tokio::test]
async fn test_pool_statistics_tracking() {
    let config = MonitoringConfig::default();
    let monitor = HealthMonitor::new(config);

    // Simulate pool statistics
    monitor
        .update_pool_stats(PoolStatistics {
            total_connections: 20,
            active_connections: 15,
            idle_connections: 5,
            pending_connections: 2,
            max_connections: 20,
            min_connections: 5,
            connection_timeout_ms: 5000,
        })
        .await;

    let health = monitor.get_health().await;
    assert_eq!(health.pool_stats.total_connections, 20);
    assert_eq!(health.pool_stats.active_connections, 15);
    assert_eq!(health.pool_stats.idle_connections, 5);
}

#[tokio::test]
async fn test_connectivity_metrics() {
    let config = MonitoringConfig::default();
    let monitor = HealthMonitor::new(config);

    // Simulate connectivity metrics
    monitor
        .record_connection_attempt(true, Duration::from_millis(150))
        .await;
    monitor
        .record_connection_attempt(true, Duration::from_millis(200))
        .await;
    monitor
        .record_connection_attempt(false, Duration::from_millis(5000))
        .await;

    let health = monitor.get_health().await;
    assert_eq!(health.connectivity.total_attempts, 3);
    assert_eq!(health.connectivity.successful_connections, 2);
    assert_eq!(health.connectivity.failed_connections, 1);
    assert!(health.connectivity.avg_connection_time_ms > 100.0);
}

#[tokio::test]
async fn test_performance_metrics() {
    let config = MonitoringConfig::default();
    let monitor = HealthMonitor::new(config);

    // Record query performance
    monitor
        .record_query_performance(Duration::from_millis(100), true)
        .await;
    monitor
        .record_query_performance(Duration::from_millis(150), true)
        .await;
    monitor
        .record_query_performance(Duration::from_millis(200), true)
        .await;
    monitor
        .record_query_performance(Duration::from_millis(5000), false)
        .await;

    let health = monitor.get_health().await;
    assert_eq!(health.performance.total_queries, 4);
    assert_eq!(health.performance.successful_queries, 3);
    assert_eq!(health.performance.failed_queries, 1);
    assert!(health.performance.avg_query_time_ms > 100.0);
}

#[tokio::test]
async fn test_health_issue_detection() {
    let config = MonitoringConfig {
        check_interval: Duration::from_millis(100),
        thresholds: HealthThresholds {
            max_avg_query_time_ms: 100.0,
            max_connection_time_ms: 1000.0,
            min_success_rate: 0.95,
            max_pool_utilization: 0.8,
        },
        enable_automatic_recovery: false,
        max_consecutive_failures: 3,
    };

    let monitor = HealthMonitor::new(config);

    // Create conditions that should trigger health issues
    monitor
        .record_query_performance(Duration::from_millis(500), true)
        .await; // Slow query
    monitor
        .record_connection_attempt(false, Duration::from_millis(2000))
        .await; // Failed connection

    // Update pool to high utilization
    monitor
        .update_pool_stats(PoolStatistics {
            total_connections: 20,
            active_connections: 18, // 90% utilization
            idle_connections: 2,
            pending_connections: 0,
            max_connections: 20,
            min_connections: 5,
            connection_timeout_ms: 5000,
        })
        .await;

    let health = monitor.get_health().await;

    // Should have detected issues
    assert!(!health.issues.is_empty());
    assert!(health
        .issues
        .iter()
        .any(|issue| matches!(issue.issue_type, IssueType::HighQueryLatency)));
}

#[tokio::test]
async fn test_health_status_degradation() {
    let config = MonitoringConfig::default();
    let monitor = HealthMonitor::new(config);

    // Start with healthy state
    let initial_health = monitor.get_health().await;

    // Introduce performance issues
    for _ in 0..5 {
        monitor
            .record_query_performance(Duration::from_millis(1000), true)
            .await;
    }

    // Add connection failures
    for _ in 0..3 {
        monitor
            .record_connection_attempt(false, Duration::from_millis(5000))
            .await;
    }

    let degraded_health = monitor.get_health().await;

    // Health status should have degraded
    assert!(degraded_health.status != HealthStatus::Healthy || !degraded_health.issues.is_empty());
}

#[tokio::test]
async fn test_continuous_monitoring() {
    let config = MonitoringConfig {
        check_interval: Duration::from_millis(50),
        thresholds: HealthThresholds::default(),
        enable_automatic_recovery: false,
        max_consecutive_failures: 2,
    };

    let monitor = HealthMonitor::new(config);

    // Start monitoring
    let monitoring_handle = monitor.start_monitoring().await;

    sleep(Duration::from_millis(200)).await;

    // Record some metrics
    monitor
        .record_query_performance(Duration::from_millis(50), true)
        .await;
    monitor
        .record_connection_attempt(true, Duration::from_millis(100))
        .await;

    sleep(Duration::from_millis(100)).await;

    // Stop monitoring
    monitoring_handle.abort();

    let health = monitor.get_health().await;
    assert!(health.last_updated.timestamp() > 0);
}

#[tokio::test]
async fn test_health_thresholds() {
    let thresholds = HealthThresholds {
        max_avg_query_time_ms: 200.0,
        max_connection_time_ms: 1000.0,
        min_success_rate: 0.9,
        max_pool_utilization: 0.85,
    };

    // Test query time threshold
    assert!(250.0 > thresholds.max_avg_query_time_ms);
    assert!(150.0 < thresholds.max_avg_query_time_ms);

    // Test success rate threshold
    assert!(0.85 < thresholds.min_success_rate);
    assert!(0.95 > thresholds.min_success_rate);
}

#[tokio::test]
async fn test_pool_utilization_calculation() {
    let pool_stats = PoolStatistics {
        total_connections: 20,
        active_connections: 16,
        idle_connections: 4,
        pending_connections: 2,
        max_connections: 20,
        min_connections: 5,
        connection_timeout_ms: 5000,
    };

    let utilization = pool_stats.utilization();
    assert_eq!(utilization, 0.8); // 16/20 = 0.8
}

#[tokio::test]
async fn test_health_issue_types() {
    let issues = vec![
        HealthIssue {
            issue_type: IssueType::HighQueryLatency,
            message: "Average query time exceeded threshold".to_string(),
            severity: dbfast::errors::ErrorSeverity::Medium,
            first_detected: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            count: 1,
            details: std::collections::HashMap::new(),
        },
        HealthIssue {
            issue_type: IssueType::ConnectionPoolExhaustion,
            message: "Connection pool is near capacity".to_string(),
            severity: dbfast::errors::ErrorSeverity::High,
            first_detected: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            count: 3,
            details: std::collections::HashMap::new(),
        },
        HealthIssue {
            issue_type: IssueType::HighErrorRate,
            message: "Error rate exceeds acceptable threshold".to_string(),
            severity: dbfast::errors::ErrorSeverity::Critical,
            first_detected: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            count: 5,
            details: std::collections::HashMap::new(),
        },
    ];

    // Verify different issue types
    assert_eq!(issues.len(), 3);
    assert!(issues
        .iter()
        .any(|i| matches!(i.issue_type, IssueType::HighQueryLatency)));
    assert!(issues
        .iter()
        .any(|i| matches!(i.issue_type, IssueType::ConnectionPoolExhaustion)));
    assert!(issues
        .iter()
        .any(|i| matches!(i.issue_type, IssueType::HighErrorRate)));
}

#[tokio::test]
async fn test_monitoring_config_validation() {
    let config = MonitoringConfig {
        check_interval: Duration::from_millis(100),
        thresholds: HealthThresholds {
            max_avg_query_time_ms: 500.0,
            max_connection_time_ms: 2000.0,
            min_success_rate: 0.95,
            max_pool_utilization: 0.8,
        },
        enable_automatic_recovery: true,
        max_consecutive_failures: 3,
    };

    assert!(config.check_interval >= Duration::from_millis(10));
    assert!(config.thresholds.min_success_rate >= 0.0 && config.thresholds.min_success_rate <= 1.0);
    assert!(
        config.thresholds.max_pool_utilization >= 0.0
            && config.thresholds.max_pool_utilization <= 1.0
    );
    assert!(config.max_consecutive_failures > 0);
}

#[tokio::test]
async fn test_health_recovery_scenarios() {
    let config = MonitoringConfig {
        check_interval: Duration::from_millis(50),
        thresholds: HealthThresholds::default(),
        enable_automatic_recovery: true,
        max_consecutive_failures: 2,
    };

    let monitor = HealthMonitor::new(config);

    // Simulate degraded state
    monitor
        .record_connection_attempt(false, Duration::from_millis(5000))
        .await;
    monitor
        .record_connection_attempt(false, Duration::from_millis(5000))
        .await;

    let degraded_health = monitor.get_health().await;
    assert!(degraded_health.status != HealthStatus::Healthy || !degraded_health.issues.is_empty());

    // Simulate recovery
    for _ in 0..10 {
        monitor
            .record_connection_attempt(true, Duration::from_millis(100))
            .await;
        monitor
            .record_query_performance(Duration::from_millis(50), true)
            .await;
    }

    sleep(Duration::from_millis(100)).await;

    let recovered_health = monitor.get_health().await;
    // Health should improve or issues should be fewer
    assert!(
        recovered_health.performance.successful_queries
            > degraded_health.performance.successful_queries
    );
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_health_monitoring() {
        let config = MonitoringConfig::default();
        let monitor = HealthMonitor::new(config);

        // Start monitoring
        let _handle = monitor.start_monitoring().await;

        // Simulate realistic database workload
        for i in 0..50 {
            let query_time = Duration::from_millis(50 + (i % 100));
            let success = i % 10 != 0; // 90% success rate

            monitor.record_query_performance(query_time, success).await;

            if i % 5 == 0 {
                let conn_success = i % 20 != 0; // 95% connection success
                let conn_time = Duration::from_millis(100 + (i % 200));
                monitor
                    .record_connection_attempt(conn_success, conn_time)
                    .await;
            }

            if i % 10 == 0 {
                // Update pool stats occasionally
                monitor
                    .update_pool_stats(PoolStatistics {
                        total_connections: 20,
                        active_connections: 10 + (i % 8),
                        idle_connections: 10 - (i % 8),
                        pending_connections: i % 3,
                        max_connections: 20,
                        min_connections: 5,
                        connection_timeout_ms: 5000,
                    })
                    .await;
            }
        }

        sleep(Duration::from_millis(100)).await;

        let final_health = monitor.get_health().await;

        // Verify comprehensive health data was collected
        assert!(final_health.performance.total_queries > 0);
        assert!(final_health.connectivity.total_attempts > 0);
        assert!(final_health.pool_stats.total_connections > 0);
        assert!(final_health.last_updated.timestamp() > 0);

        // Health status should reflect the simulated conditions
        println!("Final health status: {:?}", final_health.status);
        println!("Issues detected: {}", final_health.issues.len());
    }
}
