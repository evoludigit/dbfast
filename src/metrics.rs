//! Performance metrics collection and monitoring system
//!
//! This module provides comprehensive metrics collection for:
//! - Operation timing and performance
//! - Resource utilization tracking
//! - Error rate monitoring
//! - System health indicators

use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Central metrics collector for the application
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<Mutex<MetricsCollectorInner>>,
}

/// Internal metrics storage
struct MetricsCollectorInner {
    /// Timing metrics for operations
    timings: HashMap<String, TimingMetrics>,

    /// Counter metrics for events
    counters: HashMap<String, CounterMetrics>,

    /// Gauge metrics for current values
    gauges: HashMap<String, GaugeMetrics>,

    /// System metrics
    system_metrics: SystemMetrics,

    /// Configuration for metrics collection
    config: MetricsConfig,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Maximum number of timing samples to keep per operation
    pub max_timing_samples: usize,

    /// Time window for rate calculations (minutes)
    pub rate_window_minutes: u64,

    /// Whether to collect detailed system metrics
    pub collect_system_metrics: bool,

    /// Minimum operation duration to track (microseconds)
    pub min_duration_us: u64,
}

/// Timing metrics for operations
#[derive(Debug, Clone)]
pub struct TimingMetrics {
    /// Operation name
    pub name: String,

    /// Recent timing samples (duration in microseconds)
    pub samples: VecDeque<TimingSample>,

    /// Total number of operations recorded
    pub total_count: u64,

    /// Sum of all durations for average calculation
    pub total_duration_us: u64,

    /// Minimum duration observed
    pub min_duration_us: u64,

    /// Maximum duration observed
    pub max_duration_us: u64,

    /// Last operation timestamp
    pub last_operation: DateTime<Utc>,
}

/// Individual timing sample
#[derive(Debug, Clone)]
pub struct TimingSample {
    /// Duration in microseconds
    pub duration_us: u64,

    /// When this sample was recorded
    pub timestamp: DateTime<Utc>,

    /// Additional context tags
    pub tags: HashMap<String, String>,
}

/// Counter metrics for discrete events
#[derive(Debug, Clone)]
pub struct CounterMetrics {
    /// Counter name
    pub name: String,

    /// Current value
    pub value: u64,

    /// Rate per minute (calculated over window)
    pub rate_per_minute: f64,

    /// Recent events for rate calculation
    pub recent_events: VecDeque<DateTime<Utc>>,

    /// Last increment timestamp
    pub last_increment: DateTime<Utc>,
}

/// Gauge metrics for current values
#[derive(Debug, Clone)]
pub struct GaugeMetrics {
    /// Gauge name
    pub name: String,

    /// Current value
    pub value: f64,

    /// Recent values for trending
    pub recent_values: VecDeque<(DateTime<Utc>, f64)>,

    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// System-level metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Memory usage information
    pub memory: MemoryMetrics,

    /// Application uptime
    pub uptime: Duration,

    /// Application start time
    pub start_time: DateTime<Utc>,
}

/// Memory usage metrics
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)] // Consistent naming with 'bytes' suffix for clarity
pub struct MemoryMetrics {
    /// Current memory usage in bytes
    pub current_usage_bytes: Option<u64>,

    /// Peak memory usage in bytes
    pub peak_usage_bytes: Option<u64>,

    /// Available system memory in bytes
    pub available_bytes: Option<u64>,
}

/// Metrics query result
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,

    /// All timing metrics
    pub timings: HashMap<String, TimingStats>,

    /// All counter metrics
    pub counters: HashMap<String, CounterStats>,

    /// All gauge metrics
    pub gauges: HashMap<String, GaugeStats>,

    /// System metrics
    pub system: SystemMetrics,
}

/// Aggregated timing statistics
#[derive(Debug, Clone)]
pub struct TimingStats {
    /// Total number of operations
    pub count: u64,

    /// Average duration in milliseconds
    pub avg_ms: f64,

    /// Minimum duration in milliseconds
    pub min_ms: f64,

    /// Maximum duration in milliseconds
    pub max_ms: f64,

    /// 95th percentile duration in milliseconds
    pub p95_ms: f64,

    /// 99th percentile duration in milliseconds
    pub p99_ms: f64,

    /// Operations per second
    pub ops_per_sec: f64,
}

/// Counter statistics
#[derive(Debug, Clone)]
pub struct CounterStats {
    /// Current counter value
    pub value: u64,

    /// Rate per minute
    pub rate_per_minute: f64,

    /// Rate per second
    pub rate_per_second: f64,
}

/// Gauge statistics
#[derive(Debug, Clone)]
pub struct GaugeStats {
    /// Current value
    pub current: f64,

    /// Average over recent period
    pub average: f64,

    /// Minimum recent value
    pub min: f64,

    /// Maximum recent value
    pub max: f64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_timing_samples: 1000,
            rate_window_minutes: 5,
            collect_system_metrics: true,
            min_duration_us: 100, // 0.1ms minimum
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new(config: Option<MetricsConfig>) -> Self {
        let config = config.unwrap_or_default();

        Self {
            inner: Arc::new(Mutex::new(MetricsCollectorInner {
                timings: HashMap::new(),
                counters: HashMap::new(),
                gauges: HashMap::new(),
                system_metrics: SystemMetrics {
                    memory: MemoryMetrics {
                        current_usage_bytes: None,
                        peak_usage_bytes: None,
                        available_bytes: None,
                    },
                    uptime: Duration::from_secs(0),
                    start_time: Utc::now(),
                },
                config,
            })),
        }
    }

    /// Record a timing for an operation
    pub fn record_timing(
        &self,
        operation: &str,
        duration: Duration,
        tags: Option<HashMap<String, String>>,
    ) {
        #[allow(clippy::cast_possible_truncation)] // Microsecond precision sufficient for timing
        let duration_us = duration.as_micros() as u64;

        if let Ok(mut inner) = self.inner.lock() {
            // Skip very fast operations if configured
            let min_duration = inner.config.min_duration_us;
            let max_samples = inner.config.max_timing_samples;

            if duration_us < min_duration {
                return;
            }

            let timing_metrics = inner
                .timings
                .entry(operation.to_string())
                .or_insert_with(|| TimingMetrics {
                    name: operation.to_string(),
                    samples: VecDeque::new(),
                    total_count: 0,
                    total_duration_us: 0,
                    min_duration_us: u64::MAX,
                    max_duration_us: 0,
                    last_operation: Utc::now(),
                });

            // Add sample
            let sample = TimingSample {
                duration_us,
                timestamp: Utc::now(),
                tags: tags.unwrap_or_default(),
            };

            timing_metrics.samples.push_back(sample);

            // Keep only recent samples
            while timing_metrics.samples.len() > max_samples {
                timing_metrics.samples.pop_front();
            }

            // Update aggregates
            timing_metrics.total_count += 1;
            timing_metrics.total_duration_us += duration_us;
            timing_metrics.min_duration_us = timing_metrics.min_duration_us.min(duration_us);
            timing_metrics.max_duration_us = timing_metrics.max_duration_us.max(duration_us);
            timing_metrics.last_operation = Utc::now();

            debug!("Recorded timing for {}: {}μs", operation, duration_us);
        }
    }

    /// Increment a counter
    pub fn increment_counter(&self, counter: &str, increment: u64) {
        if let Ok(mut inner) = self.inner.lock() {
            let rate_window = inner.config.rate_window_minutes;

            let counter_metrics = inner
                .counters
                .entry(counter.to_string())
                .or_insert_with(|| CounterMetrics {
                    name: counter.to_string(),
                    value: 0,
                    rate_per_minute: 0.0,
                    recent_events: VecDeque::new(),
                    last_increment: Utc::now(),
                });

            counter_metrics.value += increment;
            counter_metrics.last_increment = Utc::now();

            // Add events for rate calculation
            for _ in 0..increment {
                counter_metrics.recent_events.push_back(Utc::now());
            }

            // Clean old events
            #[allow(clippy::cast_possible_wrap)]
            let cutoff = Utc::now() - chrono::Duration::minutes(rate_window as i64);
            while let Some(&front_time) = counter_metrics.recent_events.front() {
                if front_time < cutoff {
                    counter_metrics.recent_events.pop_front();
                } else {
                    break;
                }
            }

            // Update rate
            #[allow(clippy::cast_precision_loss)]
            {
                counter_metrics.rate_per_minute = counter_metrics.recent_events.len() as f64;
            }

            debug!(
                "Incremented counter {}: +{} (total: {})",
                counter, increment, counter_metrics.value
            );
        }
    }

    /// Set a gauge value
    pub fn set_gauge(&self, gauge: &str, value: f64) {
        if let Ok(mut inner) = self.inner.lock() {
            let rate_window = inner.config.rate_window_minutes;

            let gauge_metrics =
                inner
                    .gauges
                    .entry(gauge.to_string())
                    .or_insert_with(|| GaugeMetrics {
                        name: gauge.to_string(),
                        value: 0.0,
                        recent_values: VecDeque::new(),
                        last_update: Utc::now(),
                    });

            gauge_metrics.value = value;
            gauge_metrics.last_update = Utc::now();
            gauge_metrics.recent_values.push_back((Utc::now(), value));

            // Keep only recent values
            #[allow(clippy::cast_possible_wrap)]
            let cutoff = Utc::now() - chrono::Duration::minutes(rate_window as i64);
            while let Some(&(timestamp, _)) = gauge_metrics.recent_values.front() {
                if timestamp < cutoff {
                    gauge_metrics.recent_values.pop_front();
                } else {
                    break;
                }
            }

            debug!("Set gauge {}: {}", gauge, value);
        }
    }

    /// Get a snapshot of all current metrics
    #[must_use]
    pub fn get_snapshot(&self) -> Option<MetricsSnapshot> {
        self.inner.lock().map_or(None, |inner| {
            let mut timing_stats = HashMap::new();
            for (name, metrics) in &inner.timings {
                timing_stats.insert(name.clone(), Self::calculate_timing_stats(metrics));
            }

            let mut counter_stats = HashMap::new();
            for (name, metrics) in &inner.counters {
                counter_stats.insert(
                    name.clone(),
                    CounterStats {
                        value: metrics.value,
                        rate_per_minute: metrics.rate_per_minute,
                        rate_per_second: metrics.rate_per_minute / 60.0,
                    },
                );
            }

            let mut gauge_stats = HashMap::new();
            for (name, metrics) in &inner.gauges {
                gauge_stats.insert(name.clone(), Self::calculate_gauge_stats(metrics));
            }

            // Update system metrics
            let mut system_metrics = inner.system_metrics.clone();
            system_metrics.uptime = Utc::now()
                .signed_duration_since(system_metrics.start_time)
                .to_std()
                .unwrap_or(Duration::from_secs(0));

            Some(MetricsSnapshot {
                timestamp: Utc::now(),
                timings: timing_stats,
                counters: counter_stats,
                gauges: gauge_stats,
                system: system_metrics,
            })
        })
    }

    /// Calculate timing statistics
    fn calculate_timing_stats(metrics: &TimingMetrics) -> TimingStats {
        if metrics.samples.is_empty() {
            return TimingStats {
                count: metrics.total_count,
                avg_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                ops_per_sec: 0.0,
            };
        }

        let mut durations: Vec<u64> = metrics.samples.iter().map(|s| s.duration_us).collect();
        durations.sort_unstable();

        #[allow(clippy::cast_precision_loss)]
        let avg_ms = (metrics.total_duration_us as f64 / metrics.total_count as f64) / 1000.0;
        #[allow(clippy::cast_precision_loss)]
        let min_ms = metrics.min_duration_us as f64 / 1000.0;
        #[allow(clippy::cast_precision_loss)]
        let max_ms = metrics.max_duration_us as f64 / 1000.0;

        // Calculate percentiles
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let p95_index = (durations.len() as f64 * 0.95) as usize;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let p99_index = (durations.len() as f64 * 0.99) as usize;

        let p95_ms = durations
            .get(p95_index.min(durations.len() - 1))
            .map_or(0.0, |&d| {
                #[allow(clippy::cast_precision_loss)]
                {
                    d as f64 / 1000.0
                }
            });

        let p99_ms = durations
            .get(p99_index.min(durations.len() - 1))
            .map_or(0.0, |&d| {
                #[allow(clippy::cast_precision_loss)]
                {
                    d as f64 / 1000.0
                }
            });

        // Calculate ops per second based on recent samples
        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let recent_count = metrics
            .samples
            .iter()
            .filter(|s| s.timestamp > one_minute_ago)
            .count();
        #[allow(clippy::cast_precision_loss)]
        let ops_per_sec = recent_count as f64 / 60.0;

        TimingStats {
            count: metrics.total_count,
            avg_ms,
            min_ms,
            max_ms,
            p95_ms,
            p99_ms,
            ops_per_sec,
        }
    }

    /// Calculate gauge statistics
    fn calculate_gauge_stats(metrics: &GaugeMetrics) -> GaugeStats {
        if metrics.recent_values.is_empty() {
            return GaugeStats {
                current: metrics.value,
                average: metrics.value,
                min: metrics.value,
                max: metrics.value,
            };
        }

        let values: Vec<f64> = metrics.recent_values.iter().map(|(_, v)| *v).collect();
        let sum: f64 = values.iter().sum();
        #[allow(clippy::cast_precision_loss)]
        let average = sum / values.len() as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        GaugeStats {
            current: metrics.value,
            average,
            min,
            max,
        }
    }

    /// Log current metrics summary
    pub fn log_summary(&self) {
        if let Some(snapshot) = self.get_snapshot() {
            info!("=== Metrics Summary ===");
            info!("Uptime: {:?}", snapshot.system.uptime);

            for (name, timing) in &snapshot.timings {
                info!(
                    "Timing {}: count={}, avg={:.2}ms, p95={:.2}ms, ops/sec={:.1}",
                    name, timing.count, timing.avg_ms, timing.p95_ms, timing.ops_per_sec
                );
            }

            for (name, counter) in &snapshot.counters {
                info!(
                    "Counter {}: value={}, rate={:.1}/min",
                    name, counter.value, counter.rate_per_minute
                );
            }

            for (name, gauge) in &snapshot.gauges {
                info!(
                    "Gauge {}: current={:.2}, avg={:.2}",
                    name, gauge.current, gauge.average
                );
            }
        }
    }

    /// Get metrics for a specific operation
    #[must_use]
    pub fn get_operation_metrics(&self, operation: &str) -> Option<TimingStats> {
        self.inner.lock().map_or(None, |inner| {
            inner
                .timings
                .get(operation)
                .map(Self::calculate_timing_stats)
        })
    }
}

/// Utility for automatically timing operations
pub struct TimingGuard {
    collector: MetricsCollector,
    operation: String,
    start_time: Instant,
    tags: Option<HashMap<String, String>>,
}

impl TimingGuard {
    /// Start timing an operation
    #[must_use]
    pub fn new(
        collector: MetricsCollector,
        operation: String,
        tags: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            collector,
            operation,
            start_time: Instant::now(),
            tags,
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.collector
            .record_timing(&self.operation, duration, self.tags.take());
    }
}

/// Convenience macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($collector:expr, $operation:expr, $block:block) => {{
        let _guard =
            $crate::metrics::TimingGuard::new($collector.clone(), $operation.to_string(), None);
        $block
    }};

    ($collector:expr, $operation:expr, $tags:expr, $block:block) => {{
        let _guard = $crate::metrics::TimingGuard::new(
            $collector.clone(),
            $operation.to_string(),
            Some($tags),
        );
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(None);
        let snapshot = collector.get_snapshot().unwrap();

        assert!(snapshot.timings.is_empty());
        assert!(snapshot.counters.is_empty());
        assert!(snapshot.gauges.is_empty());
    }

    #[test]
    #[allow(clippy::float_cmp)] // Test values are exact
    fn test_timing_metrics() {
        let collector = MetricsCollector::new(None);

        // Record some timings
        collector.record_timing("test_operation", Duration::from_millis(100), None);
        collector.record_timing("test_operation", Duration::from_millis(200), None);

        let snapshot = collector.get_snapshot().unwrap();
        let timing = snapshot.timings.get("test_operation").unwrap();

        assert_eq!(timing.count, 2);
        assert_eq!(timing.avg_ms, 150.0);
        assert_eq!(timing.min_ms, 100.0);
        assert_eq!(timing.max_ms, 200.0);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Test values are exact
    fn test_counter_metrics() {
        let collector = MetricsCollector::new(None);

        collector.increment_counter("test_counter", 5);
        collector.increment_counter("test_counter", 3);

        let snapshot = collector.get_snapshot().unwrap();
        let counter = snapshot.counters.get("test_counter").unwrap();

        assert_eq!(counter.value, 8);
        assert_eq!(counter.rate_per_minute, 8.0);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Test values are exact
    fn test_gauge_metrics() {
        let collector = MetricsCollector::new(None);

        collector.set_gauge("test_gauge", 10.0);
        collector.set_gauge("test_gauge", 20.0);
        collector.set_gauge("test_gauge", 15.0);

        let snapshot = collector.get_snapshot().unwrap();
        let gauge = snapshot.gauges.get("test_gauge").unwrap();

        assert_eq!(gauge.current, 15.0);
        assert_eq!(gauge.average, 15.0);
        assert_eq!(gauge.min, 10.0);
        assert_eq!(gauge.max, 20.0);
    }

    #[test]
    fn test_timing_guard() {
        let collector = MetricsCollector::new(None);

        {
            let _guard = TimingGuard::new(collector.clone(), "auto_timed".to_string(), None);
            thread::sleep(Duration::from_millis(10));
        }

        let snapshot = collector.get_snapshot().unwrap();
        let timing = snapshot.timings.get("auto_timed").unwrap();

        assert_eq!(timing.count, 1);
        assert!(timing.avg_ms >= 10.0);
    }
}
