# Rust Tokio `spawn` 완벽 가이드

## 📚 목차
1. [기본 개념](#기본-개념)
2. [Tokio Runtime 이해하기](#tokio-runtime-이해하기)
3. [spawn 함수 상세 설명](#spawn-함수-상세-설명)
4. [프로젝트의 spawn 사용 예시](#프로젝트의-spawn-사용-예시)
5. [실행 흐름 분석](#실행-흐름-분석)
6. [주요 개념 정리](#주요-개념-정리)

---

## 기본 개념

### 비동기 프로그래밍(Async Programming)이란?

**동기 방식 (Synchronous)**
```
작업 A 완료 → 작업 B 시작 → 작업 C 시작
⏱️  대기 시간 발생 (비효율)
```

**비동기 방식 (Asynchronous)**
```
작업 A 시작 (완료 대기) → 작업 B 시작 (완료 대기) → 작업 C 시작
✅ 동시에 진행 가능 (효율적)
```

### Tokio란?

Rust의 **비동기 런타임(Async Runtime)** 라이브러리입니다.
- 여러 작업을 동시에 처리할 수 있게 해줍니다
- 네트워크 I/O, 파일 I/O 등을 효율적으로 관리합니다
- 스레드보다 가볍고 빠릅니다

---

## Tokio Runtime 이해하기

### Runtime이란?

프로그램이 실행되는 "환경"입니다.

```rust
#[tokio::main]
async fn main() {
    // 이 매크로가 Tokio Runtime을 자동으로 생성합니다
}
```

**`#[tokio::main]` 매크로가 하는 일:**
```rust
// 실제로는 이렇게 변환됩니다:
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // 여기가 async main 코드
    })
}
```

### Runtime의 역할

| 역할 | 설명 |
|------|------|
| **Task 스케줄링** | 여러 작업을 CPU에 분배 |
| **I/O 관리** | 네트워크, 파일 등의 I/O 처리 |
| **타이머 관리** | sleep, timeout 등 처리 |
| **작업 실행** | 비동기 작업 실행 |

---

## spawn 함수 상세 설명

### spawn의 기본 문법

```rust
tokio::spawn(async {
    // 비동기 코드
});
```

### spawn이 하는 일

```
Runtime (메인 프로세스)
│
├─ Task 1 (spawn으로 생성)
│  └─ 병렬 실행 중...
│
├─ Task 2 (spawn으로 생성)
│  └─ 병렬 실행 중...
│
└─ 메인 코드 계속 실행...
```

### spawn의 특징

| 특징 | 설명 |
|------|------|
| **비차단(Non-blocking)** | 새 작업을 시작하고 즉시 반환 |
| **백그라운드 실행** | 메인 코드와 동시 실행 |
| **독립적** | 각 작업이 독립적으로 실행 |
| **가볍다** | 스레드보다 메모리 사용 적음 |

### spawn 반환값

```rust
let handle = tokio::spawn(async {
    42
});

// handle은 JoinHandle<T>를 반환합니다
let result = handle.await.unwrap(); // 42
```

---

## 프로젝트의 spawn 사용 예시

### 코드 위치: `tests/health_check.rs` (13-15줄)

```rust
let _ = tokio::spawn(async move {
    let _ = server.await;
});
```

### 단계별 분석

#### 1단계: 함수 정의
```rust
fn spawn_app() -> String {
    // 이 함수는 테스트를 위해 서버를 시작하고 주소를 반환합니다
```

#### 2단계: 리스너 생성
```rust
let listener = TcpListener::bind("127.0.0.1:0")
    .expect("Failed to bind random port");
let port = listener.local_addr().unwrap().port();
```

**"127.0.0.1:0"의 의미:**
- `127.0.0.1`: localhost (이 컴퓨터만 접근 가능)
- `:0`: OS가 사용 가능한 임의의 포트 할당

#### 3단계: 서버 생성
```rust
let server = startup(listener)
    .expect("Failed to create server");
```

**startup 함수 (`src/lib.rs` 13-24줄):**
```rust
pub fn startup(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
```

#### 4단계: spawn으로 백그라운드에서 실행
```rust
let _ = tokio::spawn(async move {
    let _ = server.await;
});
```

**이 코드의 의미:**
```
┌─────────────────────────────────────────┐
│ tokio::spawn( ... )                     │
│ ├─ 새로운 Task 생성                      │
│ └─ 즉시 반환 (대기하지 않음)              │
└─────────────────────────────────────────┘
        │
        ├─ 백그라운드에서 실행
        │  async move {
        │      server.await  ← 서버 시작 & 대기
        │  }
        │
        └─ 메인 코드는 계속 진행
            return addr
```

#### 5단계: 주소 반환
```rust
format!("http://127.0.0.1:{}", port)
```

---

## 실행 흐름 분석

### 전체 시작 프로세스

```
1. main() 시작 (@main.rs)
   │
   ├─ #[tokio::main] 매크로 작동
   └─ Tokio Runtime 생성

2. run() 호출 (@lib.rs)
   │
   ├─ TcpListener 바인드
   └─ startup() 호출

3. startup() 실행 (@lib.rs)
   │
   ├─ HttpServer 생성
   ├─ 라우트 등록 (/health_check, /)
   └─ server.run() 호출 (아직 시작 안됨)

4. server.await 대기
   │
   └─ ✅ 서버 실행 중!
```

### 테스트 시 실행 흐름

```
1. #[tokio::test] 매크로 작동
   │
   └─ 테스트 코드를 위한 Runtime 생성

2. spawn_app() 호출
   │
   ├─ 리스너 바인드
   ├─ 서버 생성
   │
   ├─ tokio::spawn() 호출 ⭐
   │  │
   │  └─ 백그라운드 Task 생성
   │     └─ server.await (서버 실행)
   │
   └─ 즉시 주소 반환

3. HTTP 요청 전송
   │
   ├─ GET /health_check
   └─ ✅ 백그라운드의 서버가 응답

4. 테스트 완료
   │
   └─ Runtime 종료 (모든 Task도 함께 종료)
```

---

## 주요 개념 정리

### spawn의 용도

```rust
// ✅ 좋은 사용 예시들:

// 1. 백그라운드 작업
tokio::spawn(async {
    println!("백그라운드에서 실행");
});

// 2. 서버 실행 (테스트)
tokio::spawn(async move {
    server.await
});

// 3. 타이머
tokio::spawn(async {
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("10초 후 실행");
});

// 4. 여러 작업 동시 처리
tokio::spawn(async { task1().await });
tokio::spawn(async { task2().await });
tokio::spawn(async { task3().await });
```

### spawn vs await

| 항목 | spawn | await |
|------|-------|-------|
| **대기** | ❌ 하지 않음 | ✅ 함 |
| **반환** | 즉시 | 작업 완료 후 |
| **병렬 실행** | ✅ 가능 | ❌ 순차 실행 |
| **사용 사례** | 백그라운드 작업 | 결과 필요할 때 |

### async move의 의미

```rust
let server = startup(listener).unwrap();

// move: server 변수를 새 Task로 이동
tokio::spawn(async move {
    server.await
});

// server는 이제 spawn된 Task 내에만 존재
// 메인 코드에서는 사용 불가
```

### 에러 처리

```rust
// JoinHandle은 Result를 반환
let handle = tokio::spawn(async {
    Ok::<i32, String>(42)
});

match handle.await {
    Ok(Ok(value)) => println!("성공: {}", value),
    Ok(Err(e)) => println!("Task 내 에러: {}", e),
    Err(e) => println!("Task 조인 실패: {}", e),
}
```

---

## 이 프로젝트에서의 역할

### 테스트 구조

```
test 실행 (tokio::test)
│
├─ spawn_app()
│  └─ tokio::spawn(server.await)
│     └─ 백그라운드에서 서버 실행
│
├─ HTTP 클라이언트로 요청 전송
│
└─ 응답 확인 & 테스트 완료
```

### 왜 spawn을 사용?

1. **테스트 병렬화**: 여러 테스트를 동시에 실행 가능
2. **메인 코드 대기 없음**: `spawn`으로 서버를 백그라운드에서 실행
3. **테스트 속도 향상**: 각 테스트가 독립적으로 서버 실행

---

## 요약

| 개념 | 설명 |
|------|------|
| **spawn** | 새로운 비동기 작업을 백그라운드에서 시작 |
| **Runtime** | 비동기 작업을 관리하는 환경 |
| **async/await** | 비동기 코드 작성 문법 |
| **Task** | spawn으로 생성되는 실행 단위 |
| **JoinHandle** | spawn이 반환하는 작업 핸들 |

이제 이 프로젝트의 spawn 사용을 완전히 이해했습니다! 🚀
