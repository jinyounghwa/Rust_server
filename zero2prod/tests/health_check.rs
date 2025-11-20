use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, Executor, Connection, PgConnection, Row};

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

    let server = run(listener, connection_pool.clone())
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

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(
        saved.get::<String, _>("email"),
        "ursula_le_guin@gmail.com"
    );
    assert_eq!(saved.get::<String, _>("name"), "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_rejects_email_exceeding_256_chars() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Email with more than 256 characters
    let long_email = format!("{}@example.com", "a".repeat(250));
    let body = format!("name=Test&email={}", urlencoding::encode(&long_email));

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16(), "Should reject email exceeding 256 characters");
}

#[tokio::test]
async fn subscribe_rejects_name_exceeding_256_chars() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Name with 257 characters
    let long_name = "a".repeat(257);
    let body = format!("name={}&email=test@example.com", urlencoding::encode(&long_name));

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16(), "Should reject name exceeding 256 characters");
}

#[tokio::test]
async fn subscribe_rejects_sql_injection_in_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let malicious_emails = vec![
        "user' UNION SELECT * FROM subscriptions--@example.com",
        "user'; DROP TABLE subscriptions;--@example.com",
        "user@example.com' OR '1'='1",
    ];

    for malicious_email in malicious_emails {
        let body = format!("name=Test&email={}", urlencoding::encode(malicious_email));

        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject SQL injection attempt: {}", malicious_email);
    }
}

#[tokio::test]
async fn subscribe_rejects_sql_injection_in_name() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let malicious_names = vec![
        "Test'; DROP TABLE subscriptions;--",
        "Test UNION SELECT * FROM subscriptions",
        "Test' OR '1'='1",
    ];

    for malicious_name in malicious_names {
        let body = format!("name={}&email=test@example.com", urlencoding::encode(malicious_name));

        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(),
            "Should reject SQL injection attempt: {}", malicious_name);
    }
}

#[tokio::test]
async fn subscribe_rejects_invalid_email_format() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let invalid_emails = vec![
        "notanemail",
        "user@",
        "@example.com",
        "user@@example.com",
        "user@.com",
    ];

    for invalid_email in invalid_emails {
        let body = format!("name=Test&email={}", invalid_email);

        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "Should reject invalid email: {}", invalid_email);
    }
}

#[tokio::test]
async fn subscribe_rejects_duplicate_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=Test&email=test@example.com";

    // First subscription should succeed
    let response1 = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response1.status().as_u16());

    // Duplicate subscription should return 409 Conflict
    let response2 = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(409, response2.status().as_u16(), "Should reject duplicate email with 409 Conflict");
}

#[tokio::test]
async fn subscribe_rejects_control_characters_in_name() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Name with null byte
    let body = "name=Test%00Name&email=test@example.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16(), "Should reject name with control characters");
}
