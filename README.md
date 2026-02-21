# pleme-error

Unified error handling library for Pleme platform services.

## Philosophy

This library implements **Railway-Oriented Programming** (Scott Wlaschin) principles:
- Errors are first-class citizens
- Two tracks: success path vs error path
- Composable error handling
- Fail-fast with meaningful context

## Industry Standards

- **Railway-Oriented Programming** (Scott Wlaschin) - Functional error handling
- **Domain-Driven Design** (Eric Evans) - Domain-specific error types
- **12-Factor App** - Treat errors as event streams
- **Rust Error Handling** - thiserror for custom errors, anyhow for context

## Features

- `context` - Error context and chaining with anyhow
- `graphql` - GraphQL error conversion for async-graphql
- `http-errors` - HTTP status code conversion for Axum
- `logging` - Structured error logging with tracing
- `database` - Database error conversions (sqlx, Redis)
- `web` - Full web stack (graphql + http-errors + logging)
- `full` - All features enabled

## Usage

### Basic Usage

```rust
use pleme_error::{ServiceError, Result};

fn get_user(id: &str) -> Result<User> {
    let user = db.find(id)
        .ok_or_else(|| ServiceError::not_found("User", id))?;
    Ok(user)
}
```

### Railway-Oriented Programming

```rust
use pleme_error::{ServiceError, Result};

// Success track: User -> Validation -> Persistence -> Email
// Error track: Any error short-circuits to error response

fn register_user(input: RegisterInput) -> Result<User> {
    let user = validate_input(input)?;     // ← May jump to error track
    let saved = persist_user(user)?;       // ← May jump to error track
    send_welcome_email(&saved)?;           // ← May jump to error track
    Ok(saved)                              // ← Success track
}
```

### Error Types

```rust
// Not found
ServiceError::not_found("User", user_id)

// Invalid input
ServiceError::invalid_input("Email must be valid")
ServiceError::invalid_field("email", "Must be valid")

// Database error
db.query().await
    .map_err(|e| ServiceError::database("Query failed", e))?

// External service error
stripe.charge(amount).await
    .map_err(|e| ServiceError::external_service("Stripe", "Payment failed", e))?

// Authentication/Authorization
ServiceError::Unauthenticated("Login required".to_string())
ServiceError::PermissionDenied("Admin only".to_string())

// Business rules
ServiceError::BusinessRule("Cannot refund shipped orders".to_string())

// Conflict
ServiceError::Conflict("Email already registered".to_string())
```

### GraphQL Integration

```toml
[dependencies]
pleme-error = { path = "../pleme-error", features = ["graphql"] }
```

```rust
use pleme_error::{ServiceError, Result};
use async_graphql::{Context, Object};

#[Object]
impl Query {
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<User> {
        let user = db.find(&id).await
            .ok_or_else(|| ServiceError::not_found("User", id))?;
        Ok(user)  // Automatically converts to GraphQL error on Err
    }
}
```

GraphQL error response:
```json
{
  "errors": [{
    "message": "Not found: User with 550e8400-e29b-41d4-a716-446655440000",
    "extensions": {
      "code": "NOT_FOUND"
    }
  }]
}
```

### HTTP/Axum Integration

```toml
[dependencies]
pleme-error = { path = "../pleme-error", features = ["http-errors", "serialization"] }
```

```rust
use pleme_error::{ServiceError, Result};
use axum::{Router, routing::get, Json};

async fn get_user(Path(id): Path<String>) -> Result<Json<User>> {
    let user = db.find(&id).await
        .ok_or_else(|| ServiceError::not_found("User", id))?;
    Ok(Json(user))  // Automatically converts to HTTP response on Err
}
```

HTTP error response (404 Not Found):
```json
{
  "error": "NOT_FOUND",
  "message": "Not found: User with 123",
  "field": null,
  "details": null
}
```

### Database Integration

```toml
[dependencies]
pleme-error = { path = "../pleme-error", features = ["database"] }
```

```rust
use pleme_error::Result;

async fn create_user(email: String) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email) VALUES ($1) RETURNING *",
        email
    )
    .fetch_one(&pool)
    .await?;  // Automatically converts sqlx::Error to ServiceError

    Ok(user)
}
```

Constraint violations become `ServiceError::Conflict`:
```rust
// Duplicate email → ServiceError::Conflict("Constraint violation: users_email_key")
```

### Structured Logging

```toml
[dependencies]
pleme-error = { path = "../pleme-error", features = ["logging"] }
```

```rust
use pleme_error::{ServiceError, log_error};

fn process_payment(order_id: &str) -> Result<Payment> {
    match charge_payment(order_id) {
        Ok(payment) => Ok(payment),
        Err(e) => {
            log_error(&e, "payment_processing");  // Structured logging
            Err(e)
        }
    }
}
```

Output:
```
WARN Service error occurred, error="External service error: Stripe - Card declined", context="payment_processing"
```

### Error Context (anyhow pattern)

```toml
[dependencies]
pleme-error = { path = "../pleme-error", features = ["context"] }
```

```rust
use pleme_error::{ServiceError, Result, Context};

fn load_config() -> Result<Config> {
    let file = std::fs::read_to_string("config.yaml")
        .context("Failed to read config.yaml")?;  // Add context

    let config: Config = serde_yaml::from_str(&file)
        .context("Failed to parse config.yaml")?;  // Add context

    Ok(config)
}
```

### Retry Logic

```rust
use pleme_error::ServiceError;

async fn with_retry<F, T>(mut f: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempts = 0;

    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if e.is_retryable() && attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Design Principles

### 1. Fail-Closed Security
Errors default to denial. `PermissionDenied` and `Unauthenticated` are explicit.

### 2. Meaningful Messages
Error messages guide users and operators to solutions.

```rust
// ❌ BAD
ServiceError::internal_msg("Error")

// ✅ GOOD
ServiceError::database("Failed to connect to PostgreSQL at db.example.com:5432", error)
```

### 3. Structured Error Codes
GraphQL and HTTP responses include machine-readable error codes (`NOT_FOUND`, `INVALID_INPUT`, etc.)

### 4. Severity Levels
Errors know if they're severe (database failure) vs expected (not found).

```rust
if error.is_severe() {
    tracing::error!("Critical error: {}", error);
} else {
    tracing::warn!("Expected error: {}", error);
}
```

## Error Handling Patterns

### Pattern 1: Validation Errors

```rust
fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') {
        return Err(ServiceError::invalid_field("email", "Must contain @"));
    }
    Ok(())
}
```

### Pattern 2: Not Found vs Empty

```rust
// NOT FOUND: Expected resource doesn't exist (404)
db.find_by_id(id).await
    .ok_or_else(|| ServiceError::not_found("User", id))?

// EMPTY: Query returned no results (200 OK with [])
let users: Vec<User> = db.find_all().await?;  // Returns Ok(vec![])
```

### Pattern 3: Business Rule Violations

```rust
if order.status == OrderStatus::Shipped {
    return Err(ServiceError::BusinessRule(
        "Cannot cancel shipped orders".to_string()
    ));
}
```

### Pattern 4: External Service Failures

```rust
let payment = stripe_client
    .charge(amount)
    .await
    .map_err(|e| ServiceError::external_service("Stripe", "Charge failed", e))?;
```

## Migration Guide

### From Custom Error Types

```rust
// BEFORE
#[derive(Error, Debug)]
pub enum MyServiceError {
    #[error("Not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(String),
}

// AFTER
use pleme_error::{ServiceError, Result};

// Just use ServiceError directly!
```

### From Direct Database Errors

```rust
// BEFORE
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await?;  // Exposes sqlx::Error to GraphQL/HTTP

// AFTER
use pleme_error::{ServiceError, Result};

let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await?;  // Automatically converts to ServiceError with proper HTTP status
```

## Best Practices

1. **Use descriptive error messages** - Help operators debug issues
2. **Include context** - What operation failed? What resource?
3. **Don't leak sensitive data** - No database connection strings, API keys, etc.
4. **Use proper error types** - `NotFound` vs `InvalidInput` vs `Internal`
5. **Log severe errors** - Database failures, panics, external service outages
6. **Don't log expected errors** - Not found, validation failures
7. **Add retry logic for retryable errors** - Use `is_retryable()`

## Testing

```rust
#[cfg(test)]
mod tests {
    use pleme_error::{ServiceError, Result};

    #[test]
    fn test_error_handling() {
        let result: Result<User> = Err(ServiceError::not_found("User", "123"));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ServiceError::NotFound { .. }
        ));
    }

    #[test]
    fn test_retryable() {
        let db_error = ServiceError::database_msg("Connection timeout");
        assert!(db_error.is_retryable());

        let not_found = ServiceError::not_found("User", "123");
        assert!(!not_found.is_retryable());
    }
}
```

## See Also

- [Railway-Oriented Programming](https://fsharpforfunandprofit.com/posts/recipe-part2/) - Scott Wlaschin
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror](https://docs.rs/thiserror/) - Custom error types
- [anyhow](https://docs.rs/anyhow/) - Error context and chaining
