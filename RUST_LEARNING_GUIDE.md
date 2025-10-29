# Rust 초보자를 위한 프로젝트 가이드

이 문서는 Rust를 처음 배우는 분들을 위해 zero2prod 프로젝트의 각 파일이 어떻게 연결되고 동작하는지 설명합니다.

## 목차
1. [프로젝트 개요](#프로젝트-개요)
2. [Cargo.toml - 프로젝트 설정 파일](#cargotoml---프로젝트-설정-파일)
3. [src/main.rs - 프로그램의 시작점](#srcmainrs---프로그램의-시작점)
4. [src/lib.rs - 핵심 비즈니스 로직](#srclibrs---핵심-비즈니스-로직)
5. [tests/health_check.rs - 통합 테스트](#testshealth_checkrs---통합-테스트)
6. [파일 간 연결 관계](#파일-간-연결-관계)
7. [실행 흐름](#실행-흐름)

---

## 프로젝트 개요

이 프로젝트는 **actix-web**을 사용한 간단한 웹 서버입니다. 주요 기능은:
- 건강 체크 엔드포인트 (`/health_check`)
- 인사말 엔드포인트 (`/`, `/{name}`)
- 구독 폼 처리 (`/subscriptions`)

---

## Cargo.toml - 프로젝트 설정 파일

**위치**: `zero2prod/Cargo.toml`

### 역할
Cargo.toml은 Rust 프로젝트의 "설정 파일"입니다. 프로젝트 정보와 필요한 외부 라이브러리(의존성)를 정의합니다.

### 주요 내용

```toml
[package]
name = "zero2prod"
version = "0.1.0"
edition = "2021"
```

- **name**: 프로젝트 이름
- **version**: 프로젝트 버전
- **edition**: 사용할 Rust 에디션 (2021은 최신 문법을 사용)

```toml
[dependencies]
actix-web = "4"
tokio = {version = "1", features = ["macros", "rt-multi-thread"]}
serde = {version = "1", features = ["derive"]}
```

- **actix-web**: 웹 서버 프레임워크 (Node.js의 Express 같은 역할)
- **tokio**: 비동기 런타임 (async/await를 실행하는 엔진)
- **serde**: 데이터 직렬화/역직렬화 라이브러리 (JSON 처리 등)

```toml
[dev-dependencies]
reqwest = {version = "0.11", features = ["json"]}
```

- **reqwest**: HTTP 클라이언트 (테스트에서 서버에 요청을 보낼 때 사용)
- **dev-dependencies**: 개발 및 테스트할 때만 필요한 의존성

### 초보자를 위한 팁
> Cargo.toml은 Node.js의 `package.json`, Python의 `requirements.txt`와 비슷한 역할을 합니다.

---

## src/main.rs - 프로그램의 시작점

**위치**: `zero2prod/src/main.rs`

### 역할
모든 Rust 프로그램은 `main` 함수에서 시작됩니다. 이 파일은 프로그램의 **진입점(entry point)**입니다.

### 코드 분석

```rust
use zero2prod::run;
```
- `lib.rs`에서 정의한 `run` 함수를 가져옵니다 (import)

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
```
- `#[tokio::main]`: tokio 비동기 런타임을 시작하는 매크로
- `async fn`: 비동기 함수 선언 (JavaScript의 `async function`과 유사)
- `-> std::io::Result<()>`: 함수가 성공하거나 에러를 반환할 수 있음

```rust
    let server = run().await?;
    server.await
```
- `run()`: lib.rs의 run 함수를 호출하여 서버를 생성
- `.await`: 비동기 작업이 완료될 때까지 대기 (JavaScript의 `await`와 동일)
- `?`: 에러가 발생하면 즉시 반환 (에러 처리 간소화)

### 초보자를 위한 팁
> `main.rs`는 매우 간단합니다. 실제 로직은 `lib.rs`에 있고, `main.rs`는 단지 프로그램을 시작하는 역할만 합니다. 이렇게 분리하면 테스트하기 쉽고 코드를 재사용하기 좋습니다.

---

## src/lib.rs - 핵심 비즈니스 로직

**위치**: `zero2prod/src/lib.rs`

### 역할
웹 서버의 핵심 로직이 모두 들어있습니다. 라우트(경로) 설정, 요청 처리 함수, 서버 시작 등을 담당합니다.

### 1. Import 선언

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::net::TcpListener;
use actix_web::dev::Server;
```

필요한 모듈과 타입들을 가져옵니다.

### 2. 데이터 구조 정의

```rust
#[derive(Deserialize)]
struct FormData {
    name: Option<String>,
    email: Option<String>,
}
```

- `struct`: 구조체 (데이터를 담는 컨테이너)
- `#[derive(Deserialize)]`: 자동으로 역직렬화 기능 추가 (폼 데이터를 Rust 구조체로 변환)
- `Option<String>`: 값이 있을 수도, 없을 수도 있음 (null 가능)

### 3. 요청 처리 함수들

#### a) 인사말 엔드포인트

```rust
async fn greet(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}
```

- URL 경로에서 `name` 파라미터를 추출
- 없으면 기본값 "World" 사용
- `format!`: 문자열 포맷팅 (Python의 f-string 비슷)

#### b) 건강 체크 엔드포인트

```rust
async fn health_check() -> impl Responder {
    "OK"
}
```

- 서버가 정상 동작하는지 확인하는 간단한 엔드포인트

#### c) 이메일 검증 함수

```rust
fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    !trimmed.is_empty() && trimmed.contains('@') && trimmed.len() > 5
}
```

- `&str`: 문자열 참조 (소유권을 빌려옴)
- 이메일 형식이 유효한지 간단히 검증

#### d) 구독 처리 엔드포인트

```rust
async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    // 이름 검증
    let name_valid = form.name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    // 이메일 검증
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

- `web::Form<FormData>`: 폼 데이터를 자동으로 파싱
- `.as_ref()`: Option 값을 참조로 변환
- `.map()`: 값이 있으면 함수 적용
- `.unwrap_or(false)`: 값이 없으면 기본값 사용
- 유효성 검증 후 200 OK 또는 400 Bad Request 반환

### 4. 서버 시작 함수들

#### a) startup 함수

```rust
pub fn startup(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
```

- **라우트 설정**: URL 경로와 처리 함수를 연결
  - `GET /health_check` → `health_check` 함수
  - `POST /subscriptions` → `subscribe` 함수
  - `GET /` → `greet` 함수
  - `GET /{name}` → `greet` 함수 (name은 URL 파라미터)

- `pub`: 다른 파일에서 사용할 수 있도록 공개
- `TcpListener`: 네트워크 포트를 관리하는 객체

#### b) run 함수

```rust
pub async fn run() -> std::io::Result<Server> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    startup(listener)
}
```

- 8080 포트에 서버를 바인딩
- `startup` 함수를 호출하여 서버 생성

### 초보자를 위한 팁
> `lib.rs`는 프로젝트의 핵심입니다. 모든 비즈니스 로직이 여기 있습니다. 라우트 설정 → 요청 처리 함수 → 응답 반환의 흐름을 이해하면 웹 서버의 기본을 이해한 것입니다!

---

## tests/health_check.rs - 통합 테스트

**위치**: `zero2prod/tests/health_check.rs`

### 역할
실제로 서버를 실행하고 HTTP 요청을 보내서 제대로 동작하는지 테스트합니다.

### 1. 테스트 헬퍼 함수

```rust
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
```

- `"127.0.0.1:0"`: 운영체제가 자동으로 사용 가능한 포트 할당
- `tokio::spawn`: 백그라운드에서 서버 실행
- 서버 주소(URL)를 문자열로 반환

### 2. 테스트 케이스들

#### a) 건강 체크 테스트

```rust
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
```

- `#[tokio::test]`: 비동기 테스트 매크로
- `reqwest::Client`: HTTP 클라이언트로 요청 전송
- `assert!`: 조건이 참인지 검증
- `assert_eq!`: 두 값이 같은지 검증

#### b) 인사말 테스트

```rust
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
```

- URL에 이름을 포함하여 요청
- "Hello Alice!" 응답이 오는지 확인

#### c) 구독 성공 테스트

```rust
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}
```

- POST 요청으로 폼 데이터 전송
- 유효한 데이터일 때 200 응답 확인
- `%20`: URL 인코딩된 공백
- `%40`: URL 인코딩된 @

#### d) 구독 실패 테스트

```rust
#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
```

- 여러 잘못된 케이스를 테스트
- 각 케이스마다 400 에러가 반환되는지 확인
- `for` 루프로 여러 테스트를 한 번에 실행

### 초보자를 위한 팁
> 테스트는 코드가 제대로 동작하는지 자동으로 확인합니다. `cargo test` 명령어로 모든 테스트를 실행할 수 있습니다. 좋은 테스트는 버그를 빨리 찾고 코드 변경 시 안전하게 리팩토링할 수 있게 해줍니다.

---

## 파일 간 연결 관계

```
┌─────────────────┐
│  Cargo.toml     │  ← 프로젝트 설정, 의존성 정의
└─────────────────┘

┌─────────────────┐
│  src/main.rs    │  ← 프로그램 시작점
│                 │
│  use zero2prod  │  ← lib.rs를 import
│  ::run;         │
│                 │
│  main() {       │
│    run().await  │  ← lib.rs의 run() 호출
│  }              │
└────────┬────────┘
         │
         │ 호출
         ▼
┌─────────────────┐
│  src/lib.rs     │  ← 핵심 비즈니스 로직
│                 │
│  - greet()      │  ← 인사말 처리
│  - health_check │  ← 건강 체크
│  - subscribe()  │  ← 구독 처리
│  - startup()    │  ← 서버 설정
│  - run()        │  ← 서버 시작 (main.rs에서 호출)
└────────┬────────┘
         │
         │ 테스트
         ▼
┌─────────────────┐
│  tests/         │
│  health_check.rs│  ← 통합 테스트
│                 │
│  use zero2prod  │  ← lib.rs를 import
│  ::startup;     │
│                 │
│  spawn_app() {  │
│    startup(...)│  ← lib.rs의 startup() 호출
│  }              │
│                 │
│  테스트 함수들  │  ← 실제 HTTP 요청으로 테스트
└─────────────────┘
```

### 흐름 요약

1. **Cargo.toml**이 필요한 라이브러리들을 정의
2. **main.rs**가 프로그램을 시작하고 **lib.rs**의 `run()` 호출
3. **lib.rs**가 서버를 설정하고 실행 (라우트, 처리 함수 등)
4. **tests/health_check.rs**가 **lib.rs**의 함수들을 사용해서 테스트

---

## 실행 흐름

### 서버 실행 시

```
1. cargo run
   ↓
2. main.rs의 main() 실행
   ↓
3. lib.rs의 run() 호출
   ↓
4. TcpListener가 8080 포트에 바인딩
   ↓
5. startup() 함수로 라우트 설정
   ↓
6. HttpServer 시작 (무한 대기)
   ↓
7. 요청이 오면:
   - /health_check → health_check() 함수
   - /subscriptions → subscribe() 함수
   - /{name} → greet() 함수
```

### 테스트 실행 시

```
1. cargo test
   ↓
2. health_check.rs의 각 테스트 함수 실행
   ↓
3. spawn_app()으로 테스트용 서버 생성
   - 랜덤 포트에 바인딩
   - 백그라운드에서 서버 실행
   ↓
4. reqwest로 HTTP 요청 전송
   ↓
5. 응답 검증 (assert)
   ↓
6. 테스트 통과/실패 결과 출력
```

---

## 주요 Rust 개념 정리

### 1. 소유권 (Ownership)
Rust의 가장 독특한 특징입니다. 각 값은 하나의 "소유자"만 가질 수 있습니다.

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1의 소유권이 s2로 이동
// println!("{}", s1);  // 에러! s1은 더 이상 사용 불가
```

### 2. 참조와 빌림 (References & Borrowing)
소유권을 이동하지 않고 값을 "빌려서" 사용할 수 있습니다.

```rust
fn is_valid_email(email: &str) -> bool {  // &str은 참조
    // email을 빌려서 사용
}
```

### 3. Option 타입
값이 있을 수도, 없을 수도 있는 경우를 표현합니다 (null 대신).

```rust
struct FormData {
    name: Option<String>,  // Some("홍길동") 또는 None
}
```

### 4. Result 타입
성공 또는 에러를 표현합니다.

```rust
fn main() -> std::io::Result<()> {  // Ok(()) 또는 Err(에러)
    // ...
}
```

### 5. 비동기 프로그래밍
`async/await`로 비블로킹 작업을 처리합니다.

```rust
async fn greet() -> impl Responder {
    // 비동기 함수
}

// 호출 시 .await 사용
let result = greet().await;
```

---

## 실습 가이드

### 1. 서버 실행하기

```bash
cd zero2prod
cargo run
```

브라우저에서 `http://localhost:8080` 접속

### 2. 테스트 실행하기

```bash
cargo test
```

### 3. 새로운 엔드포인트 추가해보기

**lib.rs에 함수 추가**:
```rust
async fn goodbye(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Goodbye {}!", name)
}
```

**startup 함수에 라우트 추가**:
```rust
.route("/goodbye/{name}", web::get().to(goodbye))
```

**테스트 추가** (health_check.rs):
```rust
#[tokio::test]
async fn goodbye_works() {
    let addr = spawn_app();

    let response = reqwest::Client::new()
        .get(&format!("{}/goodbye/Alice", addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.text().await.unwrap(), "Goodbye Alice!");
}
```

---

## 다음 단계

1. **데이터베이스 연동**: PostgreSQL이나 SQLite 추가
2. **로깅**: tracing 크레이트로 로그 추가
3. **에러 처리**: 더 상세한 에러 메시지
4. **인증**: JWT 토큰 기반 인증 추가
5. **배포**: Docker 컨테이너로 배포

---

## 참고 자료

- [Rust 공식 문서](https://doc.rust-lang.org/book/)
- [actix-web 문서](https://actix.rs/)
- [tokio 문서](https://tokio.rs/)
- [Zero To Production In Rust 책](https://www.zero2prod.com/)

---

## 자주 묻는 질문 (FAQ)

### Q: `?` 연산자는 무엇인가요?
A: 에러가 발생하면 즉시 함수에서 반환합니다. 에러 처리를 간결하게 만들어줍니다.

```rust
let listener = TcpListener::bind("127.0.0.1:8080")?;
// 위는 아래와 같습니다:
let listener = match TcpListener::bind("127.0.0.1:8080") {
    Ok(l) => l,
    Err(e) => return Err(e),
};
```

### Q: `impl Responder`는 무엇인가요?
A: "Responder 트레이트를 구현하는 어떤 타입"을 의미합니다. 구체적인 타입을 명시하지 않아도 됩니다.

### Q: 왜 `main.rs`와 `lib.rs`를 분리하나요?
A:
- **재사용성**: 다른 프로그램에서 lib.rs를 라이브러리로 사용 가능
- **테스트**: 테스트에서 lib.rs의 함수를 직접 호출 가능
- **모듈화**: 코드를 논리적으로 분리

### Q: `#[tokio::main]`은 무엇인가요?
A: 비동기 런타임을 초기화하고 `async fn main`을 실행할 수 있게 해주는 매크로입니다.

---

이 가이드가 Rust와 웹 서버 개발을 이해하는 데 도움이 되길 바랍니다! 🦀
