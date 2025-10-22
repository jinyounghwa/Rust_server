# health_check.rs 파일 설명 (초급 개발자용)

## 📚 이 파일은 무엇인가요?

`health_check.rs`는 **통합 테스트(Integration Test)** 파일입니다.
우리가 만든 웹 서버가 제대로 작동하는지 자동으로 확인하는 코드입니다.

## 🎯 왜 필요한가요?

웹 서버를 만들면 다음과 같은 질문들이 생깁니다:
- "서버가 정말 시작되나요?"
- "API 요청을 보내면 올바른 응답이 오나요?"
- "내가 코드를 수정했을 때 기존 기능이 망가지지 않았나요?"

이런 질문들에 답하기 위해 **자동 테스트**를 작성합니다.

## 📖 코드 상세 설명

### 1. 필요한 라이브러리 가져오기

```rust
use std::net::TcpListener;
use zero2prod::startup;
```

- `TcpListener`: 네트워크 연결을 받는 도구 (전화기의 수화기 같은 역할)
- `zero2prod::startup`: 우리가 만든 서버를 시작하는 함수

---

### 2. `spawn_app()` 함수 - 테스트용 서버 시작하기

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

#### 🔍 단계별 설명

1. **`TcpListener::bind("127.0.0.1:0")`**
   - `127.0.0.1`: 로컬호스트 (내 컴퓨터)
   - `:0`: 운영체제야, 사용 가능한 포트 아무거나 골라줘!
   - 왜 0을 쓸까? → 테스트를 여러 개 동시에 실행해도 포트 충돌이 없도록

2. **`let port = listener.local_addr().unwrap().port();`**
   - 운영체제가 할당해준 포트 번호를 가져옴
   - 예: 8080, 3000 같은 숫자

3. **`let server = startup(listener)`**
   - 실제 서버를 생성

4. **`tokio::spawn(async move { ... })`**
   - 서버를 백그라운드에서 실행
   - 비유: 음악을 재생하고 다른 작업을 계속하는 것처럼

5. **`format!("http://127.0.0.1:{}", port)`**
   - 서버 주소를 문자열로 반환
   - 예: "http://127.0.0.1:8080"

---

### 3. 테스트 1: `health_check_works()` - 헬스 체크 테스트

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

#### 🔍 무슨 일을 하나요?

1. **`let addr = spawn_app();`**
   - 테스트용 서버 시작

2. **`reqwest::Client::new().get(...)`**
   - HTTP GET 요청 보내기
   - 브라우저에서 주소창에 URL 입력하는 것과 같음
   - 요청 주소: `http://127.0.0.1:포트번호/health_check`

3. **`assert!(response.status().is_success());`**
   - 응답 상태 코드가 성공(200)인지 확인
   - HTTP 상태 코드란?
     - 200: 성공
     - 404: 페이지를 찾을 수 없음
     - 500: 서버 에러

4. **`assert_eq!(response.text().await.unwrap(), "OK");`**
   - 응답 본문이 정확히 "OK"인지 확인

#### ✅ 이 테스트가 성공하면?

서버가 제대로 시작되고, `/health_check` 엔드포인트가 "OK"를 반환한다는 것을 확인!

---

### 4. 테스트 2: `greet_returns_name()` - 이름으로 인사하기

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

#### 🔍 무슨 일을 하나요?

- `/Alice` 주소로 요청을 보냄
- 서버가 "Hello Alice!"를 응답하는지 확인
- 다른 이름을 넣으면? → "Hello [이름]!"이 반환되어야 함

---

### 5. 테스트 3: `greet_default_world()` - 기본 인사

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

#### 🔍 무슨 일을 하나요?

- 루트 경로(`/`)로 요청을 보냄
- 서버가 "Hello World!"를 응답하는지 확인
- 이름을 지정하지 않으면 기본값으로 "World"를 사용

---

## 🚀 테스트 실행하기

터미널에서 다음 명령어를 실행:

```bash
cargo test
```

모든 테스트가 통과하면 ✅ 표시가 나타납니다!

---

## 💡 핵심 개념 정리

### 1. **통합 테스트 (Integration Test)**
- 여러 부분이 함께 제대로 동작하는지 확인
- 실제 서버를 시작하고 HTTP 요청을 보내서 테스트

### 2. **`#[tokio::test]` 속성**
- 비동기 테스트를 실행할 수 있게 해줌
- `async/await`를 사용할 수 있음

### 3. **`assert!` 매크로**
- `assert!(조건)`: 조건이 참인지 확인
- `assert_eq!(a, b)`: a와 b가 같은지 확인
- 실패하면 테스트가 중단되고 에러 메시지 출력

### 4. **왜 포트를 0으로 바인딩하나요?**
- 운영체제가 자동으로 사용 가능한 포트를 할당
- 여러 테스트를 동시에 실행해도 충돌 없음

---

## 🎓 초급 개발자를 위한 팁

1. **테스트는 문서입니다**
   - 이 코드를 보면 서버가 어떻게 동작해야 하는지 알 수 있어요

2. **작은 변경 후 항상 테스트 실행**
   ```bash
   cargo test
   ```

3. **테스트가 실패하면?**
   - 에러 메시지를 꼼꼼히 읽어보세요
   - 어떤 assert가 실패했는지 확인하세요
   - 예상값과 실제값을 비교하세요

4. **새 기능을 추가하면?**
   - 새로운 테스트도 함께 작성하세요
   - 예: `/goodbye` 엔드포인트를 추가하면 `test_goodbye()` 작성

---

## 📚 더 공부하기

- **Rust 공식 문서**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **Tokio 문서**: https://tokio.rs/
- **Reqwest 문서**: https://docs.rs/reqwest/

---

## ❓ 자주 묻는 질문

**Q: 왜 `unwrap()`을 사용하나요?**
- 테스트 코드에서는 에러가 발생하면 즉시 패닉(panic)해도 괜찮아요
- 실제 프로덕션 코드에서는 `unwrap()` 대신 적절한 에러 처리가 필요합니다

**Q: `async/await`가 뭔가요?**
- 비동기 프로그래밍: 작업이 완료될 때까지 기다리지 않고 다른 작업을 할 수 있음
- `await`: 이 작업이 완료될 때까지 기다려 줘

**Q: 테스트가 너무 느려요**
- 통합 테스트는 실제 서버를 시작하므로 단위 테스트보다 느립니다
- 빠른 피드백이 필요하면 단위 테스트를 더 많이 작성하세요

---

이 문서가 도움이 되었기를 바랍니다! 🎉
