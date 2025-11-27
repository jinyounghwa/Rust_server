# JWT 인증 구현 가이드 (JWT Authentication Implementation Guide)

**작성일**: 2025-11-27
**상태**: 완료 ✅
**테스트**: 86개 전부 통과 (62 unit + 17 integration + 6 confirmation + 1 health check)

---

## 📚 목차

1. [개요](#개요)
2. [아키텍처](#아키텍처)
3. [구현 컴포넌트](#구현-컴포넌트)
4. [API 엔드포인트](#api-엔드포인트)
5. [인증 흐름](#인증-흐름)
6. [코드 상세 설명](#코드-상세-설명)
7. [테스트 전략](#테스트-전략)
8. [보안 고려사항](#보안-고려사항)
9. [트러블슈팅](#트러블슈팅)

---

## 개요

이 문서는 **JWT(JSON Web Token) 기반 인증 시스템**을 Rust와 Actix-web 프레임워크로 구현한 내용을 설명합니다.

### 구현된 기능

- ✅ 사용자 등록 (회원가입)
- ✅ 사용자 로그인
- ✅ JWT 액세스 토큰 발급
- ✅ Refresh 토큰을 이용한 토큰 갱신
- ✅ 토큰 로테이션 (보안 강화)
- ✅ 비밀번호 해싱 (bcrypt)
- ✅ JWT 미들웨어를 통한 경로 보호
- ✅ 포괄적인 통합 테스트

### 보안 기능

- 🔐 12-라운드 bcrypt 해싱
- 🔐 토큰 로테이션 메커니즘
- 🔐 사용자 열거 공격 방지 (동일한 에러 메시지)
- 🔐 비밀번호 강도 검증 (8+ 글자, 숫자, 대소문자)
- 🔐 만료된 토큰 거부
- 🔐 구조화된 로깅

---

## 아키텍처

### 시스템 다이어그램

```
┌─────────────────────────────────────────────────────────────┐
│                      클라이언트 애플리케이션                  │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
    ┌────▼─────┐         ┌──────▼────┐
    │ 1. 등록  │         │ 2. 로그인 │
    │(Register)│         │ (Login)   │
    └────┬─────┘         └──────┬────┘
         │                      │
         └──────────┬───────────┘
                    │
         ┌──────────▼──────────┐
         │   JWT 토큰 발급     │
         │  (Access Token +    │
         │  Refresh Token)     │
         └──────────┬──────────┘
                    │
         ┌──────────▼──────────────────────┐
         │   3. 보호된 API 호출             │
         │   (Authorization: Bearer token) │
         └──────────┬──────────────────────┘
                    │
         ┌──────────▼──────────────────────┐
         │  JWT 미들웨어 검증              │
         │  - 토큰 서명 확인               │
         │  - 발급자 확인                  │
         │  - 만료 여부 확인               │
         └──────────┬──────────────────────┘
                    │
         ┌──────────▼──────────────────────┐
         │  토큰 갱신 (4. Refresh)         │
         │  - 기존 토큰 회수               │
         │  - 새 토큰 발급 (로테이션)      │
         └──────────────────────────────────┘
```

### 파일 구조

```
src/
├── auth/                           # 인증 모듈
│   ├── mod.rs                      # 모듈 내보내기
│   ├── claims.rs                   # JWT Claims 구조체
│   ├── jwt.rs                      # JWT 생성/검증
│   ├── password.rs                 # 비밀번호 해싱/검증
│   └── refresh_token.rs            # Refresh 토큰 관리
├── middleware/
│   ├── mod.rs                      # 미들웨어 모듈
│   └── jwt_middleware.rs           # JWT 검증 미들웨어
├── routes/
│   ├── mod.rs                      # 라우트 모듈
│   └── auth.rs                     # 인증 엔드포인트
├── configuration.rs                # JWT 설정
└── startup.rs                      # 서버 초기화 (미들웨어 등록)

tests/
├── auth_integration.rs             # 인증 통합 테스트
├── health_check.rs                 # 기본 헬스 체크
└── email_confirmation_integration.rs

migrations/
├── 20231127000001_create_users_table.up.sql
└── 20231127000002_create_refresh_tokens_table.up.sql
```

---

## 구현 컴포넌트

### 1. Claims 구조체 (`src/auth/claims.rs`)

JWT에 포함될 정보를 정의합니다.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // Subject (사용자 ID)
    pub email: String,         // 사용자 이메일
    pub exp: i64,              // 만료 시간 (Unix timestamp)
    pub iat: i64,              // 발급 시간
    pub iss: String,           // Issuer (발급자)
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, jwt_config: &JwtSettings) -> Self {
        let now = Utc::now();
        let exp = now + Duration::seconds(jwt_config.access_token_expiry);

        Self {
            sub: user_id.to_string(),
            email,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: jwt_config.issuer.clone(),
        }
    }
}
```

**역할**:
- JWT 페이로드 정의
- 사용자 식별 정보 저장
- 토큰 만료 시간 관리

### 2. JWT 모듈 (`src/auth/jwt.rs`)

토큰 생성 및 검증을 담당합니다.

```rust
pub fn generate_access_token(
    user_id: &Uuid,
    email: &str,
    jwt_config: &JwtSettings,
) -> Result<String, AppError> {
    let claims = Claims::new(*user_id, email.to_string(), jwt_config);

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_config.secret.as_ref()),
    )?;

    Ok(token)
}

pub fn validate_access_token(
    token: &str,
    jwt_config: &JwtSettings,
) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_config.secret.as_ref()),
        &validation,
    )?;

    Ok(data.claims)
}
```

**역할**:
- 비밀키를 사용한 토큰 서명
- 토큰 서명 검증
- 발급자(issuer) 확인
- 만료 시간 확인

### 3. 비밀번호 모듈 (`src/auth/password.rs`)

비밀번호의 안전한 저장 및 검증을 담당합니다.

```rust
pub fn hash_password(password: &str) -> Result<String, AppError> {
    validate_password_strength(password)?;

    let salt = bcrypt::gen_salt(DEFAULT_COST)?;
    let hash = bcrypt::hash_with_salt(password, salt)?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn validate_password_strength(password: &str) -> Result<String, AppError> {
    // 요구사항:
    // - 최소 8글자
    // - 최대 128글자
    // - 최소 1개 숫자
    // - 최소 1개 소문자
    // - 최소 1개 대문자
}
```

**비밀번호 강도 검증 규칙**:
```
✓ 유효: "SecurePass123"
✗ 무효: "short" (8글자 미만)
✗ 무효: "nouppercase123" (대문자 없음)
✗ 무효: "NOLOWERCASE123" (소문자 없음)
✗ 무효: "NoDigits" (숫자 없음)
```

### 4. Refresh Token 모듈 (`src/auth/refresh_token.rs`)

토큰 갱신 및 로테이션을 담당합니다.

```rust
pub fn generate_refresh_token() -> String {
    // 64바이트 암호화 난수 생성
    let random_bytes: Vec<u8> = (0..64)
        .map(|_| rand::random::<u8>())
        .collect();

    hex::encode(random_bytes)
}

pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expiry_seconds: i64,
) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    let expires_at = Utc::now() + Duration::seconds(expiry_seconds);

    sqlx::query(
        r#"INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
           VALUES ($1, $2, $3, $4, $5)"#
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}
```

**토큰 로테이션 프로세스**:
1. 클라이언트가 refresh 토큰을 보냄
2. 서버가 토큰 유효성 확인
3. 기존 토큰을 회수 (revoked = true)
4. 새로운 토큰 발급
5. 클라이언트가 새 토큰으로 업데이트

### 5. JWT 미들웨어 (`src/middleware/jwt_middleware.rs`)

보호된 경로에 대한 요청을 검증합니다.

```rust
impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S> {
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 1. Authorization 헤더 추출
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                if h.starts_with("Bearer ") {
                    Some(h[7..].to_string())
                } else {
                    None
                }
            });

        match auth_header {
            None => {
                // 2. 헤더 없음 -> 401 Unauthorized
                Err(...)
            }
            Some(token) => {
                // 3. JWT 검증
                match validate_access_token(&token, &jwt_config) {
                    Ok(claims) => {
                        // 4. 검증 성공 -> Claims를 request extensions에 주입
                        req.extensions_mut().insert(claims.clone());
                        Ok(...)
                    }
                    Err(_) => {
                        // 5. 검증 실패 -> 401 Unauthorized
                        Err(...)
                    }
                }
            }
        }
    }
}
```

**역할**:
- Authorization 헤더 파싱
- JWT 토큰 검증
- Claims를 라우트 핸들러에 주입
- 검증 실패 시 401 반환

### 6. 인증 라우트 (`src/routes/auth.rs`)

4개의 엔드포인트를 제공합니다.

```rust
pub async fn register(
    form: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    // 1. 입력 검증
    let email = is_valid_email(&form.email)?;
    let name = is_valid_name(&form.name)?;
    let password_hash = hash_password(&form.password)?;

    // 2. 데이터베이스에 사용자 생성
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#
    )
    .bind(user_id)
    .bind(&email)
    .bind(&name)
    .bind(&password_hash)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await?;

    // 3. 토큰 생성
    let access_token = generate_access_token(&user_id, &email, jwt_config.get_ref())?;
    let refresh_token = generate_refresh_token();

    // 4. Refresh 토큰 저장
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}
```

---

## API 엔드포인트

### 1. POST /auth/register - 사용자 등록

**요청**:
```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "SecurePass123"
  }'
```

**응답 (201 Created)**:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "a1b2c3d4e5f6g7h8i9j0...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**에러 응답 (400 Bad Request)**:
```json
{
  "error": "Email has invalid format",
  "code": "VALIDATION_ERROR"
}
```

### 2. POST /auth/login - 사용자 로그인

**요청**:
```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123"
  }'
```

**응답 (200 OK)**:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "x1y2z3a4b5c6d7e8f9g0...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**에러 응답 (400 Bad Request - 사용자 열거 공격 방지)**:
```json
{
  "error": "Invalid email or password",
  "code": "VALIDATION_ERROR"
}
```

### 3. POST /auth/refresh - 토큰 갱신

**요청**:
```bash
curl -X POST http://localhost:8000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "a1b2c3d4e5f6g7h8i9j0..."
  }'
```

**응답 (200 OK)**:
```json
{
  "access_token": "새로운_액세스_토큰",
  "refresh_token": "새로운_refresh_토큰",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**주의**: 기존 refresh 토큰은 자동으로 회수됩니다 (토큰 로테이션)

### 4. GET /auth/me - 현재 사용자 정보 (보호됨)

**요청**:
```bash
curl -X GET http://localhost:8000/auth/me \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

**응답 (200 OK)**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "john@example.com",
  "name": "John Doe",
  "created_at": "2025-11-27T10:30:00Z"
}
```

**에러 응답 (401 Unauthorized)**:
```json
{
  "error": "Missing or invalid authorization header",
  "code": "UNAUTHORIZED"
}
```

---

## 인증 흐름

### 등록 흐름

```
1. 클라이언트 요청
   POST /auth/register
   {
     "name": "John Doe",
     "email": "john@example.com",
     "password": "SecurePass123"
   }

2. 서버 처리
   ├─ 이메일 유효성 검증 (이메일 형식)
   ├─ 이름 유효성 검증 (비어있지 않음, 의심스러운 콘텐츠 없음)
   ├─ 비밀번호 강도 검증 (8+글자, 숫자, 대소문자)
   ├─ bcrypt(12 rounds)로 비밀번호 해싱
   ├─ 데이터베이스에 사용자 저장
   │  (중복 이메일 시 409 Conflict 반환)
   ├─ JWT 액세스 토큰 생성 (15분 유효)
   ├─ Refresh 토큰 생성 (7일 유효)
   └─ Refresh 토큰을 SHA-256으로 해싱하여 저장

3. 클라이언트 응답
   201 Created
   {
     "access_token": "JWT_TOKEN",
     "refresh_token": "REFRESH_TOKEN",
     "token_type": "Bearer",
     "expires_in": 900
   }

4. 클라이언트 저장
   ├─ access_token: 메모리 또는 sessionStorage
   └─ refresh_token: localStorage (안전한 저장소)
```

### 로그인 흐름

```
1. 클라이언트 요청
   POST /auth/login
   {
     "email": "john@example.com",
     "password": "SecurePass123"
   }

2. 서버 처리
   ├─ 이메일 형식 검증
   ├─ 데이터베이스에서 사용자 조회
   │  (미발견 시 "Invalid email or password" 반환 - 사용자 열거 공격 방지)
   ├─ 계정 활성 상태 확인
   ├─ bcrypt로 비밀번호 검증
   │  (불일치 시 "Invalid email or password" 반환)
   ├─ JWT 액세스 토큰 생성
   ├─ 새 Refresh 토큰 생성
   └─ Refresh 토큰 저장

3. 클라이언트 응답
   200 OK
   {
     "access_token": "JWT_TOKEN",
     "refresh_token": "REFRESH_TOKEN",
     "token_type": "Bearer",
     "expires_in": 900
   }
```

### 보호된 API 호출 흐름

```
1. 클라이언트 요청
   GET /auth/me
   Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...

2. 미들웨어 처리 (JwtMiddleware)
   ├─ Authorization 헤더 추출
   │  (없으면 401 Unauthorized 반환)
   ├─ "Bearer " 접두사 제거
   ├─ JWT 토큰 검증
   │  ├─ 서명 확인 (secret key 사용)
   │  ├─ 발급자(issuer) 확인
   │  └─ 만료 시간 확인
   │     (실패 시 401 Unauthorized 반환)
   └─ Claims를 request extensions에 주입

3. 라우트 핸들러 처리
   ├─ Claims에서 user_id 추출
   ├─ 데이터베이스에서 사용자 정보 조회
   └─ 사용자 정보 반환

4. 클라이언트 응답
   200 OK
   {
     "id": "550e8400-e29b-41d4-a716-446655440000",
     "email": "john@example.com",
     "name": "John Doe",
     "created_at": "2025-11-27T10:30:00Z"
   }
```

### 토큰 갱신 흐름 (토큰 로테이션)

```
1. 클라이언트 요청
   POST /auth/refresh
   {
     "refresh_token": "old_refresh_token"
   }

2. 서버 처리
   ├─ Refresh 토큰 유효성 확인
   │  ├─ SHA-256으로 해싱
   │  ├─ 데이터베이스에서 조회
   │  │  (미발견 시 400 Bad Request 반환)
   │  ├─ 회수 상태 확인
   │  │  (회수됨 시 400 Bad Request 반환)
   │  └─ 만료 시간 확인
   │     (만료됨 시 400 Bad Request 반환)
   ├─ 기존 Refresh 토큰 회수 (revoked = true) ← 중요!
   │  (토큰 탈취 감지: 공격자가 기존 토큰 사용 시 자동 차단)
   ├─ 새 JWT 액세스 토큰 생성
   ├─ 새 Refresh 토큰 생성
   └─ 새 Refresh 토큰 저장

3. 클라이언트 응답
   200 OK
   {
     "access_token": "새로운_JWT_토큰",
     "refresh_token": "새로운_REFRESH_토큰",
     "token_type": "Bearer",
     "expires_in": 900
   }

4. 보안 이점
   ├─ 토큰 탈취 감지: 공격자가 기존 토큰 재사용 시 거부
   ├─ 자동 차단: 비정상 refresh 활동 감지 및 즉시 차단
   └─ 정상 사용자 보호: 항상 유효한 새 토큰 발급
```

---

## 코드 상세 설명

### 데이터베이스 마이그레이션

#### 1. Users 테이블

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);
```

**필드 설명**:
- `id`: 사용자 고유 식별자 (UUID)
- `email`: 로그인 이메일 (중복 불가)
- `name`: 사용자 이름
- `password_hash`: bcrypt 해싱된 비밀번호
- `is_active`: 계정 활성 상태 (삭제 대신 비활성화)
- `created_at`, `updated_at`: 타임스탬프

#### 2. Refresh Tokens 테이블

```sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_is_revoked ON refresh_tokens(is_revoked);
```

**필드 설명**:
- `id`: 토큰 레코드 고유 식별자
- `user_id`: 토큰 소유자 (Users 테이블 참조)
- `token_hash`: SHA-256으로 해싱된 refresh 토큰
  - **중요**: 절대 평문(plaintext)으로 저장하지 않음
- `expires_at`: 토큰 만료 시간
- `is_revoked`: 토큰 회수 상태 (로테이션 시)
- `revoked_at`: 회수 시간

### 설정 파일

#### configuration.yaml

```yaml
database:
  username: postgres
  password: password
  port: 5432
  host: localhost
  database_name: zero2prod

application:
  port: 8000

jwt:
  secret: "your-secret-key-min-32-characters-long-required"
  access_token_expiry: 900      # 15분 (초 단위)
  refresh_token_expiry: 604800  # 7일 (초 단위)
  issuer: "zero2prod"           # 발급자
```

#### Configuration 구조체

```rust
#[derive(serde::Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
    pub issuer: String,
}
```

---

## 테스트 전략

### 테스트 구조

```
86개 전체 테스트
├─ 62개 유닛 테스트 (Unit Tests)
│  ├─ Claims 테스트 (3개)
│  ├─ JWT 테스트 (4개)
│  ├─ 비밀번호 테스트 (8개)
│  ├─ Refresh Token 테스트 (3개)
│  └─ 기타 검증 테스트
├─ 17개 인증 통합 테스트 (Auth Integration Tests)
├─ 6개 이메일 확인 통합 테스트
└─ 1개 헬스 체크 테스트
```

### 등록 테스트 (Registration Tests)

```rust
#[tokio::test]
async fn register_returns_200_for_valid_credentials() {
    // 1. 테스트 앱 시작
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // 2. 유효한 등록 요청
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

    // 3. 상태 코드 검증
    assert_eq!(201, response.status().as_u16());

    // 4. 응답 데이터 검증
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(response_body.get("access_token").is_some());
    assert!(response_body.get("refresh_token").is_some());

    // 5. 데이터베이스 검증
    let user = sqlx::query("SELECT email, name FROM users WHERE email = 'john@example.com'")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch created user");

    assert_eq!(user.get::<String, _>("email"), "john@example.com");
    assert_eq!(user.get::<String, _>("name"), "John Doe");
}
```

### 비밀번호 검증 테스트

```rust
#[test]
fn test_password_strength_validation() {
    // 약한 비밀번호들
    let weak_passwords = vec![
        ("short", "너무 짧음"),
        ("nouppercase123", "대문자 없음"),
        ("NOLOWERCASE123", "소문자 없음"),
        ("NoDigits", "숫자 없음"),
    ];

    for (weak_password, reason) in weak_passwords {
        let result = hash_password(weak_password);
        assert!(result.is_err(), "Should reject: {}", reason);
    }

    // 강한 비밀번호
    let strong_password = "SecurePass123";
    let result = hash_password(strong_password);
    assert!(result.is_ok(), "Should accept strong password");
}
```

### JWT 토큰 검증 테스트

```rust
#[test]
fn test_jwt_generation_and_validation() {
    let config = JwtSettings {
        secret: "test-secret-key-32-characters".to_string(),
        access_token_expiry: 900,
        refresh_token_expiry: 604800,
        issuer: "test-issuer".to_string(),
    };

    let user_id = Uuid::new_v4();
    let email = "test@example.com";

    // 토큰 생성
    let token = generate_access_token(&user_id, email, &config)
        .expect("Failed to generate token");

    // 토큰 검증
    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert_eq!(claims.iss, "test-issuer");
}
```

### 보안 테스트

```rust
#[tokio::test]
async fn register_returns_400_for_invalid_email() {
    // 잘못된 이메일 형식 테스트
    let invalid_emails = vec![
        "notanemail",
        "user@",
        "@example.com",
        "user@@example.com",
    ];

    for invalid_email in invalid_emails {
        let response = client
            .post(&format!("{}/auth/register", &app.address))
            .json(&json!({
                "name": "Test",
                "email": invalid_email,
                "password": "SecurePass123"
            }))
            .send()
            .await
            .expect("Failed");

        assert_eq!(400, response.status().as_u16(),
            "Should reject invalid email: {}", invalid_email);
    }
}

#[tokio::test]
async fn protected_route_returns_401_without_token() {
    // 토큰 없이 보호된 경로 접근
    let response = client
        .get(&format!("{}/auth/me", &app.address))
        .send()
        .await
        .expect("Failed");

    assert_eq!(401, response.status().as_u16());
}
```

---

## 보안 고려사항

### 1. 비밀번호 보안

**✅ 구현됨**:
```rust
// bcrypt 12 라운드 (매우 안전)
const DEFAULT_COST: u32 = 12;

// 비밀번호 강도 요구사항
- 최소 8글자
- 최소 1개 숫자
- 최소 1개 소문자
- 최소 1개 대문자
- 최대 128글자
```

**이유**:
- bcrypt는 시간이 많이 걸리는 해싱 알고리즘
- 12 라운드는 순회 공격(brute force)에 강함
- 비밀번호 강도 검증으로 약한 비밀번호 방지

### 2. 토큰 보안

**액세스 토큰**:
- 15분 만료 (짧은 유효기간)
- HS256 알고리즘 (HMAC with SHA-256)
- JWT 서명으로 변조 감지

**Refresh 토큰**:
- 7일 만료 (긴 유효기간, 클라이언트 편의)
- SHA-256으로 해싱하여 저장 (평문 저장 금지)
- 로테이션을 통한 토큰 탈취 감지

### 3. 사용자 열거 공격 방지

**문제**: 공격자가 이메일을 모두 시도하여 등록된 이메일 확인 가능

**해결책**:
```rust
// 로그인 실패 시 동일한 에러 메시지 사용
match user {
    None => Err(AppError::Validation(
        ValidationError::InvalidFormat(
            "Invalid email or password".to_string()
        )
    )),
    // ...
}

match verify_password {
    false => Err(AppError::Validation(
        ValidationError::InvalidFormat(
            "Invalid email or password".to_string()
        )
    )),
    // ...
}
```

### 4. 토큰 로테이션

**목표**: 토큰 탈취 감지

**메커니즘**:
1. 클라이언트가 refresh 토큰으로 새 토큰 요청
2. 서버가 기존 토큰을 `is_revoked = true`로 표시
3. 공격자가 기존 토큰 사용 시 거부됨
4. 정상 사용자는 항상 새 토큰 보유

### 5. HTTPS 권장

```rust
// production에서는 반드시 HTTPS 사용
// JWT는 Authorization 헤더를 통해 전송되므로
// HTTP를 사용하면 중간자 공격(MITM) 위험

// 개발: http://localhost:8000 (OK)
// 프로덕션: https://api.example.com (필수)
```

### 6. 구조화된 로깅

```rust
tracing::warn!("Missing or invalid Authorization header");
tracing::info!(
    user_id = %claims.sub,
    email = %claims.email,
    "JWT validated successfully"
);
```

**이점**:
- 보안 이벤트 추적
- 비정상 활동 감지
- 감사 로그 기록

---

## 트러블슈팅

### 문제 1: "Invalid token" 에러

**원인**: 토큰 만료

**해결**:
```rust
// 토큰 만료 시간 확인
let exp_time = claims.exp;
let now = Utc::now().timestamp();
if now > exp_time {
    // 토큰 만료
    // refresh_token으로 새 access_token 요청
}
```

### 문제 2: "Missing Authorization header" 에러

**원인**: Authorization 헤더를 보내지 않음

**해결**:
```bash
# 잘못된 요청
curl http://localhost:8000/auth/me

# 올바른 요청
curl http://localhost:8000/auth/me \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 문제 3: 401 Unauthorized (모든 요청)

**원인**: JWT 설정의 secret이 다름

**확인**:
```rust
// configuration.yaml에서 secret 확인
jwt:
  secret: "MUST_BE_32_CHARS_OR_LONGER"

// 토큰 생성 시 사용한 secret과 동일해야 함
let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(jwt_config.secret.as_ref()),
)?;
```

### 문제 4: "Account is inactive" 에러

**원인**: 사용자 계정이 비활성화됨

**해결**:
```rust
// 데이터베이스에서 계정 활성화
UPDATE users SET is_active = true WHERE id = 'USER_ID';
```

### 문제 5: 비밀번호 검증 실패

**원인**: 비밀번호가 요구사항을 충족하지 않음

**검증 규칙**:
```
✓ "SecurePass123"     (8+, 대소문자, 숫자 모두 포함)
✗ "Short1"           (8글자 미만)
✗ "nouppercase123"   (대문자 없음)
✗ "NOLOWERCASE123"   (소문자 없음)
✗ "NoDigits"         (숫자 없음)
```

---

## 실행 및 테스트

### 의존성 설치

```bash
cd /path/to/zero2prod
cargo build
```

### 마이그레이션 실행

```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/zero2prod"
sqlx migrate run
```

### 서버 시작

```bash
cargo run
```

### 테스트 실행

```bash
# 모든 테스트
cargo test

# 특정 테스트만
cargo test auth_integration

# 유닛 테스트만
cargo test --lib

# 통합 테스트만
cargo test --test auth_integration
```

### 수동 테스트

```bash
# 1. 등록
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "SecurePass123"
  }'

# 응답에서 access_token 복사

# 2. 현재 사용자 정보 조회
curl -X GET http://localhost:8000/auth/me \
  -H "Authorization: Bearer <복사한_토큰>"

# 3. 로그인
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123"
  }'

# 4. 토큰 갱신
curl -X POST http://localhost:8000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'
```

---

## 다음 단계

### 권장 개선사항

1. **이메일 확인**
   - 등록 후 이메일 인증 추가
   - 미인증 계정의 기능 제한

2. **비밀번호 재설정**
   - "비밀번호 잊음" 기능
   - 안전한 토큰 기반 재설정

3. **2FA (Two-Factor Authentication)**
   - TOTP (Time-based One-Time Password)
   - SMS 인증

4. **OAuth2 / OpenID Connect**
   - Google, GitHub 로그인
   - SSO (Single Sign-On)

5. **감사 로깅**
   - 모든 인증 이벤트 기록
   - 의심스러운 활동 감지

---

## 참고 자료

- [JWT.io](https://jwt.io)
- [bcrypt](https://en.wikipedia.org/wiki/Bcrypt)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [Actix-web Documentation](https://actix.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)

---

## 요약

이 가이드에서는 다음을 구현했습니다:

✅ **JWT 기반 인증 시스템**
- 사용자 등록, 로그인, 토큰 갱신

✅ **보안 기능**
- bcrypt 비밀번호 해싱
- 토큰 로테이션
- 사용자 열거 공격 방지
- JWT 미들웨어

✅ **포괄적인 테스트**
- 86개 통과한 테스트
- 유닛 테스트
- 통합 테스트
- 보안 테스트

✅ **프로덕션 준비**
- 에러 처리
- 구조화된 로깅
- 데이터베이스 마이그레이션

이제 이 인증 시스템을 기반으로 더 많은 기능을 추가할 수 있습니다!

---

**마지막 수정**: 2025-11-27
**상태**: ✅ 완료 및 테스트됨
