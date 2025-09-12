//! Comprehensive observability and monitoring infrastructure
//!
//! This module provides enterprise-grade observability features including:
//! - Structured logging with correlation IDs
//! - Distributed tracing
//! - Metrics collection and export
//! - Performance monitoring
//! - Audit logging

use crate::errors::DbFastError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Central observability manager
#[derive(Clone)]
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    audit_logger: Arc<AuditLogger>,
    metrics_exporter: Arc<MetricsExporter>,
    trace_context: Arc<RwLock<TraceContext>>,
}

/// Feature flags for observability capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Feature flags are expected to have many bools
pub struct ObservabilityFeatures {
    /// Enable distributed tracing
    pub tracing: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Enable metrics export
    pub metrics_export: bool,
    /// Enable correlation ID tracking
    pub correlation_ids: bool,
    /// Enable security event logging
    pub security_logging: bool,
}

impl Default for ObservabilityFeatures {
    fn default() -> Self {
        Self {
            tracing: true,
            audit_logging: true,
            metrics_export: true,
            correlation_ids: true,
            security_logging: true,
        }
    }
}

/// Configuration for observability features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Feature flags
    pub features: ObservabilityFeatures,

    /// Metrics export interval in seconds
    pub metrics_export_interval: u64,

    /// Log level for structured logging
    pub log_level: String,

    /// Maximum trace context size
    pub max_trace_context_size: usize,
}

/// Distributed tracing context
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Current trace ID
    pub trace_id: Option<String>,

    /// Current span ID
    pub span_id: Option<String>,

    /// Parent span ID
    pub parent_span_id: Option<String>,

    /// Trace attributes
    pub attributes: HashMap<String, String>,

    /// Baggage items (cross-service context)
    pub baggage: HashMap<String, String>,
}

/// Audit logger for security and compliance
#[derive(Debug)]
pub struct AuditLogger {
    #[allow(dead_code)] // Configuration stored for future use
    config: ObservabilityConfig,
    audit_entries: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,

    /// Event type (authentication, authorization, `data_access`, etc.)
    pub event_type: AuditEventType,

    /// User or system identifier
    pub actor: String,

    /// Resource being accessed
    pub resource: Option<String>,

    /// Action performed
    pub action: String,

    /// Result of the action
    pub result: AuditResult,

    /// Additional context
    pub context: HashMap<String, String>,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Authentication events
    Authentication,

    /// Authorization/permission checks
    Authorization,

    /// Data access events
    DataAccess,

    /// Configuration changes
    ConfigurationChange,

    /// Security events
    SecurityEvent,

    /// System administration
    SystemAdministration,

    /// Deployment events
    Deployment,

    /// Error/exception events
    ErrorEvent,
}

/// Result of audited action
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action succeeded
    Success,

    /// Action failed
    Failure,

    /// Action was denied
    Denied,

    /// Action requires additional authorization
    PendingAuthorization,
}

/// Risk level for audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Low risk events
    Low,

    /// Medium risk events
    Medium,

    /// High risk events
    High,

    /// Critical security events
    Critical,
}

/// Metrics exporter for external monitoring systems
#[derive(Debug)]
pub struct MetricsExporter {
    #[allow(dead_code)] // Configuration stored for future use
    config: ObservabilityConfig,
    exported_metrics: Arc<RwLock<Vec<ExportedMetric>>>,
}

/// Exported metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMetric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Metric type
    pub metric_type: MetricType,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Tags/labels
    pub tags: HashMap<String, String>,

    /// Unit of measurement
    pub unit: Option<String>,
}

/// Types of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (monotonic)
    Counter,

    /// Gauge metric (current value)
    Gauge,

    /// Histogram metric
    Histogram,

    /// Timing metric
    Timing,
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    /// Log timestamp
    pub timestamp: DateTime<Utc>,

    /// Log level
    pub level: String,

    /// Log message
    pub message: String,

    /// Source component
    pub component: String,

    /// Correlation ID
    pub correlation_id: Option<String>,

    /// Trace ID
    pub trace_id: Option<String>,

    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,

    /// Error information if applicable
    pub error: Option<ErrorInfo>,
}

/// Error information for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Error severity
    pub severity: String,

    /// Stack trace if available
    pub stack_trace: Option<String>,

    /// Error context
    pub context: HashMap<String, String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            features: ObservabilityFeatures::default(),
            metrics_export_interval: 60,
            log_level: "info".to_string(),
            max_trace_context_size: 100,
        }
    }
}

impl ObservabilityManager {
    /// Create a new observability manager
    #[must_use]
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            audit_logger: Arc::new(AuditLogger::new(config.clone())),
            metrics_exporter: Arc::new(MetricsExporter::new(config.clone())),
            trace_context: Arc::new(RwLock::new(TraceContext::new())),
            config,
        }
    }

    /// Start observability services
    pub async fn start(&self) -> Result<(), DbFastError> {
        info!("Starting observability services");

        if self.config.features.tracing {
            self.start_tracing().await?;
        }

        if self.config.features.metrics_export {
            self.start_metrics_export().await?;
        }

        if self.config.features.audit_logging {
            self.start_audit_logging().await?;
        }

        info!("Observability services started successfully");
        Ok(())
    }

    /// Start distributed tracing
    async fn start_tracing(&self) -> Result<(), DbFastError> {
        debug!("Initializing distributed tracing");

        // Initialize trace context
        self.trace_context.write().await.trace_id = Some(Uuid::new_v4().to_string());

        Ok(())
    }

    /// Start metrics export
    async fn start_metrics_export(&self) -> Result<(), DbFastError> {
        debug!("Starting metrics export service");

        let exporter = self.metrics_exporter.clone();
        let interval = self.config.metrics_export_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval.tick().await;
                if let Err(e) = exporter.export_metrics().await {
                    error!("Failed to export metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start audit logging
    async fn start_audit_logging(&self) -> Result<(), DbFastError> {
        debug!("Starting audit logging service");
        Ok(())
    }

    /// Create a new trace span
    pub async fn create_span(&self, name: &str, attributes: HashMap<String, String>) -> TraceSpan {
        let span_id = Uuid::new_v4().to_string();
        let (parent_span_id, ()) = {
            let mut context = self.trace_context.write().await;
            let parent = context.span_id.clone();
            context.span_id = Some(span_id.clone());
            context.attributes.extend(attributes.clone());
            drop(context);
            (parent, ())
        };

        TraceSpan {
            span_id,
            name: name.to_string(),
            start_time: Utc::now(),
            attributes,
            parent_span_id,
        }
    }

    /// Log an audit event
    pub async fn audit(&self, event: AuditEntry) -> Result<(), DbFastError> {
        self.audit_logger.log_event(event).await
    }

    /// Export a metric
    pub async fn export_metric(&self, metric: ExportedMetric) -> Result<(), DbFastError> {
        self.metrics_exporter.add_metric(metric).await
    }

    /// Log a structured message
    pub fn log_structured(entry: StructuredLogEntry) {
        let json = serde_json::to_string(&entry).unwrap_or_default();

        match entry.level.as_str() {
            "error" => error!("{}", json),
            "warn" => warn!("{}", json),
            "info" => info!("{}", json),
            "debug" => debug!("{}", json),
            _ => info!("{}", json),
        }
    }

    /// Get current correlation ID
    pub async fn correlation_id(&self) -> Option<String> {
        let context = self.trace_context.read().await;
        context.trace_id.clone()
    }
}

/// Trace span for distributed tracing
#[derive(Debug, Clone)]
pub struct TraceSpan {
    /// Span ID
    pub span_id: String,

    /// Span name
    pub name: String,

    /// Start time
    pub start_time: DateTime<Utc>,

    /// Span attributes
    pub attributes: HashMap<String, String>,

    /// Parent span ID
    pub parent_span_id: Option<String>,
}

impl TraceSpan {
    /// Finish the span
    #[must_use]
    pub fn finish(self) -> FinishedSpan {
        FinishedSpan {
            span_id: self.span_id,
            name: self.name,
            start_time: self.start_time,
            end_time: Utc::now(),
            attributes: self.attributes,
            parent_span_id: self.parent_span_id,
        }
    }

    /// Add attribute to span
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
}

/// Finished trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishedSpan {
    /// Span ID
    pub span_id: String,

    /// Span name
    pub name: String,

    /// Start time
    pub start_time: DateTime<Utc>,

    /// End time
    pub end_time: DateTime<Utc>,

    /// Span attributes
    pub attributes: HashMap<String, String>,

    /// Parent span ID
    pub parent_span_id: Option<String>,
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceContext {
    /// Create new trace context
    #[must_use]
    pub fn new() -> Self {
        Self {
            trace_id: None,
            span_id: None,
            parent_span_id: None,
            attributes: HashMap::new(),
            baggage: HashMap::new(),
        }
    }
}

impl AuditLogger {
    /// Create new audit logger
    #[must_use]
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config,
            audit_entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEntry) -> Result<(), DbFastError> {
        // Add to in-memory store
        self.audit_entries.write().await.push(event.clone());

        // Log to structured logging
        let log_entry = StructuredLogEntry {
            timestamp: event.timestamp,
            level: match event.risk_level {
                RiskLevel::Critical => "error",
                RiskLevel::High => "warn",
                RiskLevel::Medium => "info",
                RiskLevel::Low => "debug",
            }
            .to_string(),
            message: format!(
                "AUDIT: {} {} {}",
                event.actor,
                event.action,
                event.resource.as_deref().unwrap_or("system")
            ),
            component: "audit".to_string(),
            correlation_id: event.correlation_id,
            trace_id: None,
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    "event_type".to_string(),
                    serde_json::to_value(&event.event_type).unwrap(),
                );
                fields.insert(
                    "result".to_string(),
                    serde_json::to_value(event.result).unwrap(),
                );
                fields.insert(
                    "risk_level".to_string(),
                    serde_json::to_value(event.risk_level).unwrap(),
                );
                for (k, v) in event.context {
                    fields.insert(k, serde_json::Value::String(v));
                }
                fields
            },
            error: None,
        };

        // In a real implementation, this would be sent to external audit systems
        debug!(
            "Audit event logged: {}",
            serde_json::to_string(&log_entry).unwrap()
        );

        Ok(())
    }

    /// Get audit events by criteria
    pub async fn get_events_by_actor(&self, actor: &str) -> Vec<AuditEntry> {
        let entries = self.audit_entries.read().await;
        entries
            .iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Get high-risk audit events
    pub async fn get_high_risk_events(&self) -> Vec<AuditEntry> {
        let entries = self.audit_entries.read().await;
        entries
            .iter()
            .filter(|e| e.risk_level >= RiskLevel::High)
            .cloned()
            .collect()
    }
}

impl MetricsExporter {
    /// Create new metrics exporter
    #[must_use]
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config,
            exported_metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add metric for export
    pub async fn add_metric(&self, metric: ExportedMetric) -> Result<(), DbFastError> {
        self.exported_metrics.write().await.push(metric);
        Ok(())
    }

    /// Export metrics to external systems
    pub async fn export_metrics(&self) -> Result<(), DbFastError> {
        let metrics = {
            let mut metrics_guard = self.exported_metrics.write().await;
            let metrics = metrics_guard.clone();
            metrics_guard.clear();
            metrics
        };

        if !metrics.is_empty() {
            debug!("Exporting {} metrics", metrics.len());

            // In a real implementation, this would send metrics to external systems
            // like Prometheus, DataDog, CloudWatch, etc.
            for metric in &metrics {
                debug!(
                    "Metric: {} = {} ({})",
                    metric.name,
                    metric.value,
                    serde_json::to_string(&metric.metric_type).unwrap()
                );
            }
        }

        Ok(())
    }

    /// Get metrics summary
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let metrics = self.exported_metrics.read().await;

        let mut counter_count = 0;
        let mut gauge_count = 0;
        let mut histogram_count = 0;
        let mut timing_count = 0;

        for metric in metrics.iter() {
            match metric.metric_type {
                MetricType::Counter => counter_count += 1,
                MetricType::Gauge => gauge_count += 1,
                MetricType::Histogram => histogram_count += 1,
                MetricType::Timing => timing_count += 1,
            }
        }

        MetricsSummary {
            total_metrics: metrics.len(),
            counter_metrics: counter_count,
            gauge_metrics: gauge_count,
            histogram_metrics: histogram_count,
            timing_metrics: timing_count,
            last_export: Utc::now(),
        }
    }
}

/// Metrics export summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total number of metrics
    pub total_metrics: usize,

    /// Number of counter metrics
    pub counter_metrics: usize,

    /// Number of gauge metrics
    pub gauge_metrics: usize,

    /// Number of histogram metrics
    pub histogram_metrics: usize,

    /// Number of timing metrics
    pub timing_metrics: usize,

    /// Last export timestamp
    pub last_export: DateTime<Utc>,
}

/// Create an audit entry for authentication events
#[must_use]
pub fn create_auth_audit_entry(
    actor: &str,
    action: &str,
    result: AuditResult,
    correlation_id: Option<String>,
) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        event_type: AuditEventType::Authentication,
        actor: actor.to_string(),
        resource: None,
        action: action.to_string(),
        result,
        context: HashMap::new(),
        risk_level: match result {
            AuditResult::Success => RiskLevel::Low,
            AuditResult::Failure | AuditResult::PendingAuthorization => RiskLevel::Medium,
            AuditResult::Denied => RiskLevel::High,
        },
        correlation_id,
    }
}

/// Create an audit entry for data access events
#[must_use]
pub fn create_data_access_audit_entry(
    actor: &str,
    resource: &str,
    action: &str,
    result: AuditResult,
    sensitive: bool,
    correlation_id: Option<String>,
) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        event_type: AuditEventType::DataAccess,
        actor: actor.to_string(),
        resource: Some(resource.to_string()),
        action: action.to_string(),
        result,
        context: {
            let mut context = HashMap::new();
            context.insert("sensitive_data".to_string(), sensitive.to_string());
            context
        },
        risk_level: if sensitive {
            match result {
                AuditResult::Success => RiskLevel::Medium,
                AuditResult::Failure | AuditResult::PendingAuthorization => RiskLevel::High,
                AuditResult::Denied => RiskLevel::Critical,
            }
        } else {
            match result {
                AuditResult::Success | AuditResult::PendingAuthorization => RiskLevel::Low,
                AuditResult::Failure | AuditResult::Denied => RiskLevel::Medium,
            }
        },
        correlation_id,
    }
}

/// Create a security audit entry
#[allow(clippy::implicit_hasher)]
#[must_use]
pub fn create_security_audit_entry(
    actor: &str,
    action: &str,
    threat_level: RiskLevel,
    details: HashMap<String, String>,
    correlation_id: Option<String>,
) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        event_type: AuditEventType::SecurityEvent,
        actor: actor.to_string(),
        resource: None,
        action: action.to_string(),
        result: AuditResult::Denied, // Security events are typically denials
        context: details,
        risk_level: threat_level,
        correlation_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_manager_creation() {
        let config = ObservabilityConfig::default();
        let manager = ObservabilityManager::new(config);

        assert!(manager.correlation_id().await.is_none()); // No active trace initially
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let config = ObservabilityConfig::default();
        let logger = AuditLogger::new(config);

        let audit_entry = create_auth_audit_entry(
            "user123",
            "login",
            AuditResult::Success,
            Some("trace-123".to_string()),
        );

        assert!(logger.log_event(audit_entry).await.is_ok());

        let events = logger.get_events_by_actor("user123").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].actor, "user123");
        assert_eq!(events[0].action, "login");
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let config = ObservabilityConfig::default();
        let exporter = MetricsExporter::new(config);

        let metric = ExportedMetric {
            name: "test_counter".to_string(),
            value: 42.0,
            metric_type: MetricType::Counter,
            timestamp: Utc::now(),
            tags: HashMap::new(),
            unit: Some("count".to_string()),
        };

        assert!(exporter.add_metric(metric).await.is_ok());

        let summary = exporter.get_metrics_summary().await;
        assert_eq!(summary.total_metrics, 1);
        assert_eq!(summary.counter_metrics, 1);
    }

    #[tokio::test]
    async fn test_trace_span() {
        let config = ObservabilityConfig::default();
        let manager = ObservabilityManager::new(config);

        let attributes = {
            let mut attrs = HashMap::new();
            attrs.insert("operation".to_string(), "test".to_string());
            attrs
        };

        let mut span = manager.create_span("test_span", attributes).await;
        span.add_attribute("additional", "value");

        let finished_span = span.finish();
        assert_eq!(finished_span.name, "test_span");
        assert!(finished_span.attributes.contains_key("operation"));
        assert!(finished_span.attributes.contains_key("additional"));
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = create_security_audit_entry(
            "potential_attacker",
            "attempted_sql_injection",
            RiskLevel::Critical,
            {
                let mut details = HashMap::new();
                details.insert("ip_address".to_string(), "192.168.1.100".to_string());
                details.insert("user_agent".to_string(), "malicious_bot".to_string());
                details
            },
            Some("security-alert-456".to_string()),
        );

        assert_eq!(entry.actor, "potential_attacker");
        assert_eq!(entry.risk_level, RiskLevel::Critical);
        assert_eq!(entry.event_type, AuditEventType::SecurityEvent);
        assert!(entry.context.contains_key("ip_address"));
    }
}
