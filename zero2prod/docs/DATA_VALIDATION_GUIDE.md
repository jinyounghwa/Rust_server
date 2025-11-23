# Data Validation Guide

## Overview

Data Validation is a critical component of the Zero2Prod application that ensures data integrity across the system. It consists of two layers:

1. **Input Validation** - Validates data at the API boundary (user input)
2. **Data Validation** - Validates stored data from the database

This guide covers both validation systems and their implementation.

## Table of Contents

- [Validation Architecture](#validation-architecture)
- [Input Validation System](#input-validation-system)
- [Data Validation System](#data-validation-system)
- [Validation Rules](#validation-rules)
- [Implementation Examples](#implementation-examples)
- [Testing](#testing)
- [Best Practices](#best-practices)

## Validation Architecture

### Two-Layer Validation Model

```
User Input
    ↓
┌─────────────────────────┐
│ Input Validation Layer  │  ← Validates incoming data
│ (validators.rs)         │     - Email format
├─────────────────────────┤    - Name length/format
│ Database Storage        │     - SQL injection patterns
├─────────────────────────┤
│ Data Validation Layer   │  ← Validates stored data
│ (data_validation.rs)    │     - UUID format
└─────────────────────────┘    - Email validity
        ↓                       - Status values
   Email Sent                   - Data consistency

```

### Module Organization

**Input Validation** (`src/validators.rs`)
- Email validation: `is_valid_email()`
- Name validation: `is_valid_name()`
- Shared SQL injection detection
- Shared phishing detection

**Data Validation** (`src/data_validation.rs`)
- Complete subscriber validation: `validate_subscriber_data()`
- UUID validation: `validate_uuid()`
- Stored name validation: `validate_stored_name()`
- Status validation: `validate_subscription_status()`
- Batch validation: `validate_subscribers_batch()`

## Input Validation System

### Overview

Input validation occurs when users submit data through API endpoints. It protects against:
- Malformed data
- Phishing attacks
- SQL injection
- DoS attacks
- Data theft

### Email Validation

#### Function Signature
```rust
pub fn is_valid_email(email: &str) -> Result<String, ValidationError>
```

#### Validation Rules

| Rule | Min | Max | Notes |
|------|-----|-----|-------|
| Length | 5 chars | 254 chars | RFC 5321 standard |
| Format | RFC 5322 simplified | - | Regex pattern match |
| @ Symbol Count | 1 | 1 | Exactly one @ |
| Local Part | - | 64 chars | Before @ symbol |
| Null Bytes | 0 | 0 | Not allowed |

#### Email Validation Sequence

```
Input Email
    ↓
Trim whitespace
    ↓
Check not empty
    ↓
Check length (5-254)
    ↓
Check RFC 5322 format
    ↓
Check for suspicious patterns
    ├─ Multiple @ symbols
    ├─ Extremely long local part (>64 chars)
    └─ Null bytes
    ↓
Check for SQL injection patterns
    ├─ UNION-based
    ├─ Comment-based
    ├─ Stacked queries
    ├─ Time-based blind
    ├─ Boolean-based
    └─ Function-based
    ↓
Return validated email or error
```

#### Valid Examples
```
user@example.com
test.email@domain.co.uk
user+tag@example.com
first.last@subdomain.example.com
```

#### Invalid Examples
```
invalid                              # No @
user@@domain.com                    # Multiple @
user@                               # Missing domain
@domain.com                         # Missing local part
user' OR '1'='1@example.com        # SQL injection attempt
a@b                                 # Too short
user;\0drop@example.com            # Null byte
```

### Name Validation

#### Function Signature
```rust
pub fn is_valid_name(name: &str) -> Result<String, ValidationError>
```

#### Validation Rules

| Rule | Min | Max | Notes |
|------|-----|-----|-------|
| Length | 1 char | 256 chars | Custom limit |
| Whitespace | Allowed | - | Trimmed before check |
| Special Chars | Limited | 5 max | - | Prevents excessive chars |
| Control Chars | 0 | 0 | Not allowed |
| Null Bytes | 0 | 0 | Not allowed |

#### Name Validation Sequence

```
Input Name
    ↓
Trim whitespace
    ↓
Check not empty
    ↓
Check length (1-256)
    ↓
Check for suspicious patterns
    ├─ Null bytes
    ├─ Control characters
    └─ Excessive special characters (>5)
    ↓
Check for SQL injection patterns
    ├─ UNION-based
    ├─ Comment-based
    ├─ Stacked queries
    ├─ Time-based blind
    ├─ Boolean-based
    └─ Function-based
    ↓
Return validated name or error
```

#### Valid Examples
```
John Doe
Jean-Pierre
O'Brien
Maria Garcia-Lopez
```

#### Invalid Examples
```
                                    # Empty after trim
John'; DROP TABLE subscriptions--  # SQL injection
Name UNION SELECT *               # SQL injection
Control\x00Character              # Null byte
!!!!!!@@@@@@                       # Excessive special chars
John\t\nDoe                        # Control characters
```

### SQL Injection Detection

The system detects 6 types of SQL injection patterns:

#### 1. Union-Based Injection
```
Pattern: \s+UNION\s+
Example: "John UNION SELECT *"
```

#### 2. Comment-Based Injection
```
Patterns: --, ;, /*, */, xp_, sp_
Example: "John; DROP TABLE--"
```

#### 3. Stacked Queries
```
Pattern: ;\s*(INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)
Example: "John; DELETE FROM users"
```

#### 4. Time-Based Blind Injection
```
Pattern: (SLEEP|WAITFOR|BENCHMARK|DBMS_LOCK)
Example: "user' AND SLEEP(5)--"
```

#### 5. Boolean-Based Injection
```
Pattern: (\bOR\b|\bAND\b)\s*...
Example: "user' OR '1'='1"
```

#### 6. Function-Based Injection
```
Pattern: (CAST|CONVERT|SUBSTRING|CONCAT|LOAD_FILE)
Example: "user' AND SUBSTRING(version(), 1, 1)='5'"
```

## Data Validation System

### Overview

Data Validation validates data retrieved from the database before it's used. This ensures:
- Database integrity
- Consistency with current schema
- No corrupted records
- Safe data for operations

### Subscriber Data Validation

#### Function Signature
```rust
pub fn validate_subscriber_data(
    id: &str,
    email: &str,
    name: &str,
    status: &str,
) -> Result<(), ValidationError>
```

#### Validation Sequence

```
Database Record
    ↓
Validate ID (UUID format)
    ↓
Validate Email (RFC 5322 + extras)
    ↓
Validate Name (length + format)
    ↓
Validate Status (pending/confirmed)
    ↓
Return success or error with details
```

### UUID Validation

#### Function Signature
```rust
pub fn validate_uuid(id: &str) -> Result<(), ValidationError>
```

#### Validation Rules
- Format: `8-4-4-4-12` hexadecimal characters
- Case-insensitive (converted to lowercase for validation)
- Version: Typically v4 (random)
- Example: `550e8400-e29b-41d4-a716-446655440000`

#### Valid Examples
```
550e8400-e29b-41d4-a716-446655440000
6ba7b810-9dad-11d1-80b4-00c04fd430c8  (RFC 4122 example)
00000000-0000-0000-0000-000000000000  (All zeros - valid format)
```

#### Invalid Examples
```
not-a-uuid                           # Wrong format
550e8400-e29b-41d4-a716             # Too short
550e8400-e29b-41d4-a716-446655440000-extra  # Too long
550e8400-e29b-41d4-a716-44665544000G # Invalid hex
```

### Name Validation (Stored)

Same rules as input validation but focused on stored data integrity.

### Status Validation

#### Function Signature
```rust
pub fn validate_subscription_status(status: &str) -> Result<(), ValidationError>
```

#### Valid Values
- `pending` - Awaiting email confirmation
- `confirmed` - Email confirmed by user

#### Validation Rules
- Must be exactly one of the valid values
- Case-sensitive (lowercase required)
- No extra whitespace allowed
- No null values

#### Invalid Examples
```
"Confirmed"          # Wrong case
"active"             # Not a valid status
"unconfirmed"        # Different naming
"confirmed "         # Extra whitespace
```

### Batch Validation

#### Function Signature
```rust
pub fn validate_subscribers_batch(
    subscribers: &[(String, String, String, String)],
) -> Result<(), (usize, ValidationError)>
```

#### Purpose
Validates multiple subscriber records and reports the first error found.

#### Return Value
- `Ok(())` - All subscribers are valid
- `Err((index, error))` - Error at subscriber index with details

#### Example
```rust
let subscribers = vec![
    ("550e8400-e29b-41d4-a716-446655440000".to_string(),
     "user@example.com".to_string(),
     "John Doe".to_string(),
     "confirmed".to_string()),
    ("invalid-uuid".to_string(),  // ← Error here at index 1
     "user2@example.com".to_string(),
     "Jane Doe".to_string(),
     "pending".to_string()),
];

match validate_subscribers_batch(&subscribers) {
    Ok(_) => println!("All valid"),
    Err((idx, err)) => println!("Error at index {}: {}", idx, err),
}
```

## Validation Rules

### Complete Reference

#### Email Field
```
├─ Input Validation (is_valid_email)
│  ├─ Not empty
│  ├─ Length: 5-254 chars
│  ├─ Format: RFC 5322 simplified regex
│  ├─ Single @ symbol
│  ├─ Local part max 64 chars
│  ├─ No null bytes
│  ├─ No SQL injection patterns
│  └─ No suspicious patterns
│
└─ Data Validation (validate_stored_email via validate_subscriber_data)
   └─ Same rules as input validation
```

#### Name Field
```
├─ Input Validation (is_valid_name)
│  ├─ Not empty
│  ├─ Length: 1-256 chars
│  ├─ No control characters
│  ├─ No null bytes
│  ├─ Max 5 special characters
│  └─ No SQL injection patterns
│
└─ Data Validation (validate_stored_name)
   └─ Same rules as input validation
```

#### ID Field (UUID)
```
└─ Data Validation Only (validate_uuid)
   ├─ Not empty
   ├─ Format: 8-4-4-4-12 hex pattern
   ├─ Exactly 36 characters with hyphens
   └─ All segments contain valid hex
```

#### Status Field
```
└─ Data Validation Only (validate_subscription_status)
   ├─ Not empty
   ├─ Must be "pending" or "confirmed"
   ├─ Case-sensitive (lowercase)
   └─ No extra whitespace
```

## Implementation Examples

### Example 1: Validating Input on Subscription

```rust
// In src/routes/subscriptions.rs
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    // Input Validation - happens here
    let email = form.email.as_ref()
        .ok_or_else(|| AppError::Validation(
            ValidationError::EmptyField("email".to_string())
        ))?;

    let email = is_valid_email(email)  // ← Input validation
        .map_err(|e| AppError::Validation(e))?;

    // Rest of subscription flow...
    Ok(HttpResponse::Ok().finish())
}
```

### Example 2: Validating Stored Data Before Sending

```rust
// In src/routes/newsletters.rs
pub async fn send_newsletter_to_all(
    form: web::Json<NewsletterData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, AppError> {
    // Fetch subscribers
    let subscribers = get_all_subscribers(&pool, &error_context).await?;

    // Data Validation - happens here
    for subscriber in subscribers {
        if let Err(validation_err) = validate_subscriber_data(
            &subscriber.id,
            &subscriber.email,
            &subscriber.name,
            &subscriber.status,
        ) {
            // Log error and skip subscriber
            tracing::warn!("Subscriber data validation failed: {}", validation_err);
            failed_count += 1;
            continue;
        }

        // Safe to use subscriber data
        email_client.send_email(&subscriber.email, subject, html_content).await?;
    }

    Ok(HttpResponse::Ok().json(response))
}
```

### Example 3: Batch Validation

```rust
// Validate multiple records at once
let subscribers = vec![
    (id1.to_string(), email1, name1, status1),
    (id2.to_string(), email2, name2, status2),
    (id3.to_string(), email3, name3, status3),
];

match validate_subscribers_batch(&subscribers) {
    Ok(_) => {
        // All subscribers are valid
        process_subscribers(&subscribers).await
    }
    Err((invalid_index, error)) => {
        // Handle validation error for subscriber at invalid_index
        eprintln!("Validation failed at index {}: {}", invalid_index, error);
    }
}
```

## Testing

### Unit Tests Location
```
src/validators.rs - Test cases for input validation
src/data_validation.rs - Test cases for data validation
```

### Running Tests
```bash
# Run all validation tests
cargo test validators
cargo test data_validation

# Run specific test
cargo test is_valid_email

# Run with output
cargo test -- --nocapture
```

### Test Examples

#### Input Validation Tests
```rust
#[test]
fn test_valid_email() {
    assert!(is_valid_email("user@example.com").is_ok());
}

#[test]
fn test_invalid_email_format() {
    assert!(is_valid_email("invalid").is_err());
}

#[test]
fn test_sql_injection_in_email() {
    assert!(is_valid_email("user' OR '1'='1@example.com").is_err());
}
```

#### Data Validation Tests
```rust
#[test]
fn test_validate_subscriber_data_valid() {
    assert!(validate_subscriber_data(
        "550e8400-e29b-41d4-a716-446655440000",
        "user@example.com",
        "John Doe",
        "confirmed",
    ).is_ok());
}

#[test]
fn test_validate_uuid_invalid() {
    assert!(validate_uuid("not-a-uuid").is_err());
}
```

## Best Practices

### 1. Always Validate at Boundaries
```rust
// ✓ Good - Validate input from user
let email = is_valid_email(&user_input)?;

// ✗ Bad - Assume data is valid
let email = user_input;
```

### 2. Validate Data Before Using
```rust
// ✓ Good - Validate before sending email
validate_subscriber_data(&id, &email, &name, &status)?;
send_email(&email).await?;

// ✗ Bad - Use without validation
send_email(&email).await?;
```

### 3. Handle Validation Errors Gracefully
```rust
// ✓ Good - Log and skip invalid records
match validate_subscriber_data(...) {
    Ok(_) => send_email().await?,
    Err(e) => {
        tracing::warn!("Validation failed: {}", e);
        failed_count += 1;
        continue;
    }
}

// ✗ Bad - Crash on validation error
validate_subscriber_data(...).unwrap();
```

### 4. Use Specific Validation Functions
```rust
// ✓ Good - Use specific validators
is_valid_email(&email)?;
is_valid_name(&name)?;

// ✗ Bad - Generic validation
if email.contains("@") { /* ... */ }
```

### 5. Document Validation Rules
```rust
/// Validates email address
/// - Must be 5-254 characters
/// - Must match RFC 5322 simplified format
/// - Must not contain SQL injection patterns
pub fn is_valid_email(email: &str) -> Result<String, ValidationError>
```

## Error Messages

### Common Validation Errors

```
EmptyField("email")
→ Email field is empty

TooShort("email", 5)
→ Email is shorter than 5 characters

TooLong("email", 254)
→ Email is longer than 254 characters

InvalidFormat("email")
→ Email doesn't match RFC 5322 pattern

InvalidFormat("id")
→ UUID doesn't match required format

InvalidInput("Invalid status 'active'. Valid statuses are: pending, confirmed")
→ Status is not a valid value

PossibleSQLInjection
→ Input contains SQL injection patterns

SuspiciousContent("name")
→ Name contains suspicious patterns (null bytes, control chars)
```

## Related Documentation

- [Newsletter Feature](NEWSLETTER_FEATURE.md)
- [Error Handling](ERROR_HANDLING.md)
- [Security](SECURITY.md)
