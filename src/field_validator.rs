//! Multi-field validation error collection and aggregation
//!
//! Provides helpers for collecting validation errors across multiple fields
//! and converting them into ServiceError::ValidationErrors.
//!
//! # Example
//! ```rust
//! use pleme_error::FieldValidator;
//!
//! let mut validator = FieldValidator::new();
//! validator.add_if(email.is_empty(), "email", "Email é obrigatório");
//! validator.add_if(password.len() < 8, "password", "Senha muito curta");
//!
//! if !validator.is_empty() {
//!     return Err(validator.into_service_error());
//! }
//! ```

use crate::ServiceError;

/// Helper for collecting validation errors across multiple fields
#[derive(Debug, Default)]
pub struct FieldValidator {
    errors: Vec<(String, String)>,
}

impl FieldValidator {
    /// Create a new field validator
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add an error for a specific field
    pub fn add(&mut self, field: &str, message: &str) {
        self.errors.push((field.to_string(), message.to_string()));
    }

    /// Add an error conditionally
    pub fn add_if(&mut self, condition: bool, field: &str, message: &str) {
        if condition {
            self.add(field, message);
        }
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Get errors as field tuples
    pub fn errors(&self) -> &[(String, String)] {
        &self.errors
    }

    /// Convert into ServiceError::ValidationErrors
    pub fn into_service_error(self) -> ServiceError {
        validation_errors_from_fields(self.errors)
    }

    /// Get errors as Vec of tuples (consumes self)
    pub fn into_errors(self) -> Vec<(String, String)> {
        self.errors
    }

    /// Get errors as static str tuples (for backward compatibility)
    pub fn as_static_errors(&self) -> Vec<(&str, &str)> {
        self.errors.iter()
            .map(|(f, m)| (f.as_str(), m.as_str()))
            .collect()
    }
}

/// Create ServiceError::ValidationErrors from field tuples
pub fn validation_errors_from_fields(errors: Vec<(String, String)>) -> ServiceError {
    let error_map: std::collections::HashMap<String, Vec<String>> = errors.into_iter()
        .fold(std::collections::HashMap::new(), |mut acc, (field, msg)| {
            acc.entry(field).or_insert_with(Vec::new).push(msg);
            acc
        });

    ServiceError::ValidationErrors(error_map)
}

/// Create ServiceError::ValidationErrors from static str tuples (helper)
pub fn validation_from_fields(errors: Vec<(&str, &str)>) -> ServiceError {
    let converted: Vec<(String, String)> = errors.into_iter()
        .map(|(field, msg)| (field.to_string(), msg.to_string()))
        .collect();
    validation_errors_from_fields(converted)
}
