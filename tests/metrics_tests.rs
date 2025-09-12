//! Comprehensive tests for the metrics collection system

use dbfast::metrics::{
    CounterStats, GaugeStats, MetricsCollector, MetricsConfig, MetricsSnapshot, SystemMetrics,
    TimingGuard, TimingStats,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_metrics_collector_creation() {
    let config = MetricsConfig {
        max_timing_samples: 1000,
        rate_window_minutes: 15,
        collect_system_metrics: true,
        retention_hours: 24,
    };

    let collector = MetricsCollector::new(config.clone());
    let snapshot = collector.snapshot().await;

    assert_eq!(snapshot.timing_stats.len(), 0);
    assert_eq!(snapshot.counter_stats.len(), 0);
    assert_eq!(snapshot.gauge_stats.len(), 0);
}

#[tokio::test]
async fn test_timing_metrics() {
    let collector = MetricsCollector::default();

    // Record some timing metrics
    collector
        .record_timing("database_query", Duration::from_millis(150), None)
        .await;
    collector
        .record_timing("database_query", Duration::from_millis(200), None)
        .await;
    collector
        .record_timing("database_query", Duration::from_millis(100), None)
        .await;

    let snapshot = collector.snapshot().await;
    let timing_stats = snapshot.timing_stats.get("database_query").unwrap();

    assert_eq!(timing_stats.total_calls, 3);
    assert!(timing_stats.avg_duration_ms > 140.0);
    assert!(timing_stats.avg_duration_ms < 160.0);
    assert_eq!(timing_stats.min_duration_ms, 100.0);
    assert_eq!(timing_stats.max_duration_ms, 200.0);
}

#[tokio::test]
async fn test_counter_metrics() {
    let collector = MetricsCollector::default();

    // Record counter increments
    collector.increment_counter("api_requests", None).await;
    collector.increment_counter("api_requests", None).await;
    collector
        .increment_counter_by("api_requests", 3, None)
        .await;

    let snapshot = collector.snapshot().await;
    let counter_stats = snapshot.counter_stats.get("api_requests").unwrap();

    assert_eq!(counter_stats.total_count, 5); // 1 + 1 + 3
    assert!(counter_stats.rate_per_minute > 0.0);
}

#[tokio::test]
async fn test_gauge_metrics() {
    let collector = MetricsCollector::default();

    // Record gauge values
    collector.set_gauge("memory_usage", 75.5, None).await;
    collector.set_gauge("memory_usage", 80.2, None).await;
    collector.set_gauge("memory_usage", 72.1, None).await;

    let snapshot = collector.snapshot().await;
    let gauge_stats = snapshot.gauge_stats.get("memory_usage").unwrap();

    assert_eq!(gauge_stats.current_value, 72.1); // Last recorded value
    assert_eq!(gauge_stats.min_value, 72.1);
    assert_eq!(gauge_stats.max_value, 80.2);
    assert!(gauge_stats.avg_value > 75.0 && gauge_stats.avg_value < 76.0);
}

#[tokio::test]
async fn test_timing_guard() {
    let collector = MetricsCollector::default();

    {
        let _guard = TimingGuard::new(collector.clone(), "test_operation".to_string(), None);
        sleep(Duration::from_millis(100)).await;
    } // Guard drops here, recording timing

    let snapshot = collector.snapshot().await;
    let timing_stats = snapshot.timing_stats.get("test_operation").unwrap();

    assert_eq!(timing_stats.total_calls, 1);
    assert!(timing_stats.avg_duration_ms >= 90.0); // Allow for some variance
}

#[tokio::test]
async fn test_timing_guard_with_tags() {
    let collector = MetricsCollector::default();

    let mut tags = std::collections::HashMap::new();
    tags.insert("endpoint".to_string(), "users".to_string());
    tags.insert("method".to_string(), "GET".to_string());

    {
        let _guard = TimingGuard::new(collector.clone(), "api_request".to_string(), Some(tags));
        sleep(Duration::from_millis(50)).await;
    }

    let snapshot = collector.snapshot().await;
    assert!(snapshot.timing_stats.contains_key("api_request"));
}

#[tokio::test]
async fn test_percentile_calculations() {
    let collector = MetricsCollector::default();

    // Record a range of timing values to test percentiles
    for i in 1..=100 {
        collector
            .record_timing("percentile_test", Duration::from_millis(i), None)
            .await;
    }

    let snapshot = collector.snapshot().await;
    let timing_stats = snapshot.timing_stats.get("percentile_test").unwrap();

    assert_eq!(timing_stats.total_calls, 100);
    assert!(timing_stats.p50_duration_ms >= 49.0 && timing_stats.p50_duration_ms <= 51.0);
    assert!(timing_stats.p95_duration_ms >= 94.0 && timing_stats.p95_duration_ms <= 96.0);
    assert!(timing_stats.p99_duration_ms >= 98.0 && timing_stats.p99_duration_ms <= 100.0);
}

#[tokio::test]
async fn test_metrics_reset() {
    let collector = MetricsCollector::default();

    // Record some metrics
    collector.increment_counter("test_counter", None).await;
    collector.set_gauge("test_gauge", 42.0, None).await;
    collector
        .record_timing("test_timing", Duration::from_millis(100), None)
        .await;

    // Verify metrics exist
    let snapshot = collector.snapshot().await;
    assert!(snapshot.counter_stats.contains_key("test_counter"));
    assert!(snapshot.gauge_stats.contains_key("test_gauge"));
    assert!(snapshot.timing_stats.contains_key("test_timing"));

    // Reset metrics
    collector.reset().await;

    // Verify metrics are cleared
    let snapshot = collector.snapshot().await;
    assert_eq!(snapshot.counter_stats.len(), 0);
    assert_eq!(snapshot.gauge_stats.len(), 0);
    assert_eq!(snapshot.timing_stats.len(), 0);
}

#[tokio::test]
async fn test_rate_calculations() {
    let collector = MetricsCollector::default();

    // Record events over time
    for _ in 0..10 {
        collector.increment_counter("rate_test", None).await;
    }

    sleep(Duration::from_millis(100)).await;

    let snapshot = collector.snapshot().await;
    let counter_stats = snapshot.counter_stats.get("rate_test").unwrap();

    assert_eq!(counter_stats.total_count, 10);
    assert!(counter_stats.rate_per_minute > 0.0);
    assert!(counter_stats.rate_per_second > 0.0);
}

#[tokio::test]
async fn test_concurrent_metrics_collection() {
    let collector = MetricsCollector::default();
    let collector_clone = collector.clone();

    // Spawn concurrent tasks
    let handle1 = tokio::spawn(async move {
        for i in 0..50 {
            collector.increment_counter("concurrent_test", None).await;
            collector
                .record_timing("concurrent_timing", Duration::from_millis(i), None)
                .await;
        }
    });

    let handle2 = tokio::spawn(async move {
        for i in 0..50 {
            collector_clone
                .increment_counter("concurrent_test", None)
                .await;
            collector_clone
                .set_gauge("concurrent_gauge", i as f64, None)
                .await;
        }
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    let snapshot = collector.snapshot().await;
    let counter_stats = snapshot.counter_stats.get("concurrent_test").unwrap();

    assert_eq!(counter_stats.total_count, 100);
}

#[tokio::test]
async fn test_system_metrics_collection() {
    let config = MetricsConfig {
        max_timing_samples: 1000,
        rate_window_minutes: 15,
        collect_system_metrics: true,
        retention_hours: 24,
    };

    let collector = MetricsCollector::new(config);
    collector.collect_system_metrics().await;

    let snapshot = collector.snapshot().await;

    // System metrics should be collected as gauges
    assert!(snapshot.gauge_stats.len() > 0);
}

#[test]
fn test_metrics_config_defaults() {
    let config = MetricsConfig::default();

    assert_eq!(config.max_timing_samples, 1000);
    assert_eq!(config.rate_window_minutes, 15);
    assert!(config.collect_system_metrics);
    assert_eq!(config.retention_hours, 24);
}

#[tokio::test]
async fn test_metrics_snapshot_serialization() {
    let collector = MetricsCollector::default();

    collector.increment_counter("test", None).await;
    collector.set_gauge("test_gauge", 42.0, None).await;
    collector
        .record_timing("test_timing", Duration::from_millis(100), None)
        .await;

    let snapshot = collector.snapshot().await;

    // Test that snapshot can be serialized (for monitoring/alerting systems)
    let json = serde_json::to_string(&snapshot);
    assert!(json.is_ok());
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_high_volume_metrics_performance() {
        let collector = MetricsCollector::default();
        let start = Instant::now();

        // Record 10,000 metrics to test performance
        for i in 0..10_000 {
            collector.increment_counter("perf_test", None).await;
            if i % 100 == 0 {
                collector
                    .record_timing("perf_timing", Duration::from_millis(i % 200), None)
                    .await;
            }
        }

        let duration = start.elapsed();
        println!("Recorded 10,000 metrics in {:?}", duration);

        // Should complete reasonably quickly (under 1 second)
        assert!(duration < Duration::from_secs(1));

        let snapshot = collector.snapshot().await;
        assert_eq!(
            snapshot.counter_stats.get("perf_test").unwrap().total_count,
            10_000
        );
    }
}
