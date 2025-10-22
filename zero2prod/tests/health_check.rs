//! Integration tests for the zero2prod server

use std::net::TcpListener;
use zero2prod::startup;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = startup(listener)
        .expect("Failed to create server");

    let _ = tokio::spawn(async move {
        let _ = server.await;
    });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();

    let response = reqwest::Client::new()
        .get(&format!("{}/health_check", addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn greet_returns_name() {
    let addr = spawn_app();

    let response = reqwest::Client::new()
        .get(&format!("{}/Alice", addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "Hello Alice!");
}

#[tokio::test]
async fn greet_default_world() {
    let addr = spawn_app();

    let response = reqwest::Client::new()
        .get(&addr)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "Hello World!");
}