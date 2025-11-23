# Newsletter Feature Documentation

## Overview

The Newsletter Feature provides comprehensive email distribution capabilities to your subscriber base. It includes three core functionalities:

1. **Send Email to All Subscribers** - Send emails to all subscribers (both confirmed and pending)
2. **Send Email to Confirmed Subscribers Only** - Send emails only to subscribers who have confirmed their subscription
3. **Data Validation for Stored Subscribers** - Automatic validation of subscriber data before sending emails

## Features

### 1. Send Newsletter to All Subscribers

#### Endpoint
```
POST /newsletters/send-all
```

#### Description
Sends a newsletter to all subscribers in the database, regardless of their confirmation status.

#### Request Body
```json
{
  "subject": "Newsletter Subject",
  "html_content": "<h1>Welcome to our Newsletter</h1><p>Your content here...</p>"
}
```

#### Request Parameters
- `subject` (string, required): The email subject line
  - Must not be empty
  - Maximum length is reasonable for email standards

- `html_content` (string, required): The HTML content of the email
  - Must not be empty
  - Can include any valid HTML

#### Response - Success (200 OK)
```json
{
  "message": "Newsletter sent to all subscribers",
  "sent_count": 150,
  "failed_count": 2
}
```

#### Response - Empty Database (200 OK)
```json
{
  "message": "No subscribers found",
  "sent_count": 0
}
```

#### Response - Validation Error (400 Bad Request)
```json
{
  "error_id": "unique-request-id",
  "message": "Missing required field: subject",
  "code": "VALIDATION_ERROR",
  "status": 400,
  "timestamp": "2024-11-23T10:30:00Z"
}
```

#### Example Usage (cURL)
```bash
curl -X POST http://localhost:8000/newsletters/send-all \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "January Newsletter",
    "html_content": "<h1>Welcome</h1><p>January updates here</p>"
  }'
```

#### Example Usage (JavaScript/Fetch)
```javascript
const response = await fetch('http://localhost:8000/newsletters/send-all', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    subject: 'January Newsletter',
    html_content: '<h1>Welcome</h1><p>January updates here</p>'
  })
});

const result = await response.json();
console.log(`Sent: ${result.sent_count}, Failed: ${result.failed_count}`);
```

### 2. Send Newsletter to Confirmed Subscribers Only

#### Endpoint
```
POST /newsletters/send-confirmed
```

#### Description
Sends a newsletter only to subscribers who have confirmed their email address. This is the preferred method for important communications as it ensures subscribers actively opted in.

#### Request Body
```json
{
  "subject": "Newsletter Subject",
  "html_content": "<h1>Welcome to our Newsletter</h1><p>Your content here...</p>"
}
```

#### Request Parameters
Same as `/newsletters/send-all`

#### Response - Success (200 OK)
```json
{
  "message": "Newsletter sent to confirmed subscribers",
  "sent_count": 145,
  "failed_count": 0
}
```

#### Response - No Confirmed Subscribers (200 OK)
```json
{
  "message": "No confirmed subscribers found",
  "sent_count": 0
}
```

#### Response - Validation Error (400 Bad Request)
Same format as `/newsletters/send-all`

#### Example Usage (cURL)
```bash
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Exclusive Newsletter",
    "html_content": "<h1>Exclusive Content</h1><p>Only for confirmed subscribers</p>"
  }'
```

#### Example Usage (Python)
```python
import requests

data = {
    "subject": "Exclusive Newsletter",
    "html_content": "<h1>Exclusive Content</h1><p>Only for confirmed subscribers</p>"
}

response = requests.post(
    'http://localhost:8000/newsletters/send-confirmed',
    json=data
)

result = response.json()
print(f"Sent: {result['sent_count']}, Failed: {result['failed_count']}")
```

## Validation System

### Data Validation for Stored Subscribers

The newsletter system automatically validates all subscriber data before sending emails. This ensures data integrity and prevents emails from being sent to corrupted or invalid records.

#### Validation Checks

**1. UUID Format Validation**
- Validates that subscriber ID is a valid UUID v4
- Format: `8-4-4-4-12` hexadecimal characters
- Example: `550e8400-e29b-41d4-a716-446655440000`

**2. Email Validation**
- Checks RFC 5322 simplified format
- Length: 5-254 characters (RFC 5321)
- Detects phishing patterns
- Prevents SQL injection patterns
- Examples of invalid emails:
  - `invalid` (format error)
  - `user@@domain.com` (multiple @ symbols)
  - `user' OR '1'='1@domain.com` (SQL injection attempt)

**3. Name Validation**
- Length: 1-256 characters
- No null bytes or control characters
- Prevents excessive special characters
- Examples of invalid names:
  - Empty string
  - `Name\0with\0null` (null bytes)
  - `John!!!!!!` (if excessive special chars)

**4. Status Validation**
- Only allows: `pending` or `confirmed`
- Case-sensitive
- Examples:
  - Valid: `confirmed`, `pending`
  - Invalid: `active`, `verified`, `Confirmed`

#### Validation Error Handling

When subscriber data fails validation:

1. The subscriber is skipped and counted in `failed_count`
2. A warning is logged with validation error details
3. An audit log entry is created for tracking
4. Newsletter sending continues with remaining valid subscribers

#### Example Validation Scenario

```
Processing 152 subscribers...

Subscriber 1: PASS validation
- Email: user1@example.com
- Name: John Doe
- Status: confirmed
→ Email sent successfully

Subscriber 2: FAIL validation
- ID: invalid-uuid (Invalid UUID format)
- Email: user2@example.com
- Name: Jane Smith
- Status: confirmed
→ Skipped, counted in failed_count
→ Warning logged

Subscriber 3: PASS validation
- Email: user3@example.com
- Name: Bob Johnson
- Status: pending
→ Email sent successfully

Result: sent_count = 151, failed_count = 1
```

## Data Validation Module

### Module Location
`src/data_validation.rs`

### Public Functions

#### `validate_subscriber_data(id, email, name, status)`
Validates a complete subscriber record.

```rust
pub fn validate_subscriber_data(
    id: &str,
    email: &str,
    name: &str,
    status: &str,
) -> Result<(), ValidationError>
```

#### `validate_uuid(id)`
Validates UUID format.

```rust
pub fn validate_uuid(id: &str) -> Result<(), ValidationError>
```

#### `validate_stored_name(name)`
Validates subscriber name.

```rust
pub fn validate_stored_name(name: &str) -> Result<(), ValidationError>
```

#### `validate_subscription_status(status)`
Validates subscription status.

```rust
pub fn validate_subscription_status(status: &str) -> Result<(), ValidationError>
```

#### `validate_subscribers_batch(subscribers)`
Batch validates multiple subscriber records.

```rust
pub fn validate_subscribers_batch(
    subscribers: &[(String, String, String, String)],
) -> Result<(), (usize, ValidationError)>
```

Returns error with index of first invalid subscriber.

## Implementation Architecture

### Newsletter Route Handler
Location: `src/routes/newsletters.rs`

#### Data Flow

```
HTTP Request
    ↓
Validate Newsletter Data (subject, html_content)
    ↓
Fetch Subscribers from Database
    ↓
For Each Subscriber:
  1. Validate subscriber data
  2. Send email via EmailClient
  3. Log result (success/failure)
  ↓
Return Summary Response
```

#### Key Components

**NewsletterData Struct**
```rust
pub struct NewsletterData {
    subject: Option<String>,
    html_content: Option<String>,
}
```

**SubscriberData Struct**
```rust
pub struct SubscriberData {
    pub id: String,
    pub email: String,
    pub name: String,
    pub status: String,
}
```

### Error Handling

The newsletter system uses comprehensive error handling:

1. **Validation Errors** (400 Bad Request)
   - Missing or invalid subject/content
   - Logged with audit trail

2. **Database Errors** (500 Internal Server Error)
   - Failed to fetch subscribers
   - Logged with retry flag if applicable

3. **Email Service Errors** (503 Service Unavailable)
   - Logged per-subscriber
   - Continues sending to remaining subscribers

### Audit Logging

All newsletter operations are logged to the audit trail:

```
Action: SEND_NEWSLETTER
Resource Type: newsletter
Status: SUCCESS/FAILURE
Details: Newsletter sent to subscriber / Failed to send to email@example.com
Timestamp: 2024-11-23T10:30:00Z
```

### Database Queries

**Fetch All Subscribers**
```sql
SELECT id, email, name, status FROM subscriptions
```

**Fetch Confirmed Subscribers**
```sql
SELECT id, email, name, status FROM subscriptions WHERE status = 'confirmed'
```

## Security Considerations

### Input Validation
- Subject and HTML content are validated for emptiness
- Prevents injection attacks through email content

### Data Validation
- Subscriber data is validated before sending
- Prevents emails from being sent to corrupted records
- Detects potential SQL injection patterns in stored data

### Rate Limiting
- Applies to newsletter endpoints
- 10 requests per minute per IP (configurable)
- Prevents abuse of newsletter sending

### Audit Trail
- All newsletter operations logged
- Can be used for compliance and debugging
- Tracks success/failure of each send attempt

## Best Practices

### 1. Use Confirmed Subscribers for Important Communications
```bash
# Preferred
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{"subject": "...", "html_content": "..."}'

# Only if necessary
curl -X POST http://localhost:8000/newsletters/send-all \
  -H "Content-Type: application/json" \
  -d '{"subject": "...", "html_content": "..."}'
```

### 2. Monitor Failure Count
```javascript
if (result.failed_count > 0) {
  console.warn(`${result.failed_count} emails failed to send`);
  // Check logs for details
}
```

### 3. Include Unsubscribe Links in HTML
```html
<p>
  If you wish to unsubscribe, please reply with "UNSUBSCRIBE"
  or contact our support team.
</p>
```

### 4. Test with Confirmed Subscribers First
```bash
# Test endpoint
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{"subject": "Test Newsletter", "html_content": "<h1>Test</h1>"}'
```

## Testing

### Unit Tests
Location: `src/data_validation.rs`

Tests cover:
- Valid subscriber data
- Invalid UUID formats
- Invalid email addresses
- Invalid subscription statuses
- Batch validation
- Edge cases (empty fields, null bytes)

### Integration Testing

**Test sending to confirmed subscribers:**
```bash
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Test Newsletter",
    "html_content": "<h1>Test</h1><p>This is a test</p>"
  }'
```

**Expected Response:**
```json
{
  "message": "Newsletter sent to confirmed subscribers",
  "sent_count": N,
  "failed_count": 0
}
```

**Test validation error handling:**
```bash
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{"subject": ""}'
```

**Expected Response:**
```json
{
  "error_id": "...",
  "message": "Missing required field: html_content",
  "code": "VALIDATION_ERROR",
  "status": 400,
  "timestamp": "..."
}
```

## Troubleshooting

### Issue: Newsletter not sent to any subscribers
1. Check if subscribers table has records: `SELECT COUNT(*) FROM subscriptions`
2. Check email service is running and accessible
3. Check logs for database or email service errors
4. Verify subscriber data is valid with: `SELECT * FROM subscriptions LIMIT 1`

### Issue: Some newsletters fail to send
1. Check audit logs for which subscribers failed
2. Verify email service is healthy
3. Check if subscriber data is corrupted (run validation)
4. Retry sending with `/newsletters/send-confirmed` to skip pending subscribers

### Issue: Database query errors
1. Verify database connection string in configuration
2. Check database migrations have run: `\dt` in PostgreSQL
3. Verify subscriptions table has data
4. Check database logs for errors

## Related Documentation

- [Email Confirmation Service](EMAIL_CONFIRMATION_SERVICE.md)
- [Error Handling](ERROR_HANDLING.md)
- [Request Failure Logging](REQUEST_FAILURE_LOGGING.md)
- [Database Integration](DATABASE_INTEGRATION_EXAMPLE.md)
