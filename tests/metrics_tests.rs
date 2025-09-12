//! Comprehensive tests for the metrics collection system

use dbfast::metrics::{MetricsCollector, MetricsConfig};
use std::time::Duration;

#[tokio::test]
async fn test_metrics_collector_creation() {
    let config = MetricsConfig {
        max_timing_samples: 1000,
        rate_window_minutes: 15,
        collect_system_metrics: true,
        min_duration_us: 0,
    };

    let collector = MetricsCollector::new(Some(config));
    let snapshot = collector.get_snapshot().unwrap();

    assert_eq!(snapshot.timings.len(), 0);
    assert_eq!(snapshot.counters.len(), 0);
    assert_eq!(snapshot.gauges.len(), 0);
}

#[tokio::test]
async fn test_timing_metrics() {
    let collector = MetricsCollector::new(None);

    // Record some timing metrics
    collector.record_timing("database_query", Duration::from_millis(150), None);
    collector.record_timing("database_query", Duration::from_millis(200), None);
    collector.record_timing("database_query", Duration::from_millis(100), None);

    let snapshot = collector.get_snapshot().unwrap();
    let timing_stats = snapshot.timings.get("database_query").unwrap();

    assert_eq!(timing_stats.count, 3);
    assert!(timing_stats.avg_ms > 140.0); // 140ms
    assert!(timing_stats.avg_ms < 160.0); // 160ms
    assert_eq!(timing_stats.min_ms, 100.0); // 100ms
    assert_eq!(timing_stats.max_ms, 200.0); // 200ms
}

#[tokio::test]
async fn test_counter_metrics() {
    let collector = MetricsCollector::new(None);

    // Record counter increments
    collector.increment_counter("api_requests", 1);
    collector.increment_counter("api_requests", 1);
    collector.increment_counter("api_requests", 3);

    let snapshot = collector.get_snapshot().unwrap();
    let counter_stats = snapshot.counters.get("api_requests").unwrap();

    assert_eq!(counter_stats.value, 5); // 1 + 1 + 3
}

#[tokio::test]
async fn test_gauge_metrics() {
    let collector = MetricsCollector::new(None);

    // Record gauge values
    collector.set_gauge("memory_usage", 75.5);
    collector.set_gauge("memory_usage", 80.2);
    collector.set_gauge("memory_usage", 72.1);

    let snapshot = collector.get_snapshot().unwrap();
    let gauge_stats = snapshot.gauges.get("memory_usage").unwrap();

    assert_eq!(gauge_stats.current, 72.1); // Last recorded value
}

#[tokio::test]
async fn test_timing_with_manual_record() {
    let collector = MetricsCollector::new(None);

    // Record timing manually
    collector.record_timing("test_operation", Duration::from_millis(50), None);

    let snapshot = collector.get_snapshot().unwrap();
    let timing_stats = snapshot.timings.get("test_operation").unwrap();

    assert_eq!(timing_stats.count, 1);
    assert_eq!(timing_stats.avg_ms, 50.0);
}

#[tokio::test]
async fn test_concurrent_metrics_collection() {
    let collector = MetricsCollector::new(None);
    let collector_clone1 = collector.clone();
    let collector_clone2 = collector.clone();

    // Spawn concurrent tasks
    let handle1 = tokio::spawn(async move {
        for i in 0..50 {
            collector_clone1.increment_counter("concurrent_test", 1);
            collector_clone1.record_timing("concurrent_timing", Duration::from_millis(i), None);
        }
    });

    let handle2 = tokio::spawn(async move {
        for i in 0..50 {
            collector_clone2.increment_counter("concurrent_test", 1);
            collector_clone2.set_gauge("concurrent_gauge", i as f64);
        }
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    let snapshot = collector.get_snapshot().unwrap();
    let counter_stats = snapshot.counters.get("concurrent_test").unwrap();

    assert_eq!(counter_stats.value, 100);
}

#[test]
fn test_metrics_config_defaults() {
    let config = MetricsConfig::default();

    assert_eq!(config.max_timing_samples, 1000);
    assert_eq!(config.rate_window_minutes, 5);
    assert!(config.collect_system_metrics);
    assert_eq!(config.min_duration_us, 100); // 0.1ms
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_high_volume_metrics_performance() {
        let collector = MetricsCollector::new(None);
        let start = Instant::now();

        // Record 10,000 metrics to test performance
        for i in 0..10_000 {
            collector.increment_counter("perf_test", 1);
            if i % 100 == 0 {
                collector.record_timing("perf_timing", Duration::from_millis(i % 200), None);
            }
        }

        let duration = start.elapsed();
        println!("Recorded 10,000 metrics in {:?}", duration);

        // Should complete reasonably quickly (under 1 second)
        assert!(duration < Duration::from_secs(1));

        let snapshot = collector.get_snapshot().unwrap();
        assert_eq!(snapshot.counters.get("perf_test").unwrap().value, 10_000);
    }
}
