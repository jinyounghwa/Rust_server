use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, Executor, Connection, PgConnection, Row};
use serde_json::{json, Value};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let jwt_config = configuration.jwt.clone();
    let server = run(listener, connection_pool.clone(), jwt_config)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");
    connection_pool
}

// --- Registration Tests ---

#[tokio::test]
async fn register_returns_200_for_valid_credentials() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    let response = client
        .post(&format!("{}/auth/register", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(response_body.get("access_token").is_some());
    assert!(response_body.get("refresh_token").is_some());

    // Verify user was created in database
    let user = sqlx::query("SELECT email, name FROM users WHERE email = 'john@example.com'")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch created user");

    assert_eq!(user.get::<String, _>("email"), "john@example.com");
    assert_eq!(user.get::<String, _>("name"), "John Doe");
}

#[tokio::test]
async fn register_returns_400_for_invalid_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let invalid_emails = vec![
        "notanemail",
        "user@",
        "@example.com",
        "user@@example.com",
    ];

    for invalid_email in invalid_emails {
        let body = json!({
            "name": "Test User",
            "email": invalid_email,
            "password": "SecurePass123"
        });

        let response = client
            .post(&format!("{}/auth/register", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject invalid email: {}", invalid_email);
    }
}

#[tokio::test]
async fn register_returns_400_for_weak_password() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let long_password = "a".repeat(129);
    let weak_passwords = vec![
        ("short", "password too short"),
        ("nouppercase123", "no uppercase"),
        ("NOLOWERCASE123", "no lowercase"),
        ("NoDigits", "no digits"),
        (long_password.as_str(), "password too long"),
    ];

    for (weak_password, reason) in weak_passwords {
        let body = json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": weak_password
        });

        let response = client
            .post(&format!("{}/auth/register", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject weak password: {}", reason);
    }
}

#[tokio::test]
async fn register_returns_409_for_duplicate_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    // First registration should succeed
    let response1 = client
        .post(&format!("{}/auth/register", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response1.status().as_u16());

    // Duplicate registration should fail with 409
    let response2 = client
        .post(&format!("{}/auth/register", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(409, response2.status().as_u16(),
        "Should reject duplicate email with 409 Conflict");
}

#[tokio::test]
async fn register_returns_400_for_missing_fields() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (json!({"email": "test@example.com", "password": "Pass123"}), "missing name"),
        (json!({"name": "Test", "password": "Pass123"}), "missing email"),
        (json!({"name": "Test", "email": "test@example.com"}), "missing password"),
        (json!({}), "missing all fields"),
    ];

    for (body, reason) in test_cases {
        let response = client
            .post(&format!("{}/auth/register", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject request: {}", reason);
    }
}

// --- Login Tests ---

#[tokio::test]
async fn login_returns_200_for_valid_credentials() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // First register a user
    let register_body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Now login with the same credentials
    let login_body = json!({
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    let response = client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(response_body.get("access_token").is_some());
    assert!(response_body.get("refresh_token").is_some());
}

#[tokio::test]
async fn login_returns_400_for_invalid_password() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // First register a user
    let register_body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Try to login with wrong password
    let login_body = json!({
        "email": "john@example.com",
        "password": "WrongPassword123"
    });

    let response = client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn login_returns_400_for_nonexistent_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let login_body = json!({
        "email": "nonexistent@example.com",
        "password": "SecurePass123"
    });

    let response = client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn login_returns_400_for_missing_fields() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (json!({"email": "test@example.com"}), "missing password"),
        (json!({"password": "Pass123"}), "missing email"),
        (json!({}), "missing all fields"),
    ];

    for (body, reason) in test_cases {
        let response = client
            .post(&format!("{}/auth/login", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject request: {}", reason);
    }
}

// --- Protected Routes Tests ---

#[tokio::test]
async fn protected_route_returns_401_without_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/auth/me", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn protected_route_returns_401_with_invalid_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/auth/me", &app.address))
        .header("Authorization", "Bearer invalid.token.here")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["code"], "TOKEN_INVALID");
}

#[tokio::test]
async fn get_current_user_returns_200_with_valid_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Register and get token
    let register_body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    let register_response = client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let register_data: Value = register_response.json().await.expect("Failed to parse response");
    let access_token = register_data["access_token"]
        .as_str()
        .expect("No access token in response");

    // Use token to get current user
    let response = client
        .get(&format!("{}/auth/me", &app.address))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["email"], "john@example.com");
    assert_eq!(response_body["name"], "John Doe");
}

#[tokio::test]
async fn protected_route_rejects_malformed_authorization_header() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let malformed_headers = vec![
        "Bearer",  // missing token
        "Basic dXNlcjpwYXNz",  // not Bearer
        "BearerToken",  // missing space
        "",  // empty
    ];

    for header in malformed_headers {
        let response = client
            .get(&format!("{}/auth/me", &app.address))
            .header("Authorization", header)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(401, response.status().as_u16(),
            "Should reject malformed header: {}", header);
    }
}

// --- Token Refresh Tests ---

#[tokio::test]
async fn refresh_returns_200_with_valid_refresh_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Register user
    let register_body = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "password": "SecurePass123"
    });

    let register_response = client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let register_data: Value = register_response.json().await.expect("Failed to parse response");
    let old_refresh_token = register_data["refresh_token"]
        .as_str()
        .expect("No refresh token in response");

    // Refresh the token
    let refresh_body = json!({
        "refresh_token": old_refresh_token
    });

    let response = client
        .post(&format!("{}/auth/refresh", &app.address))
        .json(&refresh_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(response_body.get("access_token").is_some());
    assert!(response_body.get("refresh_token").is_some());

    let new_refresh_token = response_body["refresh_token"]
        .as_str()
        .expect("No new refresh token");

    // Verify tokens are different (token rotation)
    assert_ne!(old_refresh_token, new_refresh_token,
        "Refresh token should be rotated on each refresh");
}

#[tokio::test]
async fn refresh_returns_400_with_invalid_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let refresh_body = json!({
        "refresh_token": "definitely_not_a_valid_token_in_database"
    });

    let response = client
        .post(&format!("{}/auth/refresh", &app.address))
        .json(&refresh_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn refresh_returns_400_for_missing_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let refresh_body = json!({});

    let response = client
        .post(&format!("{}/auth/refresh", &app.address))
        .json(&refresh_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

// --- Protected Route Access Tests ---

#[tokio::test]
async fn all_protected_endpoints_require_auth() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let protected_paths = vec![
        "/auth/me",
        "/subscriptions",
        "/subscriptions/confirm",
        "/newsletters/send-all",
        "/newsletters/send-confirmed",
    ];

    for path in protected_paths {
        let response = client
            .get(&format!("{}{}", &app.address, path))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(401, response.status().as_u16(),
            "Endpoint {} should require authentication", path);
    }
}
