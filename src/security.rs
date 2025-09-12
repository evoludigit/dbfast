//! Enterprise security hardening and audit logging
//!
//! This module provides comprehensive security features including:
//! - Input validation and sanitization
//! - SQL injection prevention
//! - Authentication and authorization
//! - Rate limiting and DoS protection
//! - Encryption and data protection
//! - Security event monitoring

use crate::errors::{DbFastError, ErrorContext, ErrorSeverity};
use crate::observability::{
    create_security_audit_entry, AuditResult, ObservabilityManager, RiskLevel,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Security manager for enterprise security features
#[derive(Clone)]
pub struct SecurityManager {
    config: SecurityConfig,
    rate_limiter: Arc<RateLimiter>,
    auth_manager: Arc<AuthenticationManager>,
    input_validator: Arc<InputValidator>,
    encryption_manager: Arc<EncryptionManager>,
    observability: Option<Arc<ObservabilityManager>>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,

    /// Maximum requests per minute per client
    pub max_requests_per_minute: u32,

    /// Enable input validation
    pub enable_input_validation: bool,

    /// Maximum input length
    pub max_input_length: usize,

    /// Enable SQL injection detection
    pub enable_sql_injection_detection: bool,

    /// Enable authentication
    pub enable_authentication: bool,

    /// Session timeout in minutes
    pub session_timeout_minutes: i64,

    /// Enable encryption for sensitive data
    pub enable_encryption: bool,

    /// Encryption key rotation interval in days
    pub key_rotation_days: u32,

    /// Enable security event logging
    pub enable_security_logging: bool,

    /// Failed login attempt threshold before lockout
    pub max_failed_login_attempts: u32,

    /// Account lockout duration in minutes
    pub account_lockout_minutes: i64,
}

/// Rate limiter to prevent DoS attacks
#[derive(Debug)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, ClientRateLimit>>>,
    config: SecurityConfig,
}

/// Per-client rate limiting information
#[derive(Debug, Clone)]
struct ClientRateLimit {
    /// Number of requests in current window
    request_count: u32,

    /// Start of current time window
    window_start: DateTime<Utc>,

    /// Whether client is currently blocked
    is_blocked: bool,

    /// Block expiration time
    block_expires: Option<DateTime<Utc>>,
}

/// Authentication manager
#[derive(Debug)]
pub struct AuthenticationManager {
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttemptTracker>>>,
    config: SecurityConfig,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub session_id: String,

    /// User ID
    pub user_id: String,

    /// Session creation time
    pub created_at: DateTime<Utc>,

    /// Last activity time
    pub last_activity: DateTime<Utc>,

    /// Session permissions
    pub permissions: Vec<String>,

    /// IP address of the session
    pub ip_address: String,

    /// User agent string
    pub user_agent: Option<String>,
}

/// Failed login attempt tracking
#[derive(Debug, Clone)]
struct FailedAttemptTracker {
    /// Number of failed attempts
    attempt_count: u32,

    /// Time of first failed attempt in current window
    first_attempt: DateTime<Utc>,

    /// Time when account is locked until
    locked_until: Option<DateTime<Utc>>,
}

/// Input validator for SQL injection and other attacks
#[derive(Debug)]
pub struct InputValidator {
    config: SecurityConfig,
    sql_injection_patterns: Vec<regex::Regex>,
}

/// Encryption manager for data protection
#[derive(Debug)]
pub struct EncryptionManager {
    current_key: [u8; 32],
    key_rotation_time: DateTime<Utc>,
    config: SecurityConfig,
}

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityValidationResult {
    /// Whether input is valid
    pub is_valid: bool,

    /// Security threats detected
    pub threats: Vec<SecurityThreat>,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Recommended action
    pub recommended_action: SecurityAction,
}

/// Security threat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityThreat {
    /// Type of threat
    pub threat_type: ThreatType,

    /// Description of the threat
    pub description: String,

    /// Severity level
    pub severity: ThreatSeverity,

    /// Evidence/details
    pub evidence: HashMap<String, String>,
}

/// Types of security threats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    /// SQL injection attempt
    SqlInjection,

    /// Cross-site scripting attempt
    XssAttempt,

    /// Path traversal attempt
    PathTraversal,

    /// Command injection attempt
    CommandInjection,

    /// Brute force attack
    BruteForce,

    /// Rate limit exceeded
    RateLimitExceeded,

    /// Suspicious input pattern
    SuspiciousInput,

    /// Authentication bypass attempt
    AuthBypass,
}

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatSeverity {
    /// Low severity threat
    Low,

    /// Medium severity threat
    Medium,

    /// High severity threat
    High,

    /// Critical severity threat
    Critical,
}

/// Recommended security actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    /// Allow the request
    Allow,

    /// Block the request
    Block,

    /// Log and monitor
    LogAndMonitor,

    /// Require additional authentication
    RequireReauth,

    /// Temporary rate limit
    RateLimit,

    /// Permanent ban
    PermanentBan,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            max_requests_per_minute: 100,
            enable_input_validation: true,
            max_input_length: 10_000,
            enable_sql_injection_detection: true,
            enable_authentication: true,
            session_timeout_minutes: 60,
            enable_encryption: true,
            key_rotation_days: 30,
            enable_security_logging: true,
            max_failed_login_attempts: 5,
            account_lockout_minutes: 15,
        }
    }
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            rate_limiter: Arc::new(RateLimiter::new(config.clone())),
            auth_manager: Arc::new(AuthenticationManager::new(config.clone())),
            input_validator: Arc::new(InputValidator::new(config.clone())),
            encryption_manager: Arc::new(EncryptionManager::new(config.clone())),
            observability: None,
            config,
        }
    }

    /// Set observability manager for security event logging
    pub fn with_observability(mut self, observability: Arc<ObservabilityManager>) -> Self {
        self.observability = Some(observability);
        self
    }

    /// Validate a request for security threats
    pub async fn validate_request(
        &self,
        client_id: &str,
        input: &str,
        context: SecurityContext,
    ) -> SecurityValidationResult {
        let mut threats = Vec::new();
        let mut max_risk = RiskLevel::Low;

        // Rate limiting check
        if self.config.enable_rate_limiting {
            if let Some(rate_threat) = self.rate_limiter.check_rate_limit(client_id).await {
                threats.push(rate_threat.clone());
                if rate_threat.severity >= ThreatSeverity::High {
                    max_risk = RiskLevel::High;
                }
            }
        }

        // Input validation
        if self.config.enable_input_validation {
            if let Some(input_threats) = self.input_validator.validate_input(input).await {
                for threat in input_threats {
                    if threat.severity >= ThreatSeverity::High {
                        max_risk = RiskLevel::High;
                    } else if threat.severity >= ThreatSeverity::Medium
                        && max_risk < RiskLevel::Medium
                    {
                        max_risk = RiskLevel::Medium;
                    }
                    threats.push(threat);
                }
            }
        }

        let is_valid =
            threats.is_empty() || threats.iter().all(|t| t.severity < ThreatSeverity::High);

        let recommended_action = if !is_valid {
            match max_risk {
                RiskLevel::Critical => SecurityAction::PermanentBan,
                RiskLevel::High => SecurityAction::Block,
                RiskLevel::Medium => SecurityAction::LogAndMonitor,
                RiskLevel::Low => SecurityAction::Allow,
            }
        } else {
            SecurityAction::Allow
        };

        // Log security events
        if let Some(observability) = &self.observability {
            if !threats.is_empty() {
                let audit_entry = create_security_audit_entry(
                    client_id,
                    &format!("request_validation_{:?}", context.operation_type),
                    max_risk,
                    {
                        let mut details = HashMap::new();
                        details.insert("threat_count".to_string(), threats.len().to_string());
                        details.insert("input_length".to_string(), input.len().to_string());
                        details.insert(
                            "client_ip".to_string(),
                            context.client_ip.unwrap_or_default(),
                        );
                        details
                    },
                    context.correlation_id,
                );

                let _ = observability.audit(audit_entry).await;
            }
        }

        SecurityValidationResult {
            is_valid,
            threats,
            risk_level: max_risk,
            recommended_action,
        }
    }

    /// Authenticate a user
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
        client_info: ClientInfo,
    ) -> Result<UserSession, DbFastError> {
        // Check for account lockout
        if self.auth_manager.is_account_locked(username).await {
            let mut details = HashMap::new();
            details.insert("username".to_string(), username.to_string());
            details.insert("reason".to_string(), "account_locked".to_string());

            if let Some(observability) = &self.observability {
                let audit_entry = create_security_audit_entry(
                    username,
                    "login_attempt_while_locked",
                    RiskLevel::High,
                    details,
                    client_info.correlation_id.clone(),
                );
                let _ = observability.audit(audit_entry).await;
            }

            return Err(DbFastError::Auth {
                source: crate::errors::AuthenticationError::AccessDenied {
                    operation: "login".to_string(),
                },
                context: ErrorContext::new("authentication", "security")
                    .with_severity(ErrorSeverity::High)
                    .with_detail("reason", "account_locked"),
            });
        }

        // Simulate password verification (in real implementation, use proper password hashing)
        let password_valid = self.verify_password(username, password).await;

        if password_valid {
            // Clear failed attempts on successful login
            self.auth_manager.clear_failed_attempts(username).await;

            // Create session
            let session = self
                .auth_manager
                .create_session(username, client_info)
                .await?;

            if let Some(observability) = &self.observability {
                let audit_entry = crate::observability::create_auth_audit_entry(
                    username,
                    "login_success",
                    AuditResult::Success,
                    session.session_id.clone().into(),
                );
                let _ = observability.audit(audit_entry).await;
            }

            Ok(session)
        } else {
            // Record failed attempt
            self.auth_manager.record_failed_attempt(username).await;

            if let Some(observability) = &self.observability {
                let audit_entry = crate::observability::create_auth_audit_entry(
                    username,
                    "login_failure",
                    AuditResult::Failure,
                    client_info.correlation_id,
                );
                let _ = observability.audit(audit_entry).await;
            }

            Err(DbFastError::Auth {
                source: crate::errors::AuthenticationError::InvalidCredentials,
                context: ErrorContext::new("authentication", "security")
                    .with_severity(ErrorSeverity::Medium)
                    .with_detail("username", username),
            })
        }
    }

    /// Encrypt sensitive data
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, DbFastError> {
        self.encryption_manager.encrypt(data)
    }

    /// Decrypt sensitive data
    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, DbFastError> {
        self.encryption_manager.decrypt(encrypted_data)
    }

    /// Validate a session
    pub async fn validate_session(&self, session_id: &str) -> Result<UserSession, DbFastError> {
        self.auth_manager.validate_session(session_id).await
    }

    /// Simple password verification (in production, use proper password hashing)
    async fn verify_password(&self, _username: &str, _password: &str) -> bool {
        // This is a placeholder - in production use bcrypt, scrypt, or Argon2
        // For testing purposes, accept any non-empty password
        !_password.is_empty()
    }
}

/// Security context for request validation
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Type of operation being performed
    pub operation_type: OperationType,

    /// Client IP address
    pub client_ip: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Correlation ID for tracing
    pub correlation_id: Option<String>,

    /// Additional context data
    pub metadata: HashMap<String, String>,
}

/// Types of operations for security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Database query
    DatabaseQuery,

    /// Configuration change
    ConfigurationChange,

    /// User authentication
    Authentication,

    /// File upload
    FileUpload,

    /// API request
    ApiRequest,

    /// Administrative operation
    AdminOperation,
}

/// Client information for authentication
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client IP address
    pub ip_address: String,

    /// User agent string
    pub user_agent: Option<String>,

    /// Correlation ID
    pub correlation_id: Option<String>,
}

impl RateLimiter {
    fn new(config: SecurityConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn check_rate_limit(&self, client_id: &str) -> Option<SecurityThreat> {
        let mut requests = self.requests.write().await;
        let now = Utc::now();

        let client_limit =
            requests
                .entry(client_id.to_string())
                .or_insert_with(|| ClientRateLimit {
                    request_count: 0,
                    window_start: now,
                    is_blocked: false,
                    block_expires: None,
                });

        // Check if client is currently blocked
        if client_limit.is_blocked {
            if let Some(block_expires) = client_limit.block_expires {
                if now > block_expires {
                    // Unblock client
                    client_limit.is_blocked = false;
                    client_limit.block_expires = None;
                    client_limit.request_count = 1;
                    client_limit.window_start = now;
                } else {
                    // Still blocked
                    return Some(SecurityThreat {
                        threat_type: ThreatType::RateLimitExceeded,
                        description: "Client is currently rate limited".to_string(),
                        severity: ThreatSeverity::High,
                        evidence: {
                            let mut evidence = HashMap::new();
                            evidence.insert("client_id".to_string(), client_id.to_string());
                            evidence
                                .insert("block_expires".to_string(), block_expires.to_rfc3339());
                            evidence
                        },
                    });
                }
            }
        }

        // Reset window if necessary
        if now
            .signed_duration_since(client_limit.window_start)
            .num_minutes()
            >= 1
        {
            client_limit.request_count = 1;
            client_limit.window_start = now;
            return None;
        }

        client_limit.request_count += 1;

        // Check if rate limit exceeded
        if client_limit.request_count > self.config.max_requests_per_minute {
            client_limit.is_blocked = true;
            client_limit.block_expires = Some(now + ChronoDuration::minutes(5)); // 5 minute block

            Some(SecurityThreat {
                threat_type: ThreatType::RateLimitExceeded,
                description: format!(
                    "Client exceeded rate limit: {} requests per minute",
                    self.config.max_requests_per_minute
                ),
                severity: ThreatSeverity::High,
                evidence: {
                    let mut evidence = HashMap::new();
                    evidence.insert("client_id".to_string(), client_id.to_string());
                    evidence.insert(
                        "request_count".to_string(),
                        client_limit.request_count.to_string(),
                    );
                    evidence.insert(
                        "limit".to_string(),
                        self.config.max_requests_per_minute.to_string(),
                    );
                    evidence
                },
            })
        } else {
            None
        }
    }
}

impl InputValidator {
    fn new(config: SecurityConfig) -> Self {
        let sql_injection_patterns = vec![
            regex::Regex::new(r"(?i)(union\s+select|or\s+1\s*=\s*1|drop\s+table|delete\s+from|insert\s+into|update\s+.*\s+set)").unwrap(),
            regex::Regex::new(r"(?i)('|(\\')|(;)|(--)|(/\*)|(\*/)|(\bxp_\b)|(\bsp_\b))").unwrap(),
            regex::Regex::new(r"(?i)(exec|execute|eval|script|javascript|vbscript|onload|onerror)").unwrap(),
        ];

        Self {
            config,
            sql_injection_patterns,
        }
    }

    async fn validate_input(&self, input: &str) -> Option<Vec<SecurityThreat>> {
        let mut threats = Vec::new();

        // Check input length
        if input.len() > self.config.max_input_length {
            threats.push(SecurityThreat {
                threat_type: ThreatType::SuspiciousInput,
                description: format!(
                    "Input exceeds maximum length: {} > {}",
                    input.len(),
                    self.config.max_input_length
                ),
                severity: ThreatSeverity::Medium,
                evidence: {
                    let mut evidence = HashMap::new();
                    evidence.insert("input_length".to_string(), input.len().to_string());
                    evidence.insert(
                        "max_length".to_string(),
                        self.config.max_input_length.to_string(),
                    );
                    evidence
                },
            });
        }

        // Check for SQL injection patterns
        if self.config.enable_sql_injection_detection {
            for pattern in &self.sql_injection_patterns {
                if let Some(match_result) = pattern.find(input) {
                    threats.push(SecurityThreat {
                        threat_type: ThreatType::SqlInjection,
                        description: "Potential SQL injection detected".to_string(),
                        severity: ThreatSeverity::Critical,
                        evidence: {
                            let mut evidence = HashMap::new();
                            evidence.insert(
                                "matched_pattern".to_string(),
                                match_result.as_str().to_string(),
                            );
                            evidence.insert(
                                "match_position".to_string(),
                                match_result.start().to_string(),
                            );
                            evidence
                        },
                    });
                }
            }
        }

        // Check for path traversal
        if input.contains("../") || input.contains("..\\") {
            threats.push(SecurityThreat {
                threat_type: ThreatType::PathTraversal,
                description: "Potential path traversal detected".to_string(),
                severity: ThreatSeverity::High,
                evidence: {
                    let mut evidence = HashMap::new();
                    evidence.insert(
                        "input_sample".to_string(),
                        input.chars().take(100).collect::<String>(),
                    );
                    evidence
                },
            });
        }

        if threats.is_empty() {
            None
        } else {
            Some(threats)
        }
    }
}

impl AuthenticationManager {
    fn new(config: SecurityConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn is_account_locked(&self, username: &str) -> bool {
        let failed_attempts = self.failed_attempts.read().await;

        if let Some(tracker) = failed_attempts.get(username) {
            if let Some(locked_until) = tracker.locked_until {
                return Utc::now() < locked_until;
            }
        }

        false
    }

    async fn record_failed_attempt(&self, username: &str) {
        let mut failed_attempts = self.failed_attempts.write().await;
        let now = Utc::now();

        let tracker = failed_attempts
            .entry(username.to_string())
            .or_insert_with(|| FailedAttemptTracker {
                attempt_count: 0,
                first_attempt: now,
                locked_until: None,
            });

        tracker.attempt_count += 1;

        if tracker.attempt_count >= self.config.max_failed_login_attempts {
            tracker.locked_until =
                Some(now + ChronoDuration::minutes(self.config.account_lockout_minutes));
        }
    }

    async fn clear_failed_attempts(&self, username: &str) {
        let mut failed_attempts = self.failed_attempts.write().await;
        failed_attempts.remove(username);
    }

    async fn create_session(
        &self,
        username: &str,
        client_info: ClientInfo,
    ) -> Result<UserSession, DbFastError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = UserSession {
            session_id: session_id.clone(),
            user_id: username.to_string(),
            created_at: now,
            last_activity: now,
            permissions: vec!["read".to_string(), "write".to_string()], // Default permissions
            ip_address: client_info.ip_address,
            user_agent: client_info.user_agent,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session.clone());

        Ok(session)
    }

    async fn validate_session(&self, session_id: &str) -> Result<UserSession, DbFastError> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            let now = Utc::now();
            let session_age = now.signed_duration_since(session.last_activity);

            if session_age.num_minutes() > self.config.session_timeout_minutes {
                // Session expired
                sessions.remove(session_id);

                Err(DbFastError::Auth {
                    source: crate::errors::AuthenticationError::TokenExpired,
                    context: ErrorContext::new("session_validation", "security")
                        .with_severity(ErrorSeverity::Medium)
                        .with_detail("session_id", session_id),
                })
            } else {
                // Update last activity
                session.last_activity = now;
                Ok(session.clone())
            }
        } else {
            Err(DbFastError::Auth {
                source: crate::errors::AuthenticationError::InvalidCredentials,
                context: ErrorContext::new("session_validation", "security")
                    .with_severity(ErrorSeverity::High)
                    .with_detail("session_id", session_id),
            })
        }
    }
}

impl EncryptionManager {
    fn new(config: SecurityConfig) -> Self {
        // In production, use proper key management (HSM, key vault, etc.)
        let key = Sha256::digest(b"dbfast_encryption_key_v1").into();

        Self {
            current_key: key,
            key_rotation_time: Utc::now() + ChronoDuration::days(config.key_rotation_days as i64),
            config,
        }
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, DbFastError> {
        // Simple XOR encryption for demonstration - use AES in production
        let encrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.current_key[i % 32])
            .collect();

        Ok(encrypted)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, DbFastError> {
        // XOR is symmetric, so decryption is the same as encryption
        self.encrypt(encrypted_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = SecurityConfig {
            max_requests_per_minute: 2,
            ..Default::default()
        };

        let rate_limiter = RateLimiter::new(config);

        // First two requests should pass
        assert!(rate_limiter.check_rate_limit("client1").await.is_none());
        assert!(rate_limiter.check_rate_limit("client1").await.is_none());

        // Third request should be blocked
        let threat = rate_limiter.check_rate_limit("client1").await;
        assert!(threat.is_some());
        assert!(matches!(
            threat.unwrap().threat_type,
            ThreatType::RateLimitExceeded
        ));
    }

    #[tokio::test]
    async fn test_sql_injection_detection() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);

        let malicious_input = "'; DROP TABLE users; --";
        let threats = validator.validate_input(malicious_input).await;

        assert!(threats.is_some());
        let threats = threats.unwrap();
        assert!(!threats.is_empty());
        assert!(threats
            .iter()
            .any(|t| matches!(t.threat_type, ThreatType::SqlInjection)));
    }

    #[tokio::test]
    async fn test_authentication_flow() {
        let config = SecurityConfig::default();
        let auth_manager = AuthenticationManager::new(config);

        let client_info = ClientInfo {
            ip_address: "127.0.0.1".to_string(),
            user_agent: Some("test-agent".to_string()),
            correlation_id: Some("test-123".to_string()),
        };

        // Should not be locked initially
        assert!(!auth_manager.is_account_locked("testuser").await);

        // Create session
        let session = auth_manager.create_session("testuser", client_info).await;
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.user_id, "testuser");
        assert!(!session.session_id.is_empty());

        // Validate session
        let validated_session = auth_manager.validate_session(&session.session_id).await;
        assert!(validated_session.is_ok());
    }

    #[test]
    fn test_encryption() {
        let config = SecurityConfig::default();
        let encryption_manager = EncryptionManager::new(config);

        let data = b"sensitive data";
        let encrypted = encryption_manager.encrypt(data).unwrap();
        let decrypted = encryption_manager.decrypt(&encrypted).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_security_validation() {
        let config = SecurityConfig::default();
        let security_manager = SecurityManager::new(config);

        let context = SecurityContext {
            operation_type: OperationType::DatabaseQuery,
            client_ip: Some("192.168.1.100".to_string()),
            user_agent: Some("test-agent".to_string()),
            correlation_id: Some("test-correlation".to_string()),
            metadata: HashMap::new(),
        };

        // Test clean input
        let result = security_manager
            .validate_request(
                "client1",
                "SELECT * FROM users WHERE id = 1",
                context.clone(),
            )
            .await;
        assert!(result.is_valid);
        assert!(result.threats.is_empty());

        // Test malicious input
        let result = security_manager
            .validate_request("client1", "'; DROP TABLE users; --", context)
            .await;
        assert!(!result.is_valid);
        assert!(!result.threats.is_empty());
    }
}
