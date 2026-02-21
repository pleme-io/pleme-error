//! Railway-Oriented Programming examples
//!
//! Run with: cargo run --example railway_oriented --features context

#[cfg(feature = "context")]
use pleme_error::{ServiceError, Result, Context};
#[cfg(feature = "context")]
use uuid::Uuid;

#[cfg(feature = "context")]
#[derive(Debug)]
struct User {
    id: Uuid,
    email: String,
    age: u32,
}

#[cfg(feature = "context")]
#[derive(Debug)]
struct RegisterInput {
    email: String,
    password: String,
    age: u32,
}

#[cfg(feature = "context")]
/// Step 1: Validation (may switch to error track)
fn validate_input(input: &RegisterInput) -> Result<()> {
    // Email validation
    if !input.email.contains('@') {
        return Err(ServiceError::invalid_field("email", "Must be valid email"));
    }

    // Password validation
    if input.password.len() < 8 {
        return Err(ServiceError::invalid_field("password", "Must be at least 8 characters"));
    }

    // Age validation
    if input.age < 18 {
        return Err(ServiceError::invalid_field("age", "Must be 18 or older"));
    }

    Ok(())  // Stay on success track
}

#[cfg(feature = "context")]
/// Step 2: Check uniqueness (may switch to error track)
fn check_unique_email(email: &str) -> Result<()> {
    // Simulating database check
    let existing_users = vec!["existing@example.com", "taken@example.com"];

    if existing_users.contains(&email) {
        return Err(ServiceError::Conflict(
            "Email already registered".to_string()
        ));
    }

    Ok(())  // Stay on success track
}

#[cfg(feature = "context")]
/// Step 3: Create user (may switch to error track)
fn create_user(input: RegisterInput) -> Result<User> {
    // Simulating user creation
    Ok(User {
        id: Uuid::new_v4(),
        email: input.email,
        age: input.age,
    })
}

#[cfg(feature = "context")]
/// Step 4: Send welcome email (may switch to error track)
fn send_welcome_email(user: &User) -> Result<()> {
    // Simulating email service
    println!("📧 Sending welcome email to {}", user.email);

    // Simulate email service failure
    if user.email.contains("bounce") {
        return Err(ServiceError::external_service(
            "SendGrid",
            "Email bounce",
            std::io::Error::new(std::io::ErrorKind::Other, "Bounce")
        ));
    }

    Ok(())  // Stay on success track
}

#[cfg(feature = "context")]
/// Railway-Oriented Programming: Chain of operations
/// Each step may jump to error track
fn register_user(input: RegisterInput) -> Result<User> {
    // Success track ──┐
    //                 ├──> validate_input ──┐
    //                 │                     ├──> check_unique_email ──┐
    //                 │                     │                         ├──> create_user ──┐
    //                 │                     │                         │                  ├──> send_email ──> Ok(user)
    //                 │                     │                         │                  │
    // Error track ────┴─────────────────────┴─────────────────────────┴──────────────────┴───────────────> Err(e)

    validate_input(&input)?;           // May jump to error track
    check_unique_email(&input.email)?; // May jump to error track
    let user = create_user(input)?;    // May jump to error track
    send_welcome_email(&user)?;        // May jump to error track

    Ok(user)  // Success track
}

#[cfg(feature = "context")]
/// Example: Using context for debugging
fn load_config() -> Result<String> {
    std::fs::read_to_string("nonexistent.yaml")
        .context("Failed to read config file")
        .map_err(|e| ServiceError::Configuration(e.to_string()))
}

#[cfg(feature = "context")]
fn main() {
    println!("=== Railway-Oriented Programming Examples ===\n");

    // Success track example
    println!("1. Success Track:");
    let valid_input = RegisterInput {
        email: "alice@example.com".to_string(),
        password: "secure123".to_string(),
        age: 25,
    };

    match register_user(valid_input) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error track: Invalid email
    println!("2. Error Track - Invalid Email:");
    let invalid_email = RegisterInput {
        email: "invalid".to_string(),  // No @
        password: "secure123".to_string(),
        age: 25,
    };

    match register_user(invalid_email) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error track: Weak password
    println!("3. Error Track - Weak Password:");
    let weak_password = RegisterInput {
        email: "bob@example.com".to_string(),
        password: "123".to_string(),  // Too short
        age: 25,
    };

    match register_user(weak_password) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error track: Underage
    println!("4. Error Track - Underage:");
    let underage = RegisterInput {
        email: "young@example.com".to_string(),
        password: "secure123".to_string(),
        age: 16,  // Under 18
    };

    match register_user(underage) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error track: Duplicate email
    println!("5. Error Track - Duplicate Email:");
    let duplicate = RegisterInput {
        email: "existing@example.com".to_string(),  // Already exists
        password: "secure123".to_string(),
        age: 25,
    };

    match register_user(duplicate) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error track: Email bounce
    println!("6. Error Track - Email Service Failure:");
    let bounce = RegisterInput {
        email: "bounce@example.com".to_string(),  // Will bounce
        password: "secure123".to_string(),
        age: 25,
    };

    match register_user(bounce) {
        Ok(user) => println!("✓ User registered: {:?}", user),
        Err(e) => println!("✗ Registration failed: {}", e),
    }
    println!();

    // Error context example
    println!("7. Error Context:");
    match load_config() {
        Ok(config) => println!("✓ Config loaded: {}", config),
        Err(e) => println!("✗ Config load failed: {}", e),
    }
    println!();

    println!("\n=== Key Principles ===");
    println!("1. Two tracks: Success path and Error path");
    println!("2. Any operation can switch to error track (using ?)");
    println!("3. Once on error track, skip remaining operations");
    println!("4. Errors flow to final error handler");
    println!("5. Composable: Each function returns Result<T>");
}

#[cfg(not(feature = "context"))]
fn main() {
    eprintln!("This example requires the 'context' feature.");
    eprintln!("Run with: cargo run --example railway_oriented --features context");
}
