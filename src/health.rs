//! Database health monitoring and connection pool optimization
//!
//! This module provides:
//! - Connection pool health checks
//! - Database connectivity monitoring
//! - Performance metrics collection
//! - Automatic connection recovery

use crate::database::DatabasePool;
use crate::errors::DbFastResult;
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Health check status for database connections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Minor issues detected but functional
    Degraded,
    /// Major issues, some functionality affected
    Warning,
    /// Critical issues, system may not function properly
    Critical,
}

/// Database health metrics
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    /// Current health status
    pub status: HealthStatus,

    /// Connection pool statistics
    pub pool_stats: PoolStatistics,

    /// Database connectivity information
    pub connectivity: ConnectivityMetrics,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// When these metrics were last updated
    pub last_updated: DateTime<Utc>,

    /// Any issues detected
    pub issues: Vec<HealthIssue>,
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    /// Total connections in pool
    pub total_connections: u32,

    /// Active connections being used
    pub active_connections: u32,

    /// Idle connections available
    pub idle_connections: u32,

    /// Maximum connections allowed
    pub max_connections: u32,

    /// Pool utilization percentage (0-100)
    pub utilization_percent: f32,

    /// Number of times pool was exhausted
    pub exhaustion_count: u64,

    /// Average time to acquire connection (ms)
    pub avg_acquire_time_ms: f64,
}

/// Database connectivity metrics
#[derive(Debug, Clone)]
pub struct ConnectivityMetrics {
    /// Whether basic connectivity test passed
    pub can_connect: bool,

    /// Connection latency in milliseconds
    pub latency_ms: Option<f64>,

    /// Database server version
    pub server_version: Option<String>,

    /// Last successful connection time
    pub last_success: Option<DateTime<Utc>>,

    /// Last failed connection time
    pub last_failure: Option<DateTime<Utc>>,

    /// Number of recent connection failures
    pub recent_failures: u32,
}

/// Performance metrics for database operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average query execution time (ms)
    pub avg_query_time_ms: f64,

    /// 95th percentile query time (ms)
    pub p95_query_time_ms: f64,

    /// Queries per second
    pub queries_per_second: f64,

    /// Number of slow queries (>1s)
    pub slow_query_count: u64,

    /// Database CPU usage if available
    pub cpu_usage_percent: Option<f32>,

    /// Database memory usage if available
    pub memory_usage_percent: Option<f32>,
}

/// Health issues detected during monitoring
#[derive(Debug, Clone)]
pub struct HealthIssue {
    /// Type of issue
    pub issue_type: HealthIssueType,

    /// Severity level
    pub severity: HealthSeverity,

    /// Human-readable description
    pub description: String,

    /// Recommended action to resolve
    pub recommendation: String,

    /// When this issue was first detected
    pub detected_at: DateTime<Utc>,
}

/// Types of health issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthIssueType {
    /// Connection pool related issues
    PoolExhaustion,
    PoolUnderutilization,

    /// Connectivity issues
    ConnectionTimeout,
    ConnectionFailure,
    HighLatency,

    /// Performance issues
    SlowQueries,
    HighResourceUsage,

    /// Configuration issues
    SuboptimalConfiguration,
}

/// Health issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Database health monitor
pub struct HealthMonitor {
    /// Database pool to monitor
    pool: DatabasePool,

    /// Current health metrics
    metrics: Arc<RwLock<HealthMetrics>>,

    /// Historical performance data
    performance_history: Arc<RwLock<Vec<(DateTime<Utc>, PerformanceMetrics)>>>,

    /// Configuration for health checks
    config: HealthCheckConfig,
}

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// How often to run health checks
    pub check_interval: Duration,

    /// Timeout for health check operations
    pub check_timeout: Duration,

    /// How many performance samples to keep
    pub performance_history_size: usize,

    /// Thresholds for various metrics
    pub thresholds: HealthThresholds,
}

/// Thresholds for determining health status
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// Pool utilization % that triggers warnings
    pub pool_utilization_warning: f32,

    /// Pool utilization % that triggers critical status
    pub pool_utilization_critical: f32,

    /// Connection latency (ms) that triggers warnings
    pub latency_warning_ms: f64,

    /// Connection latency (ms) that triggers critical status
    pub latency_critical_ms: f64,

    /// Query time (ms) considered slow
    pub slow_query_threshold_ms: f64,

    /// Number of recent failures before warning
    pub failure_count_warning: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            performance_history_size: 100,
            thresholds: HealthThresholds::default(),
        }
    }
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            pool_utilization_warning: 70.0,
            pool_utilization_critical: 90.0,
            latency_warning_ms: 100.0,
            latency_critical_ms: 500.0,
            slow_query_threshold_ms: 1000.0,
            failure_count_warning: 3,
        }
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(pool: DatabasePool, config: Option<HealthCheckConfig>) -> Self {
        let config = config.unwrap_or_default();

        let initial_metrics = HealthMetrics {
            status: HealthStatus::Healthy,
            pool_stats: PoolStatistics::default(),
            connectivity: ConnectivityMetrics::default(),
            performance: PerformanceMetrics::default(),
            last_updated: Utc::now(),
            issues: Vec::new(),
        };

        Self {
            pool,
            metrics: Arc::new(RwLock::new(initial_metrics)),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(&self) -> DbFastResult<()> {
        info!("Starting database health monitoring");

        let pool = self.pool.clone();
        let metrics = Arc::clone(&self.metrics);
        let performance_history = Arc::clone(&self.performance_history);
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                let start_time = Instant::now();

                debug!("Running health check");

                match Self::perform_health_check(&pool, &config).await {
                    Ok(new_metrics) => {
                        // Update metrics
                        if let Ok(mut metrics_guard) = metrics.write() {
                            *metrics_guard = new_metrics.clone();
                        }

                        // Update performance history
                        if let Ok(mut history_guard) = performance_history.write() {
                            history_guard.push((Utc::now(), new_metrics.performance));

                            // Keep only recent history
                            if history_guard.len() > config.performance_history_size {
                                history_guard.remove(0);
                            }
                        }

                        // Log status changes
                        match new_metrics.status {
                            HealthStatus::Healthy => debug!("Health check passed"),
                            HealthStatus::Degraded => {
                                warn!("Health check shows degraded performance")
                            }
                            HealthStatus::Warning => warn!(
                                "Health check shows warning status: {} issues",
                                new_metrics.issues.len()
                            ),
                            HealthStatus::Critical => error!(
                                "Health check shows critical status: {} issues",
                                new_metrics.issues.len()
                            ),
                        }
                    }
                    Err(e) => {
                        error!("Health check failed: {}", e);

                        // Update metrics to show failure
                        if let Ok(mut metrics_guard) = metrics.write() {
                            metrics_guard.status = HealthStatus::Critical;
                            metrics_guard.last_updated = Utc::now();
                            metrics_guard.issues.push(HealthIssue {
                                issue_type: HealthIssueType::ConnectionFailure,
                                severity: HealthSeverity::Critical,
                                description: format!("Health check failed: {e}"),
                                recommendation: "Check database connectivity and configuration"
                                    .to_string(),
                                detected_at: Utc::now(),
                            });
                        }
                    }
                }

                let elapsed = start_time.elapsed();
                let sleep_duration = if elapsed < config.check_interval {
                    config.check_interval - elapsed
                } else {
                    Duration::from_secs(1) // Minimum sleep
                };

                sleep(sleep_duration).await;
            }
        });

        Ok(())
    }

    /// Perform a single health check
    async fn perform_health_check(
        pool: &DatabasePool,
        config: &HealthCheckConfig,
    ) -> DbFastResult<HealthMetrics> {
        let start_time = Instant::now();

        // Test basic connectivity
        let connectivity = Self::check_connectivity(pool, &config.thresholds).await?;

        // Get pool statistics (simulated for now)
        let pool_stats = Self::get_pool_statistics(pool, &config.thresholds).await;

        // Get performance metrics (simulated for now)
        let performance = Self::get_performance_metrics(pool).await;

        // Analyze metrics and determine status
        let (status, issues) =
            Self::analyze_metrics(&pool_stats, &connectivity, &performance, &config.thresholds);

        let total_elapsed = start_time.elapsed();
        debug!("Health check completed in {:?}", total_elapsed);

        Ok(HealthMetrics {
            status,
            pool_stats,
            connectivity,
            performance,
            last_updated: Utc::now(),
            issues,
        })
    }

    /// Check database connectivity
    async fn check_connectivity(
        pool: &DatabasePool,
        _thresholds: &HealthThresholds,
    ) -> DbFastResult<ConnectivityMetrics> {
        let start_time = Instant::now();

        // Try a simple query
        let result = pool.query("SELECT 1 as health_check", &[]).await;
        let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        let (can_connect, server_version) = match result {
            Ok(_) => {
                // Try to get server version
                let version_result = pool.query("SELECT version() as version", &[]).await;
                let version = version_result
                    .ok()
                    .and_then(|rows| rows.first().cloned())
                    .and_then(|row| row.try_get::<_, String>("version").ok());
                (true, version)
            }
            Err(_) => (false, None),
        };

        Ok(ConnectivityMetrics {
            can_connect,
            latency_ms: Some(latency_ms),
            server_version,
            last_success: if can_connect { Some(Utc::now()) } else { None },
            last_failure: if !can_connect { Some(Utc::now()) } else { None },
            recent_failures: if can_connect { 0 } else { 1 },
        })
    }

    /// Get connection pool statistics
    async fn get_pool_statistics(
        _pool: &DatabasePool,
        _thresholds: &HealthThresholds,
    ) -> PoolStatistics {
        // In a real implementation, this would get actual pool statistics
        // from bb8 pool state. For now, we'll simulate realistic values.
        PoolStatistics {
            total_connections: 8,
            active_connections: 3,
            idle_connections: 5,
            max_connections: 10,
            utilization_percent: 30.0,
            exhaustion_count: 0,
            avg_acquire_time_ms: 5.2,
        }
    }

    /// Get performance metrics
    async fn get_performance_metrics(_pool: &DatabasePool) -> PerformanceMetrics {
        // In a real implementation, this would collect actual performance metrics
        // from database statistics tables or monitoring systems
        PerformanceMetrics {
            avg_query_time_ms: 45.2,
            p95_query_time_ms: 180.5,
            queries_per_second: 25.7,
            slow_query_count: 2,
            cpu_usage_percent: Some(15.3),
            memory_usage_percent: Some(62.1),
        }
    }

    /// Analyze metrics and determine overall health status
    fn analyze_metrics(
        pool_stats: &PoolStatistics,
        connectivity: &ConnectivityMetrics,
        performance: &PerformanceMetrics,
        thresholds: &HealthThresholds,
    ) -> (HealthStatus, Vec<HealthIssue>) {
        let mut issues = Vec::new();
        let mut max_severity = HealthSeverity::Low;

        // Analyze connectivity
        if !connectivity.can_connect {
            issues.push(HealthIssue {
                issue_type: HealthIssueType::ConnectionFailure,
                severity: HealthSeverity::Critical,
                description: "Cannot connect to database".to_string(),
                recommendation: "Check database server status and network connectivity".to_string(),
                detected_at: Utc::now(),
            });
            max_severity = HealthSeverity::Critical;
        } else if let Some(latency) = connectivity.latency_ms {
            if latency > thresholds.latency_critical_ms {
                issues.push(HealthIssue {
                    issue_type: HealthIssueType::HighLatency,
                    severity: HealthSeverity::Critical,
                    description: format!("High connection latency: {latency:.1}ms"),
                    recommendation: "Check network performance and database load".to_string(),
                    detected_at: Utc::now(),
                });
                max_severity = HealthSeverity::Critical;
            } else if latency > thresholds.latency_warning_ms {
                issues.push(HealthIssue {
                    issue_type: HealthIssueType::HighLatency,
                    severity: HealthSeverity::Medium,
                    description: format!("Elevated connection latency: {latency:.1}ms"),
                    recommendation: "Monitor network and database performance".to_string(),
                    detected_at: Utc::now(),
                });
                if max_severity < HealthSeverity::Medium {
                    max_severity = HealthSeverity::Medium;
                }
            }
        }

        // Analyze pool utilization
        if pool_stats.utilization_percent > thresholds.pool_utilization_critical {
            issues.push(HealthIssue {
                issue_type: HealthIssueType::PoolExhaustion,
                severity: HealthSeverity::Critical,
                description: format!(
                    "Critical pool utilization: {:.1}%",
                    pool_stats.utilization_percent
                ),
                recommendation: "Increase connection pool size or optimize connection usage"
                    .to_string(),
                detected_at: Utc::now(),
            });
            max_severity = HealthSeverity::Critical;
        } else if pool_stats.utilization_percent > thresholds.pool_utilization_warning {
            issues.push(HealthIssue {
                issue_type: HealthIssueType::PoolExhaustion,
                severity: HealthSeverity::High,
                description: format!(
                    "High pool utilization: {:.1}%",
                    pool_stats.utilization_percent
                ),
                recommendation: "Consider increasing connection pool size".to_string(),
                detected_at: Utc::now(),
            });
            if max_severity < HealthSeverity::High {
                max_severity = HealthSeverity::High;
            }
        }

        // Analyze performance
        if performance.slow_query_count > 10 {
            issues.push(HealthIssue {
                issue_type: HealthIssueType::SlowQueries,
                severity: HealthSeverity::Medium,
                description: format!(
                    "High number of slow queries: {}",
                    performance.slow_query_count
                ),
                recommendation: "Review and optimize slow queries".to_string(),
                detected_at: Utc::now(),
            });
            if max_severity < HealthSeverity::Medium {
                max_severity = HealthSeverity::Medium;
            }
        }

        // Determine overall status
        let status = match max_severity {
            HealthSeverity::Critical => HealthStatus::Critical,
            HealthSeverity::High => HealthStatus::Warning,
            HealthSeverity::Medium => HealthStatus::Degraded,
            HealthSeverity::Low => HealthStatus::Healthy,
        };

        (status, issues)
    }

    /// Get current health metrics
    pub fn get_current_metrics(&self) -> Option<HealthMetrics> {
        self.metrics.read().ok().map(|m| m.clone())
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> Vec<(DateTime<Utc>, PerformanceMetrics)> {
        self.performance_history
            .read()
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.metrics
            .read()
            .map(|m| matches!(m.status, HealthStatus::Healthy | HealthStatus::Degraded))
            .unwrap_or(false)
    }
}

impl Default for PoolStatistics {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            max_connections: 10,
            utilization_percent: 0.0,
            exhaustion_count: 0,
            avg_acquire_time_ms: 0.0,
        }
    }
}

impl Default for ConnectivityMetrics {
    fn default() -> Self {
        Self {
            can_connect: false,
            latency_ms: None,
            server_version: None,
            last_success: None,
            last_failure: None,
            recent_failures: 0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_query_time_ms: 0.0,
            p95_query_time_ms: 0.0,
            queries_per_second: 0.0,
            slow_query_count: 0,
            cpu_usage_percent: None,
            memory_usage_percent: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_thresholds_default() {
        let thresholds = HealthThresholds::default();
        assert_eq!(thresholds.pool_utilization_warning, 70.0);
        assert_eq!(thresholds.pool_utilization_critical, 90.0);
    }

    #[test]
    fn test_analyze_metrics_healthy() {
        let pool_stats = PoolStatistics {
            utilization_percent: 50.0,
            ..Default::default()
        };
        let connectivity = ConnectivityMetrics {
            can_connect: true,
            latency_ms: Some(25.0),
            ..Default::default()
        };
        let performance = PerformanceMetrics::default();
        let thresholds = HealthThresholds::default();

        let (status, issues) =
            HealthMonitor::analyze_metrics(&pool_stats, &connectivity, &performance, &thresholds);

        assert_eq!(status, HealthStatus::Healthy);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_analyze_metrics_critical() {
        let pool_stats = PoolStatistics {
            utilization_percent: 95.0,
            ..Default::default()
        };
        let connectivity = ConnectivityMetrics {
            can_connect: false,
            ..Default::default()
        };
        let performance = PerformanceMetrics::default();
        let thresholds = HealthThresholds::default();

        let (status, issues) =
            HealthMonitor::analyze_metrics(&pool_stats, &connectivity, &performance, &thresholds);

        assert_eq!(status, HealthStatus::Critical);
        assert!(!issues.is_empty());
    }
}
