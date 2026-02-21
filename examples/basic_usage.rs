//! Basic usage examples for pleme-error

use pleme_error::{ServiceError, Result};
use uuid::Uuid;

// Example domain type
#[derive(Debug)]
struct User {
    id: Uuid,
    email: String,
}

/// Example: Not found errors
fn find_user(id: Uuid) -> Result<User> {
    // Simulating database lookup that doesn't find user
    Err(ServiceError::not_found("User", id))
}

/// Example: Invalid input errors
fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') {
        return Err(ServiceError::invalid_field("email", "Must contain @"));
    }
    if email.len() < 3 {
        return Err(ServiceError::invalid_field("email", "Too short"));
    }
    Ok(())
}

/// Example: Business rule violations
fn cancel_order(order_status: &str) -> Result<()> {
    if order_status == "shipped" {
        return Err(ServiceError::BusinessRule(
            "Cannot cancel shipped orders".to_string()
        ));
    }
    Ok(())
}

/// Example: Railway-oriented error chaining
fn register_user(email: String) -> Result<User> {
    // Each step may jump to error track
    validate_email(&email)?;  // Validation layer

    // Simulate creating user
    let user = User {
        id: Uuid::new_v4(),
        email,
    };

    Ok(user)  // Success track
}

/// Example: Error retryability
fn handle_with_retry(error: ServiceError) {
    if error.is_retryable() {
        println!("Error is retryable: {}", error);
        println!("Implementing exponential backoff...");
    } else {
        println!("Error is not retryable: {}", error);
        println!("Failing fast.");
    }
}

/// Example: Error severity
fn log_error_appropriately(error: ServiceError) {
    if error.is_severe() {
        eprintln!("SEVERE: {}", error);
    } else {
        println!("Expected error: {}", error);
    }
}

fn main() {
    println!("=== Basic pleme-error Examples ===\n");

    // Example 1: Not Found
    println!("1. Not Found Error:");
    let user_id = Uuid::new_v4();
    match find_user(user_id) {
        Ok(_) => println!("User found!"),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 2: Invalid Input
    println!("2. Invalid Input Error:");
    match validate_email("invalid") {
        Ok(_) => println!("Email is valid!"),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 3: Business Rule
    println!("3. Business Rule Violation:");
    match cancel_order("shipped") {
        Ok(_) => println!("Order cancelled!"),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 4: Railway-Oriented Programming
    println!("4. Railway-Oriented Programming:");
    match register_user("invalid".to_string()) {
        Ok(user) => println!("User registered: {:?}", user),
        Err(e) => println!("Registration failed: {}", e),
    }
    println!();

    // Example 5: Error Retryability
    println!("5. Error Retryability:");
    let db_error = ServiceError::database_msg("Connection timeout");
    handle_with_retry(db_error);

    let not_found = ServiceError::not_found("User", "123");
    handle_with_retry(not_found);
    println!();

    // Example 6: Error Severity
    println!("6. Error Severity:");
    let severe = ServiceError::internal_msg("Panic in worker thread");
    log_error_appropriately(severe);

    let expected = ServiceError::not_found("User", "123");
    log_error_appropriately(expected);
}
