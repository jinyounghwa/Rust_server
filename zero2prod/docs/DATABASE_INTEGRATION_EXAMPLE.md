# 데이터베이스 통합 실전 예제

이 문서는 Rust 프로젝트에서 PostgreSQL 데이터베이스와 실제로 상호작용하는 방법을 단계별로 설명합니다.

---

## 목차

1. [기본 설정](#기본-설정)
2. [데이터 구조 정의](#데이터-구조-정의)
3. [데이터 삽입](#데이터-삽입)
4. [데이터 조회](#데이터-조회)
5. [데이터 업데이트](#데이터-업데이트)
6. [데이터 삭제](#데이터-삭제)
7. [통합 예제: 구독자 관리 시스템](#통합-예제-구독자-관리-시스템)

---

## 기본 설정

### 1. Cargo.toml 의존성 추가

필요한 라이브러리를 추가합니다:

```toml
[dependencies]
actix-web = "4"
tokio = {version = "1", features = ["macros", "rt-multi-thread"]}
serde = {version = "1", features = ["derive"]}
sqlx = {version = "0.6", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"]}
uuid = {version = "1", features = ["v4", "serde"]}
chrono = {version = "0.4", features = ["serde"]}
```

### 2. 환경 변수 설정

`.env` 파일:

```
DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter
```

### 3. 기본 데이터베이스 연결 함수

`src/lib.rs` 또는 `src/database.rs`에 추가:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
```

---

## 데이터 구조 정의

### Subscriber 구조체

`src/models.rs` 파일 생성:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub subscribed_at: DateTime<Utc>,
}

// 데이터베이스에 저장할 구독자 정보
#[derive(Debug, Deserialize)]
pub struct NewSubscriber {
    pub email: String,
    pub name: String,
}

// 구독자 응답 DTO (API 응답용)
#[derive(Debug, Serialize)]
pub struct SubscriberResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub subscribed_at: String,
}

impl From<Subscriber> for SubscriberResponse {
    fn from(sub: Subscriber) -> Self {
        SubscriberResponse {
            id: sub.id.to_string(),
            email: sub.email,
            name: sub.name,
            subscribed_at: sub.subscribed_at.to_rfc3339(),
        }
    }
}
```

---

## 데이터 삽입

### 새 구독자 추가 (INSERT)

`src/repository.rs` 파일 생성:

```rust
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{Subscriber, NewSubscriber};

pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: NewSubscriber,
) -> Result<Subscriber, sqlx::Error> {
    let id = Uuid::new_v4();
    let subscribed_at = Utc::now();

    let subscriber = sqlx::query_as::<_, Subscriber>(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, name, subscribed_at
        "#
    )
    .bind(id)
    .bind(&new_subscriber.email)
    .bind(&new_subscriber.name)
    .bind(subscribed_at)
    .fetch_one(pool)
    .await?;

    Ok(subscriber)
}
```

### 사용 예제

```rust
#[actix_web::post("/subscriptions")]
async fn subscribe(
    pool: web::Data<PgPool>,
    form: web::Form<NewSubscriber>,
) -> impl Responder {
    match insert_subscriber(pool.get_ref(), form.into_inner()).await {
        Ok(subscriber) => {
            HttpResponse::Created().json(SubscriberResponse::from(subscriber))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## 데이터 조회

### 모든 구독자 조회 (SELECT *)

```rust
pub async fn get_all_subscribers(pool: &PgPool) -> Result<Vec<Subscriber>, sqlx::Error> {
    sqlx::query_as::<_, Subscriber>(
        "SELECT id, email, name, subscribed_at FROM subscriptions ORDER BY subscribed_at DESC"
    )
    .fetch_all(pool)
    .await
}
```

### ID로 구독자 조회

```rust
pub async fn get_subscriber_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Subscriber>, sqlx::Error> {
    sqlx::query_as::<_, Subscriber>(
        "SELECT id, email, name, subscribed_at FROM subscriptions WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
```

### 이메일로 구독자 조회

```rust
pub async fn get_subscriber_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<Subscriber>, sqlx::Error> {
    sqlx::query_as::<_, Subscriber>(
        "SELECT id, email, name, subscribed_at FROM subscriptions WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}
```

### 페이지네이션을 포함한 조회

```rust
pub async fn get_subscribers_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Subscriber>, sqlx::Error> {
    sqlx::query_as::<_, Subscriber>(
        "SELECT id, email, name, subscribed_at FROM subscriptions ORDER BY subscribed_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
```

### API 엔드포인트 예제

```rust
#[actix_web::get("/subscriptions")]
async fn list_subscribers(pool: web::Data<PgPool>) -> impl Responder {
    match get_all_subscribers(pool.get_ref()).await {
        Ok(subscribers) => {
            let response: Vec<SubscriberResponse> = subscribers
                .into_iter()
                .map(|s| s.into())
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::get("/subscriptions/{id}")]
async fn get_subscriber(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match get_subscriber_by_id(pool.get_ref(), id.into_inner()).await {
        Ok(Some(subscriber)) => {
            HttpResponse::Ok().json(SubscriberResponse::from(subscriber))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## 데이터 업데이트

### 구독자 정보 업데이트 (UPDATE)

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateSubscriber {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn update_subscriber(
    pool: &PgPool,
    id: Uuid,
    update: UpdateSubscriber,
) -> Result<Option<Subscriber>, sqlx::Error> {
    if let Some(name) = update.name {
        sqlx::query("UPDATE subscriptions SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(email) = update.email {
        sqlx::query("UPDATE subscriptions SET email = $1 WHERE id = $2")
            .bind(email)
            .bind(id)
            .execute(pool)
            .await?;
    }

    // 업데이트된 구독자 반환
    get_subscriber_by_id(pool, id).await
}
```

### API 엔드포인트

```rust
#[actix_web::put("/subscriptions/{id}")]
async fn update_subscriber_endpoint(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    update: web::Json<UpdateSubscriber>,
) -> impl Responder {
    match update_subscriber(pool.get_ref(), id.into_inner(), update.into_inner()).await {
        Ok(Some(subscriber)) => {
            HttpResponse::Ok().json(SubscriberResponse::from(subscriber))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## 데이터 삭제

### 구독자 삭제 (DELETE)

```rust
pub async fn delete_subscriber(
    pool: &PgPool,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM subscriptions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
```

### API 엔드포인트

```rust
#[actix_web::delete("/subscriptions/{id}")]
async fn delete_subscriber_endpoint(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match delete_subscriber(pool.get_ref(), id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## 통합 예제: 구독자 관리 시스템

### 전체 애플리케이션 구조

```
src/
├── main.rs              # 애플리케이션 진입점
├── lib.rs               # 핵심 비즈니스 로직
├── models.rs            # 데이터 구조
├── repository.rs        # 데이터베이스 작업
└── handlers.rs          # HTTP 핸들러
```

### main.rs

```rust
mod models;
mod repository;
mod handlers;

use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 데이터베이스 연결
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    println!("✓ Database connected successfully");

    // HTTP 서버 시작
    println!("🚀 Starting server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health_check", web::get().to(handlers::health_check))
            .route("/subscriptions", web::get().to(handlers::list_subscribers))
            .route("/subscriptions", web::post().to(handlers::subscribe))
            .route("/subscriptions/{id}", web::get().to(handlers::get_subscriber))
            .route("/subscriptions/{id}", web::put().to(handlers::update_subscriber_endpoint))
            .route("/subscriptions/{id}", web::delete().to(handlers::delete_subscriber_endpoint))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### handlers.rs

```rust
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{NewSubscriber, UpdateSubscriber, SubscriberResponse};
use crate::repository::{
    insert_subscriber, get_all_subscribers, get_subscriber_by_id,
    update_subscriber, delete_subscriber,
};

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

pub async fn subscribe(
    pool: web::Data<PgPool>,
    form: web::Form<NewSubscriber>,
) -> impl Responder {
    match insert_subscriber(pool.get_ref(), form.into_inner()).await {
        Ok(subscriber) => {
            HttpResponse::Created().json(SubscriberResponse::from(subscriber))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create subscription"
            }))
        }
    }
}

pub async fn list_subscribers(pool: web::Data<PgPool>) -> impl Responder {
    match get_all_subscribers(pool.get_ref()).await {
        Ok(subscribers) => {
            let response: Vec<SubscriberResponse> = subscribers
                .into_iter()
                .map(|s| s.into())
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_subscriber(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match get_subscriber_by_id(pool.get_ref(), id.into_inner()).await {
        Ok(Some(subscriber)) => {
            HttpResponse::Ok().json(SubscriberResponse::from(subscriber))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_subscriber_endpoint(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    update: web::Json<UpdateSubscriber>,
) -> impl Responder {
    match update_subscriber(pool.get_ref(), id.into_inner(), update.into_inner()).await {
        Ok(Some(subscriber)) => {
            HttpResponse::Ok().json(SubscriberResponse::from(subscriber))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_subscriber_endpoint(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match delete_subscriber(pool.get_ref(), id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
```

---

## API 테스트

### cURL을 사용한 테스트

```bash
# 1. 헬스 체크
curl http://localhost:8080/health_check

# 2. 새 구독자 추가
curl -X POST http://localhost:8080/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=John Doe&email=john@example.com"

# 3. 모든 구독자 조회
curl http://localhost:8080/subscriptions

# 4. 특정 구독자 조회 (ID 값으로 교체)
curl http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000

# 5. 구독자 정보 업데이트
curl -X PUT http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name":"Jane Doe"}'

# 6. 구독자 삭제
curl -X DELETE http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000
```

---

## 실행 방법

### 1. 데이터베이스 시작

```bash
docker run -d --name zero2prod-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=newsletter \
  -p 5432:5432 \
  postgres:latest
```

### 2. 마이그레이션 실행

```bash
docker exec zero2prod-db psql -U postgres -d newsletter -c "CREATE TABLE subscriptions(id uuid NOT NULL, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, subscribed_at timestamptz NOT NULL, PRIMARY KEY (id));"
```

### 3. 애플리케이션 실행

```bash
cargo run
```

### 4. API 테스트

```bash
curl http://localhost:8080/health_check
```

---

## 오류 처리 Best Practices

```rust
use serde_json::json;

// 구체적인 오류 메시지 반환
pub async fn create_subscription(
    pool: web::Data<PgPool>,
    form: web::Form<NewSubscriber>,
) -> impl Responder {
    // 입력 검증
    if form.email.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Email is required"}));
    }

    if form.name.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Name is required"}));
    }

    // 데이터베이스 작업
    match insert_subscriber(pool.get_ref(), form.into_inner()).await {
        Ok(subscriber) => {
            HttpResponse::Created()
                .json(SubscriberResponse::from(subscriber))
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound()
                .json(json!({"error": "Subscriber not found"}))
        }
        Err(e) => {
            // 고유 제약 조건 위반 (이메일 중복)
            if e.to_string().contains("unique") {
                return HttpResponse::Conflict()
                    .json(json!({"error": "Email already exists"}));
            }

            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": "Internal server error"}))
        }
    }
}
```

---

**마지막 업데이트**: 2025년 11월 5일
