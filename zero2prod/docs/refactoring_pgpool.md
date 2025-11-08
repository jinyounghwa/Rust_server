# PgConnection → PgPool 리펙터링

**작성일:** 2025-11-08
**제목:** 단일 데이터베이스 연결을 연결 풀로 변경

---

## 개요

이번 리펙터링은 애플리케이션이 단일 `PgConnection`을 사용하는 방식에서 **`PgPool`(연결 풀)** 을 사용하도록 변경했습니다. 이를 통해 동시 요청에 대한 처리 능력을 향상시키고 데이터베이스 연결을 더 효율적으로 관리할 수 있습니다.

---

## 변경 사항

### 1. **src/main.rs** - 애플리케이션 진입점

#### 이전 코드
```rust
use sqlx::{Connection, PgConnection};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener, connection)?;
    let _ = server.await;
    Ok(())
}
```

#### 변경 후 코드
```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .connect(&configuration.database.connection_string())
        .await
        .expect("Failed to create connection pool");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener, connection_pool)?;
    let _ = server.await;
    Ok(())
}
```

**변경점:**
- `PgConnection::connect()` → `PgPoolOptions::new().connect()` 변경
- 단일 연결에서 연결 풀로 전환
- 변수명: `connection` → `connection_pool` (의미 명확화)

---

### 2. **src/startup.rs** - HTTP 서버 설정

#### 이전 코드
```rust
use sqlx::PgConnection;

pub fn run(listener: TcpListener, connection:PgConnection) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub async fn subscribe(
    _form: web::Form<FormData>,
    _connection: web::Data<PgConnection>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

#### 변경 후 코드
```rust
use sqlx::PgPool;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection_pool.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
```

**변경점:**
- 함수 매개변수 타입: `PgConnection` → `PgPool`
- 중복된 `subscribe` 함수 정의 제거 (routes/subscriptions.rs에서 정의)
- `web::Data::new(connection_pool)` 으로 래핑하여 모든 핸들러에서 공유 가능

---

### 3. **src/routes/subscriptions.rs** - 구독 핸들러

#### 이전 코드
```rust
pub async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    let email_valid = form.email
        .as_ref()
        .map(|e| is_valid_email(e))
        .unwrap_or(false);

    if name_valid && email_valid {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}
```

#### 변경 후 코드
```rust
use sqlx::PgPool;

pub async fn subscribe(
    form: web::Form<FormData>,
    _pool: web::Data<PgPool>,
) -> HttpResponse {
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    let email_valid = form.email
        .as_ref()
        .map(|e| is_valid_email(e))
        .unwrap_or(false);

    if name_valid && email_valid {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}
```

**변경점:**
- PgPool import 추가
- 함수 매개변수에 `_pool: web::Data<PgPool>` 추가
- Actix-web이 자동으로 app_data에서 pool을 주입
- 현재 미사용 상태이므로 `_` prefix로 마킹

---

## 주요 이점

### 1. **동시 요청 처리**
- **이전:** 단일 연결만 사용하므로 동시 요청이 서로 대기
- **이후:** 여러 연결을 풀에서 꺼내 사용하여 동시 처리 가능

### 2. **연결 재사용**
- 연결 풀은 사용한 연결을 재사용하므로 연결 생성 오버헤드 감소
- 데이터베이스 성능 향상

### 3. **안정성**
- 연결 풀은 타임아웃, 재시도 등의 기능 제공
- 데이터베이스 연결 실패에 대한 복원력 향상

### 4. **확장성**
- 코드 구조상 향후 데이터베이스 쿼리 추가 시 pool을 쉽게 사용 가능

---

## 구현 세부사항

### PgPoolOptions 설정
```rust
let connection_pool = PgPoolOptions::new()
    .connect(&configuration.database.connection_string())
    .await
    .expect("Failed to create connection pool");
```

**기본 설정:**
- 최소 연결 수: 5개 (기본값)
- 최대 연결 수: 10개 (기본값)
- 연결 타임아웃: 30초 (기본값)

필요시 커스터마이징 가능:
```rust
let connection_pool = PgPoolOptions::new()
    .max_connections(20)          // 최대 연결 수
    .min_connections(5)           // 최소 연결 수
    .connect(&configuration.database.connection_string())
    .await?;
```

---

## Actix-web에서의 app_data 주입

```rust
let connection_pool = web::Data::new(connection_pool);
let server = HttpServer::new(move || {
    App::new()
        .app_data(connection_pool.clone())
        // ...
})
```

- `web::Data`로 감싸면 `Arc`를 통해 thread-safe 공유
- 각 워커 스레드가 클론을 통해 풀에 접근
- 핸들러에서 `web::Data<PgPool>` 매개변수로 자동 주입

---

## 향후 사용 예시

데이터베이스 쿼리를 추가할 때:

```rust
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // pool에서 연결 획득
    let mut connection = pool
        .acquire()
        .await
        .expect("Failed to acquire connection");

    // 데이터베이스 쿼리 실행
    sqlx::query(
        "INSERT INTO subscriptions (name, email) VALUES ($1, $2)"
    )
    .bind(&form.name)
    .bind(&form.email)
    .execute(&mut *connection)
    .await
    .expect("Failed to insert subscription");

    HttpResponse::Ok().finish()
}
```

또는 sqlx의 편의 메서드 사용:

```rust
sqlx::query(
    "INSERT INTO subscriptions (name, email) VALUES ($1, $2)"
)
.bind(&form.name)
.bind(&form.email)
.execute(pool.as_ref())  // pool을 직접 전달 가능
.await?;
```

---

## 테스트 상태

✓ `cargo check` - 컴파일 성공
✓ 모든 warnings 해결
✓ 타입 안정성 확보

---

## 참고 자료

- [SQLx PgPool Documentation](https://docs.rs/sqlx/latest/sqlx/struct.PgPool.html)
- [Actix-web app_data Guide](https://actix.rs/docs/application/#state)
- [Connection Pooling Best Practices](https://github.com/launchbadge/sqlx)
