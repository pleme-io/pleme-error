//! # pleme-error
//!
//! Unified error handling library for Pleme platform services.
//!
//! ## Philosophy
//!
//! This library implements Railway-Oriented Programming (Scott Wlaschin) principles:
//! - Errors are first-class citizens
//! - Railway tracks: success path vs error path
//! - Composable error handling
//!
//! ## Usage
//!
//! ```rust
//! use pleme_error::{ServiceError, Result};
//!
//! fn get_user(id: &str) -> Result<User> {
//!     let user = db.find(id)
//!         .map_err(|e| ServiceError::database("User not found", e))?;
//!     Ok(user)
//! }
//! ```
//!
//! ## Features
//!
//! - `context` - Error context and chaining with anyhow
//! - `graphql` - GraphQL error conversion for async-graphql
//! - `http-errors` - HTTP status code conversion for Axum
//! - `logging` - Structured error logging with tracing
//! - `database` - Database error conversions (sqlx, Redis)
//! - `web` - Full web stack (graphql + http-errors + logging)
//! - `full` - All features enabled

use std::fmt;
use thiserror::Error;

/// Unified error type for Pleme services
///
/// Follows Railway-Oriented Programming pattern where errors flow
/// through a separate "error track" from the success track.
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Database operation failed
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Resource not found
    #[error("Not found: {resource_type} with {identifier}")]
    NotFound {
        resource_type: String,
        identifier: String,
    },

    /// Invalid input or validation error
    #[error("Invalid input: {message}")]
    InvalidInput {
        message: String,
        field: Option<String>,
    },

    /// Authentication required
    #[error("Authentication required: {0}")]
    Unauthenticated(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Business logic constraint violated
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// External service error (e.g., payment gateway, email service)
    #[error("External service error: {service} - {message}")]
    ExternalService {
        service: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Conflict (e.g., duplicate resource)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Cache operation failed
    #[error("Cache error: {message}")]
    Cache {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Operation timeout
    #[error("Timeout: {operation} exceeded {timeout_ms}ms")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Resource exhausted (memory, disk, connections, etc.)
    #[error("Resource exhausted: {resource} - {message}")]
    ResourceExhausted {
        resource: String,
        message: String,
    },

    /// Validation errors for multiple fields
    #[error("Validation failed: {0:?}")]
    ValidationErrors(std::collections::HashMap<String, Vec<String>>),

    /// Internal server error (catch-all)
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl ServiceError {
    /// Create a database error with context
    pub fn database<E>(message: impl Into<String>, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Database {
            message: message.into(),
            source: Some(Box::new(error)),
        }
    }

    /// Create a database error without source
    pub fn database_msg(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
            source: None,
        }
    }

    /// Create a not found error
    pub fn not_found(resource_type: impl Into<String>, id: impl fmt::Display) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            identifier: id.to_string(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            field: None,
        }
    }

    /// Create an invalid input error with field name
    pub fn invalid_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create an external service error
    pub fn external_service<E>(service: impl Into<String>, message: impl Into<String>, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
            source: Some(Box::new(error)),
        }
    }

    /// Create an internal error with context
    pub fn internal<E>(message: impl Into<String>, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(error)),
        }
    }

    /// Create an internal error without source
    pub fn internal_msg(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create a cache error with context
    pub fn cache<E>(message: impl Into<String>, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Cache {
            message: message.into(),
            source: Some(Box::new(error)),
        }
    }

    /// Create a cache error without source
    pub fn cache_msg(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            source: None,
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            message: message.into(),
        }
    }

    /// Add context to an error (for chaining)
    ///
    /// This method allows Railway-Oriented Programming style error chaining:
    /// ```rust
    /// load_user(id)
    ///     .context("Failed to load user profile")?;
    /// ```
    #[cfg(feature = "context")]
    pub fn context(self, message: impl Into<String>) -> Self {
        Self::Internal {
            message: format!("{}: {}", message.into(), self),
            source: Some(Box::new(self)),
        }
    }

    /// Check if error is retryable (for exponential backoff)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Database { .. }
                | Self::ExternalService { .. }
                | Self::Cache { .. }
                | Self::Timeout { .. }
                | Self::ResourceExhausted { .. }
                | Self::RateLimitExceeded(_)
                | Self::Internal { .. }
        )
    }

    /// Check if error should be logged at error level
    pub fn is_severe(&self) -> bool {
        matches!(
            self,
            Self::Database { .. }
                | Self::Internal { .. }
                | Self::Configuration(_)
                | Self::ResourceExhausted { .. }
        )
    }
}

/// Result type alias for Railway-Oriented Programming
///
/// Success track: Ok(T)
/// Error track: Err(ServiceError)
pub type Result<T> = std::result::Result<T, ServiceError>;

// ============================================================================
// Optional: anyhow context support
// ============================================================================

#[cfg(feature = "context")]
pub use anyhow::Context;

// Field validation helpers
pub mod field_validator;
pub use field_validator::{FieldValidator, validation_errors_from_fields, validation_from_fields};

#[cfg(feature = "context")]
impl From<anyhow::Error> for ServiceError {
    fn from(error: anyhow::Error) -> Self {
        Self::internal_msg(error.to_string())
    }
}

// ============================================================================
// Optional: Serialization support
// ============================================================================

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serialization")]
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[cfg(feature = "serialization")]
impl From<ServiceError> for ErrorResponse {
    fn from(error: ServiceError) -> Self {
        let error_type = match &error {
            ServiceError::Database { .. } => "DATABASE_ERROR",
            ServiceError::NotFound { .. } => "NOT_FOUND",
            ServiceError::InvalidInput { .. } => "INVALID_INPUT",
            ServiceError::ValidationErrors(_) => "VALIDATION_ERRORS",
            ServiceError::Unauthenticated(_) => "UNAUTHENTICATED",
            ServiceError::PermissionDenied(_) => "PERMISSION_DENIED",
            ServiceError::BusinessRule(_) => "BUSINESS_RULE_VIOLATION",
            ServiceError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            ServiceError::Configuration(_) => "CONFIGURATION_ERROR",
            ServiceError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ServiceError::Conflict(_) => "CONFLICT",
            ServiceError::Cache { .. } => "CACHE_ERROR",
            ServiceError::Timeout { .. } => "TIMEOUT",
            ServiceError::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            ServiceError::Internal { .. } => "INTERNAL_ERROR",
        };

        let field = match &error {
            ServiceError::InvalidInput { field, .. } => field.clone(),
            _ => None,
        };

        Self {
            error: error_type.to_string(),
            message: error.to_string(),
            field,
            details: None,
        }
    }
}

// ============================================================================
// Optional: GraphQL error conversion
// ============================================================================

#[cfg(feature = "graphql")]
use async_graphql::ErrorExtensions;

#[cfg(feature = "graphql")]
impl ServiceError {
    /// Convert to GraphQL error with structured extensions
    ///
    /// Provides rich error information for GraphQL clients:
    /// - error code for client-side handling
    /// - retryable flag for retry logic
    /// - severe flag for logging decisions
    /// - field information for validation errors
    /// - operation-specific metadata
    pub fn into_graphql_error(self) -> async_graphql::Error {
        let message = self.to_string();

        // Add error code for client-side handling
        let code = match &self {
            ServiceError::NotFound { .. } => "NOT_FOUND",
            ServiceError::InvalidInput { .. } => "INVALID_INPUT",
            ServiceError::ValidationErrors(_) => "VALIDATION_ERRORS",
            ServiceError::Unauthenticated(_) => "UNAUTHENTICATED",
            ServiceError::PermissionDenied(_) => "PERMISSION_DENIED",
            ServiceError::BusinessRule(_) => "BUSINESS_RULE_VIOLATION",
            ServiceError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ServiceError::Conflict(_) => "CONFLICT",
            ServiceError::Cache { .. } => "CACHE_ERROR",
            ServiceError::Timeout { .. } => "TIMEOUT",
            ServiceError::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            _ => "INTERNAL_ERROR",
        };

        // Extract field information if present
        let field = match &self {
            ServiceError::InvalidInput { field, .. } => field.clone(),
            _ => None,
        };

        let retryable = self.is_retryable();
        let severe = self.is_severe();

        // Build structured error with extensions
        let mut error = async_graphql::Error::new(message);
        error = error.extend_with(|_, e| {
            e.set("code", code);
            e.set("retryable", retryable);
            e.set("severe", severe);

            if let Some(field_name) = field {
                e.set("field", field_name);
            }

            // Add timeout details for timeout errors
            if let ServiceError::Timeout { operation, timeout_ms } = &self {
                e.set("operation", operation.clone());
                e.set("timeout_ms", *timeout_ms);
            }

            // Add resource details for resource exhausted errors
            if let ServiceError::ResourceExhausted { resource, .. } = &self {
                e.set("resource", resource.clone());
            }
        });

        error
    }
}

// ============================================================================
// Optional: HTTP error conversion (Axum)
// ============================================================================

#[cfg(feature = "http-errors")]
impl From<ServiceError> for http::StatusCode {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::NotFound { .. } => http::StatusCode::NOT_FOUND,
            ServiceError::InvalidInput { .. } => http::StatusCode::BAD_REQUEST,
            ServiceError::ValidationErrors(_) => http::StatusCode::BAD_REQUEST,
            ServiceError::Unauthenticated(_) => http::StatusCode::UNAUTHORIZED,
            ServiceError::PermissionDenied(_) => http::StatusCode::FORBIDDEN,
            ServiceError::BusinessRule(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
            ServiceError::RateLimitExceeded(_) => http::StatusCode::TOO_MANY_REQUESTS,
            ServiceError::Conflict(_) => http::StatusCode::CONFLICT,
            ServiceError::Timeout { .. } => http::StatusCode::GATEWAY_TIMEOUT,
            ServiceError::ResourceExhausted { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
            ServiceError::Database { .. }
            | ServiceError::Cache { .. }
            | ServiceError::ExternalService { .. }
            | ServiceError::Configuration(_)
            | ServiceError::Internal { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "http-errors")]
impl axum::response::IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        // Extract status code before consuming self
        let status = match &self {
            ServiceError::NotFound { .. } => http::StatusCode::NOT_FOUND,
            ServiceError::InvalidInput { .. } => http::StatusCode::BAD_REQUEST,
            ServiceError::ValidationErrors(_) => http::StatusCode::BAD_REQUEST,
            ServiceError::Unauthenticated(_) => http::StatusCode::UNAUTHORIZED,
            ServiceError::PermissionDenied(_) => http::StatusCode::FORBIDDEN,
            ServiceError::BusinessRule(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
            ServiceError::RateLimitExceeded(_) => http::StatusCode::TOO_MANY_REQUESTS,
            ServiceError::Conflict(_) => http::StatusCode::CONFLICT,
            ServiceError::Timeout { .. } => http::StatusCode::GATEWAY_TIMEOUT,
            ServiceError::ResourceExhausted { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
            ServiceError::Database { .. }
            | ServiceError::Cache { .. }
            | ServiceError::ExternalService { .. }
            | ServiceError::Configuration(_)
            | ServiceError::Internal { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        #[cfg(feature = "serialization")]
        {
            let body = ErrorResponse::from(self);
            (status, axum::Json(body)).into_response()
        }

        #[cfg(not(feature = "serialization"))]
        {
            (status, self.to_string()).into_response()
        }
    }
}

// ============================================================================
// Optional: Database error conversions
// ============================================================================

#[cfg(feature = "database")]
impl From<sqlx::Error> for ServiceError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => {
                Self::not_found("record", "unknown")
            }
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(constraint) = db_err.constraint() {
                    Self::Conflict(format!("Constraint violation: {}", constraint))
                } else {
                    Self::database_msg(db_err.message())
                }
            }
            _ => Self::database("Database operation failed", error),
        }
    }
}

#[cfg(feature = "database")]
impl From<redis::RedisError> for ServiceError {
    fn from(error: redis::RedisError) -> Self {
        Self::database("Redis operation failed", error)
    }
}

// ============================================================================
// Optional: Common error type conversions
// ============================================================================

#[cfg(feature = "serialization")]
impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidInput {
            message: format!("JSON parsing failed: {}", error),
            field: None,
        }
    }
}

impl From<url::ParseError> for ServiceError {
    fn from(error: url::ParseError) -> Self {
        Self::InvalidInput {
            message: format!("URL parsing failed: {}", error),
            field: None,
        }
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound {
                resource_type: "file".to_string(),
                identifier: error.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => {
                Self::PermissionDenied(format!("I/O permission denied: {}", error))
            }
            std::io::ErrorKind::TimedOut => Self::timeout("I/O operation", 0),
            _ => Self::internal("I/O error", error),
        }
    }
}

// ============================================================================
// Optional: Structured logging
// ============================================================================

#[cfg(feature = "logging")]
pub fn log_error(error: &ServiceError, context: &str) {
    if error.is_severe() {
        tracing::error!(
            error = %error,
            context = context,
            "Service error occurred"
        );
    } else {
        tracing::warn!(
            error = %error,
            context = context,
            "Service error occurred"
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_error_creation() {
        let err = ServiceError::not_found("User", Uuid::new_v4());
        assert!(matches!(err, ServiceError::NotFound { .. }));

        let err = ServiceError::invalid_input("Invalid email");
        assert!(matches!(err, ServiceError::InvalidInput { .. }));

        let err = ServiceError::invalid_field("email", "Must be valid");
        assert!(matches!(err, ServiceError::InvalidInput { field: Some(_), .. }));
    }

    #[test]
    fn test_error_retryable() {
        assert!(ServiceError::database_msg("Connection failed").is_retryable());
        assert!(ServiceError::RateLimitExceeded("Too many requests".to_string()).is_retryable());
        assert!(!ServiceError::not_found("User", "123").is_retryable());
        assert!(!ServiceError::InvalidInput {
            message: "Bad input".to_string(),
            field: None
        }
        .is_retryable());
    }

    #[test]
    fn test_error_severity() {
        assert!(ServiceError::database_msg("Connection failed").is_severe());
        assert!(ServiceError::internal_msg("Panic").is_severe());
        assert!(!ServiceError::not_found("User", "123").is_severe());
        assert!(!ServiceError::InvalidInput {
            message: "Bad input".to_string(),
            field: None
        }
        .is_severe());
    }

    #[cfg(feature = "serialization")]
    #[test]
    fn test_error_serialization() {
        let err = ServiceError::not_found("User", "123");
        let response = ErrorResponse::from(err);
        assert_eq!(response.error, "NOT_FOUND");
        assert!(response.message.contains("User"));
    }

    #[cfg(feature = "http-errors")]
    #[test]
    fn test_http_status_conversion() {
        use http::StatusCode;

        assert_eq!(
            StatusCode::from(ServiceError::not_found("User", "123")),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            StatusCode::from(ServiceError::invalid_input("Bad")),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            StatusCode::from(ServiceError::Unauthenticated("Login required".to_string())),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            StatusCode::from(ServiceError::timeout("database query", 5000)),
            StatusCode::GATEWAY_TIMEOUT
        );
        assert_eq!(
            StatusCode::from(ServiceError::resource_exhausted("memory", "Out of memory")),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_new_error_variants() {
        let cache_err = ServiceError::cache_msg("Cache miss");
        assert!(matches!(cache_err, ServiceError::Cache { .. }));
        assert!(cache_err.is_retryable());

        let timeout_err = ServiceError::timeout("API call", 3000);
        assert!(matches!(timeout_err, ServiceError::Timeout { .. }));
        assert!(timeout_err.is_retryable());
        assert!(timeout_err.to_string().contains("3000ms"));

        let exhausted_err = ServiceError::resource_exhausted("connections", "Pool exhausted");
        assert!(matches!(exhausted_err, ServiceError::ResourceExhausted { .. }));
        assert!(exhausted_err.is_retryable());
        assert!(exhausted_err.is_severe());
    }

    #[cfg(feature = "context")]
    #[test]
    fn test_context_method() {
        let err = ServiceError::not_found("User", "123");
        let with_context = err.context("Failed to load user profile");

        assert!(matches!(with_context, ServiceError::Internal { .. }));
        assert!(with_context.to_string().contains("Failed to load user profile"));
        assert!(with_context.to_string().contains("User"));
    }

    #[cfg(feature = "graphql")]
    #[test]
    fn test_graphql_error_conversion() {
        let timeout_err = ServiceError::timeout("database query", 5000);
        let graphql_err = timeout_err.into_graphql_error();

        // async-graphql doesn't provide direct access to extensions in tests,
        // but we can verify the error message is present
        assert!(graphql_err.message.contains("database query"));
    }

    #[cfg(feature = "serialization")]
    #[test]
    fn test_json_error_conversion() {
        let json_str = r#"{"invalid": json"#;
        let result: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(json_str);

        if let Err(json_err) = result {
            let service_err = ServiceError::from(json_err);
            assert!(matches!(service_err, ServiceError::InvalidInput { .. }));
            assert!(service_err.to_string().contains("JSON parsing failed"));
        }
    }

    #[test]
    fn test_url_error_conversion() {
        let invalid_url = "not a valid url";
        let result = url::Url::parse(invalid_url);

        if let Err(url_err) = result {
            let service_err = ServiceError::from(url_err);
            assert!(matches!(service_err, ServiceError::InvalidInput { .. }));
            assert!(service_err.to_string().contains("URL parsing failed"));
        }
    }
}
