# Health Check 테스트 설명서 (2025-10-24)

## 📌 개요

이 문서는 `health_check.rs` 파일의 통합 테스트(Integration Tests)에 대해 초급 개발자를 위해 설명합니다.
이 파일은 zero2prod 서버의 다양한 엔드포인트가 정상적으로 작동하는지 검증하는 테스트들을 포함하고 있습니다.

---

## 🔧 주요 개념

### 통합 테스트(Integration Test)란?
- 여러 개의 모듈이나 컴포넌트가 함께 올바르게 작동하는지 확인하는 테스트입니다.
- 이 파일의 테스트들은 실제 서버를 시작하고 HTTP 요청을 보내 응답을 확인합니다.

### 비동기 프로그래밍(Async/Await)
- `#[tokio::test]`: 토크(Tokio) 비동기 런타임에서 테스트를 실행하는 매크로입니다.
- `async`: 비동기 함수를 선언합니다. 시간이 걸리는 작업(예: 네트워크 요청)이 완료될 때까지 기다릴 수 있습니다.
- `await`: 비동기 작업이 완료될 때까지 기다립니다.

---

## 📋 코드 상세 설명

### 1️⃣ `spawn_app()` 함수 (6-18줄)

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

**역할:** 테스트용 서버를 시작하고 서버 주소를 반환합니다.

**단계별 설명:**
1. **`TcpListener::bind("127.0.0.1:0")`**:
   - 로컬 컴퓨터(127.0.0.1)에 무작위 포트(0)로 바인딩합니다.
   - 포트 0은 OS가 자동으로 사용 가능한 포트를 할당한다는 의미입니다.

2. **`listener.local_addr().unwrap().port()`**:
   - 할당된 포트 번호를 가져옵니다.

3. **`startup(listener)`**:
   - 서버를 초기화합니다.

4. **`tokio::spawn()`**:
   - 서버를 백그라운드에서 실행합니다. (메인 스레드를 블로킹하지 않습니다)

5. **`format!("http://127.0.0.1:{}", port)`**:
   - 서버 주소를 문자열로 반환합니다. (예: "http://127.0.0.1:5000")

---

### 2️⃣ `health_check_works()` 테스트 (20-32줄)

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

**목적:** `/health_check` 엔드포인트가 정상적으로 작동하는지 확인합니다.

**검증 항목:**
- ✅ HTTP 상태 코드가 성공(200-299)인지 확인
- ✅ 응답 본문이 "OK"인지 확인

**테스트 흐름:**
1. 테스트 서버 시작
2. `/health_check` 엔드포인트로 GET 요청 전송
3. 응답이 성공 상태인지, 본문이 "OK"인지 확인

---

### 3️⃣ `greet_returns_name()` 테스트 (34-46줄)

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

**목적:** 이름을 파라미터로 받아 인사말을 반환하는 엔드포인트를 테스트합니다.

**검증 항목:**
- ✅ `/Alice` 엔드포인트에 요청했을 때 "Hello Alice!" 응답 확인

---

### 4️⃣ `greet_default_world()` 테스트 (48-60줄)

```rust
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
```

**목적:** 파라미터 없이 루트 경로(`/`)로 요청했을 때 기본값으로 "Hello World!"를 반환하는지 확인합니다.

---

### 5️⃣ `subscribe_returns_200_for_valid_form_data()` 테스트 (62-81줄)

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

**목적:** 유효한 구독 양식 데이터를 POST로 전송했을 때 200 응답을 반환하는지 확인합니다.

**주요 요소:**
- **`let body = "name=le%20guin&email=ursula_le_guin%40gmail.com"`**:
  - URL 인코딩된 양식 데이터 (le%20guin은 "le guin"의 URL 인코딩)
- **`.post()`**: GET 대신 POST 요청을 보냅니다.
- **`.header("Content-Type", "application/x-www-form-urlencoded")`**:
  - 서버에 데이터 형식을 알려줍니다.

---

### 6️⃣ `subscribe_returns_400_when_data_is_missing()` 테스트 (83-111줄)

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

        assert_eq!(400, response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message);
    }
}
```

**목적:** 필수 데이터가 누락되었을 때 400(Bad Request) 오류를 반환하는지 확인합니다.

**테스트 케이스:**
| 요청 데이터 | 설명 |
|-----------|------|
| `name=le%20guin` | 이메일 누락 |
| `email=ursula_le_guin%40gmail.com` | 이름 누락 |
| `` | 이름과 이메일 모두 누락 |

**특징:**
- `for` 루프를 사용해 여러 케이스를 한 번에 테스트합니다.
- 각 케이스마다 400 상태 코드를 확인합니다.

---

## 🚀 테스트 실행 방법

### 모든 테스트 실행
```bash
cargo test
```

### 특정 테스트만 실행
```bash
cargo test health_check_works
```

### 테스트 출력 보기
```bash
cargo test -- --nocapture
```

---

## 📌 초급 개발자를 위한 팁

### 1. `expect()` vs `unwrap()`
- 둘 다 에러 발생 시 프로그램을 중단합니다.
- `expect("메시지")`는 커스텀 에러 메시지를 제공합니다. (권장)

### 2. 상태 코드 이해
| 코드 | 의미 |
|------|------|
| 200 | OK (성공) |
| 400 | Bad Request (잘못된 요청) |
| 200-299 | 성공 범위 |

### 3. URL 인코딩
- `%20` = 공백
- `%40` = @
- 한국어나 특수문자가 포함된 데이터는 인코딩이 필요합니다.

### 4. AAA 패턴 (Arrange, Act, Assert)
일부 테스트에서 사용되는 패턴:
- **Arrange**: 테스트 준비 (데이터 설정)
- **Act**: 테스트 실행 (요청 전송)
- **Assert**: 결과 검증 (응답 확인)

---

## 🎯 요약

이 테스트 파일은 다음을 검증합니다:

1. ✅ 서버가 정상적으로 시작되는가?
2. ✅ `/health_check` 엔드포인트가 작동하는가?
3. ✅ 이름을 받아 인사말을 반환하는가?
4. ✅ 기본값 "Hello World!"를 반환하는가?
5. ✅ 유효한 구독 데이터를 받으면 200을 반환하는가?
6. ✅ 필수 데이터가 없으면 400을 반환하는가?

이러한 테스트들은 서버의 핵심 기능이 제대로 작동하는지 보장합니다.
