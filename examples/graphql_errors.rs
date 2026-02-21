//! GraphQL error conversion examples
//!
//! Run with: cargo run --example graphql_errors --features graphql

#[cfg(feature = "graphql")]
use async_graphql::{EmptySubscription, Object, Schema};
#[cfg(feature = "graphql")]
use pleme_error::{ServiceError, Result};
#[cfg(feature = "graphql")]
use uuid::Uuid;

#[cfg(feature = "graphql")]
#[derive(Debug, Clone)]
struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[cfg(feature = "graphql")]
struct Query;

#[cfg(feature = "graphql")]
#[Object]
impl Query {
    /// Example: Not found error
    async fn user(&self, id: String) -> Result<User> {
        // Simulating user not found
        Err(ServiceError::not_found("User", id))
    }

    /// Example: Invalid input error
    async fn users(&self, limit: i32) -> Result<Vec<User>> {
        if limit < 1 {
            return Err(ServiceError::invalid_field(
                "limit",
                "Must be greater than 0"
            ));
        }
        if limit > 100 {
            return Err(ServiceError::invalid_field(
                "limit",
                "Must be 100 or less"
            ));
        }

        Ok(vec![])
    }
}

#[cfg(feature = "graphql")]
struct Mutation;

#[cfg(feature = "graphql")]
#[Object]
impl Mutation {
    /// Example: Authentication required
    async fn update_user(&self, id: String, name: String) -> Result<User> {
        // Simulating unauthenticated request
        Err(ServiceError::Unauthenticated("Login required".to_string()))
    }

    /// Example: Permission denied
    async fn delete_user(&self, id: String) -> Result<bool> {
        // Simulating permission check failure
        Err(ServiceError::PermissionDenied("Admin access required".to_string()))
    }

    /// Example: Business rule violation
    async fn cancel_order(&self, order_id: String) -> Result<bool> {
        // Simulating shipped order
        Err(ServiceError::BusinessRule(
            "Cannot cancel shipped orders".to_string()
        ))
    }
}

#[cfg(feature = "graphql")]
#[tokio::main]
async fn main() {
    println!("=== GraphQL Error Conversion Examples ===\n");

    let schema = Schema::new(Query, Mutation, EmptySubscription);

    // Example 1: Not Found Error
    println!("1. Not Found Error (404):");
    let query = r#"{ user(id: "550e8400-e29b-41d4-a716-446655440000") { id name } }"#;
    let result = schema.execute(query).await;
    println!("{:#?}\n", result.errors);

    // Example 2: Invalid Input Error
    println!("2. Invalid Input Error (400):");
    let query = r#"{ users(limit: 0) { id name } }"#;
    let result = schema.execute(query).await;
    println!("{:#?}\n", result.errors);

    // Example 3: Authentication Required (401)
    println!("3. Authentication Required (401):");
    let query = r#"mutation { updateUser(id: "123", name: "New Name") { id name } }"#;
    let result = schema.execute(query).await;
    println!("{:#?}\n", result.errors);

    // Example 4: Permission Denied (403)
    println!("4. Permission Denied (403):");
    let query = r#"mutation { deleteUser(id: "123") }"#;
    let result = schema.execute(query).await;
    println!("{:#?}\n", result.errors);

    // Example 5: Business Rule Violation (422)
    println!("5. Business Rule Violation (422):");
    let query = r#"mutation { cancelOrder(orderId: "123") }"#;
    let result = schema.execute(query).await;
    println!("{:#?}\n", result.errors);

    println!("\nNote: Each error includes:");
    println!("  - Human-readable message");
    println!("  - Machine-readable code (NOT_FOUND, INVALID_INPUT, etc.)");
    println!("  - Proper GraphQL error structure");
}

#[cfg(not(feature = "graphql"))]
fn main() {
    eprintln!("This example requires the 'graphql' feature.");
    eprintln!("Run with: cargo run --example graphql_errors --features graphql");
}
