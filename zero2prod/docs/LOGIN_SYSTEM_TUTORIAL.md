# 웹 로그인 시스템 구축 완벽 가이드

**작성일**: 2025-11-29  
**난이도**: 중급  
**소요 시간**: 2-3시간

---

## 📚 목차

1. [개요](#개요)
2. [프로젝트 구조](#프로젝트-구조)
3. [백엔드 구현 (Rust)](#백엔드-구현-rust)
4. [프론트엔드 구현 (HTML/CSS/JS)](#프론트엔드-구현-htmlcssjs)
5. [데이터베이스 설정](#데이터베이스-설정)
6. [보안 고려사항](#보안-고려사항)
7. [테스트 및 검증](#테스트-및-검증)
8. [트러블슈팅](#트러블슈팅)

---

## 개요

이 가이드에서는 **Rust 백엔드**와 **Vanilla JavaScript 프론트엔드**를 사용하여 완전한 로그인/회원가입 시스템을 구축하는 방법을 단계별로 설명합니다.

### 학습 목표

- ✅ JWT 기반 인증 시스템 이해
- ✅ Rust Actix-web 프레임워크 활용
- ✅ PostgreSQL 데이터베이스 연동
- ✅ 모던 웹 UI/UX 디자인
- ✅ 보안 모범 사례 적용

### 기술 스택

**백엔드:**
- Rust 1.70+
- Actix-web 4.0
- SQLx 0.6 (PostgreSQL)
- JWT (jsonwebtoken)
- Bcrypt (비밀번호 해싱)

**프론트엔드:**
- HTML5
- CSS3 (Vanilla)
- JavaScript (ES6+)
- Google Fonts (Inter)

**데이터베이스:**
- PostgreSQL 13+
- Docker (선택사항)

---

## 프로젝트 구조

```
zero2prod/
├── public/                    # 정적 파일 (프론트엔드)
│   ├── index.html            # 로그인/회원가입 페이지
│   ├── dashboard.html        # 대시보드
│   ├── styles.css            # 스타일시트
│   ├── app.js                # 로그인/회원가입 로직
│   └── dashboard.js          # 대시보드 로직
│
├── src/                       # Rust 소스 코드
│   ├── auth/                 # 인증 모듈
│   │   ├── mod.rs            # 모듈 정의
│   │   ├── jwt.rs            # JWT 토큰 생성/검증
│   │   ├── password.rs       # 비밀번호 해싱/검증
│   │   ├── claims.rs         # JWT Claims 구조체
│   │   └── refresh_token.rs  # 리프레시 토큰 관리
│   │
│   ├── routes/               # API 라우트
│   │   ├── mod.rs
│   │   ├── auth.rs           # 인증 관련 라우트
│   │   └── ...
│   │
│   ├── middleware/           # 미들웨어
│   │   └── jwt.rs            # JWT 인증 미들웨어
│   │
│   ├── startup.rs            # 서버 설정 및 라우팅
│   ├── configuration.rs      # 설정 관리
│   ├── lib.rs                # 라이브러리 루트
│   └── main.rs               # 애플리케이션 진입점
│
├── migrations/               # 데이터베이스 마이그레이션
│   ├── *_create_users_table.up.sql
│   └── *_create_refresh_tokens_table.up.sql
│
├── Cargo.toml                # Rust 의존성
└── configuration.yaml        # 애플리케이션 설정
```

---

## 백엔드 구현 (Rust)

### 1. 의존성 설정

`Cargo.toml`에 필요한 의존성을 추가합니다:

```toml
[dependencies]
actix-web = "4"
actix-files = "0.6"
tokio = {version = "1", features = ["macros", "rt-multi-thread"]}
serde = {version = "1", features = ["derive"]}
sqlx = {version = "0.6", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"]}
config = "0.13"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "fmt"] }
serde_json = "1.0"
jsonwebtoken = "9"
bcrypt = "0.15"
rand = "0.8"
uuid = { version = "1.18.1", features = ["v4"] }
chrono = "0.4.42"
```

**주요 크레이트 설명:**
- `actix-web`: 고성능 웹 프레임워크
- `actix-files`: 정적 파일 서빙
- `sqlx`: 비동기 PostgreSQL 드라이버
- `jsonwebtoken`: JWT 토큰 생성/검증
- `bcrypt`: 비밀번호 해싱

### 2. 데이터베이스 스키마

#### Users 테이블

```sql
CREATE TABLE users(
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
```

**필드 설명:**
- `id`: UUID 기본 키
- `email`: 사용자 이메일 (고유)
- `name`: 사용자 이름
- `password_hash`: Bcrypt 해시된 비밀번호
- `created_at`: 생성 시간
- `updated_at`: 수정 시간
- `is_active`: 계정 활성화 상태

#### Refresh Tokens 테이블

```sql
CREATE TABLE refresh_tokens(
    id uuid NOT NULL PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL,
    revoked_at timestamptz
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
```

### 3. 비밀번호 해싱 구현

`src/auth/password.rs`:

```rust
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::error::AppError;

/// 비밀번호를 Bcrypt로 해싱
pub fn hash_password(password: &str) -> Result<String, AppError> {
    // 비밀번호 강도 검증
    validate_password_strength(password)?;
    
    // Bcrypt 해싱 (기본 cost: 12)
    let hash = hash(password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
    
    Ok(hash)
}

/// 비밀번호 검증
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))
}

/// 비밀번호 강도 검증
fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    
    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(AppError::Validation(
            "Password must contain uppercase, lowercase, and digit".into()
        ));
    }
    
    Ok(())
}
```

**핵심 개념:**
- **Bcrypt**: 느린 해싱 알고리즘으로 무차별 대입 공격 방지
- **Cost Factor**: 해싱 반복 횟수 (기본값 12 = 2^12 반복)
- **Salt**: Bcrypt가 자동으로 랜덤 salt 생성

### 4. JWT 토큰 생성

`src/auth/jwt.rs`:

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::configuration::JwtSettings;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub email: String,    // User email
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub iss: String,      // Issuer
}

/// JWT 액세스 토큰 생성
pub fn generate_access_token(
    user_id: &Uuid,
    email: &str,
    config: &JwtSettings,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(config.access_token_expiry);
    
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration.timestamp(),
        iat: now.timestamp(),
        iss: config.issuer.clone(),
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;
    
    Ok(token)
}

/// JWT 토큰 검증
pub fn validate_access_token(
    token: &str,
    config: &JwtSettings,
) -> Result<Claims, AppError> {
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;
    
    Ok(token_data.claims)
}
```

**JWT 구조:**
```
Header.Payload.Signature
```

- **Header**: 알고리즘 정보 (HS256)
- **Payload**: Claims (사용자 정보)
- **Signature**: 무결성 검증용 서명

### 5. 회원가입 API 구현

`src/routes/auth.rs`:

```rust
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

pub async fn register(
    form: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    // 1. 입력 유효성 검사
    let email = is_valid_email(&form.email)?;
    let name = is_valid_name(&form.name)?;
    let password_hash = hash_password(&form.password)?;
    
    // 2. 사용자 생성
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
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
    
    // 4. 리프레시 토큰 저장
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;
    
    // 5. 응답 반환
    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}
```

**처리 흐름:**
1. 입력 데이터 유효성 검사
2. 비밀번호 해싱
3. 데이터베이스에 사용자 저장
4. JWT 토큰 생성
5. 리프레시 토큰 저장
6. 클라이언트에 토큰 반환

### 6. 로그인 API 구현

```rust
pub async fn login(
    form: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
    jwt_config: web::Data<JwtSettings>,
) -> Result<HttpResponse, AppError> {
    // 1. 이메일 검증
    let email = is_valid_email(&form.email)?;
    
    // 2. 사용자 조회
    let user = sqlx::query_as::<_, (Uuid, String, String, bool)>(
        "SELECT id, email, password_hash, is_active FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;
    
    let (user_id, user_email, password_hash, is_active) = user;
    
    // 3. 계정 활성화 확인
    if !is_active {
        return Err(AppError::Forbidden("Account is inactive".into()));
    }
    
    // 4. 비밀번호 검증
    let password_valid = verify_password(&form.password, &password_hash)?;
    if !password_valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }
    
    // 5. 토큰 생성 및 반환
    let access_token = generate_access_token(&user_id, &user_email, jwt_config.get_ref())?;
    let refresh_token = generate_refresh_token();
    
    save_refresh_token(
        pool.get_ref(),
        user_id,
        &refresh_token,
        jwt_config.refresh_token_expiry,
    )
    .await?;
    
    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    }))
}
```

**보안 고려사항:**
- ✅ 동일한 에러 메시지 ("Invalid credentials") 사용 → 사용자 열거 공격 방지
- ✅ 비활성 계정 체크
- ✅ 타이밍 공격 방지 (bcrypt의 constant-time 비교)

### 7. 서버 설정 및 라우팅

`src/startup.rs`:

```rust
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_files as fs;
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    jwt_config: JwtSettings,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let jwt_config_data = web::Data::new(jwt_config.clone());
    
    let server = HttpServer::new(move || {
        App::new()
            // 미들웨어
            .wrap(Logger::default())
            
            // 공유 상태
            .app_data(connection.clone())
            .app_data(jwt_config_data.clone())
            
            // Public 라우트 (인증 불필요)
            .route("/health_check", web::get().to(health_check))
            .route("/auth/register", web::post().to(register))
            .route("/auth/login", web::post().to(login))
            .route("/auth/refresh", web::post().to(refresh))
            
            // Protected 라우트 (JWT 인증 필요)
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware::new(jwt_config.clone()))
                    .route("/me", web::get().to(get_current_user))
            )
            .route("/auth/me", web::get().to(get_current_user))
            
            // 정적 파일 서빙 (마지막에 위치)
            .service(fs::Files::new("/", "./public").index_file("index.html"))
    })
    .listen(listener)?
    .run();
    
    Ok(server)
}
```

**라우팅 구조:**
- Public 라우트가 먼저 정의됨
- Protected 라우트는 JWT 미들웨어로 보호
- 정적 파일 서빙은 마지막에 위치 (다른 라우트를 덮어쓰지 않도록)

---

## 프론트엔드 구현 (HTML/CSS/JS)

### 1. HTML 구조

`public/index.html`:

```html
<!DOCTYPE html>
<html lang="ko">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>로그인 - Zero2Prod</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div class="container">
        <div class="auth-card">
            <div class="auth-header">
                <h1 class="logo">Zero2Prod</h1>
                <p class="subtitle">안전하고 빠른 인증 시스템</p>
            </div>

            <!-- 로그인 폼 -->
            <div id="loginForm" class="form-container active">
                <h2>로그인</h2>
                <form id="login-form">
                    <div class="form-group">
                        <label for="login-email">이메일</label>
                        <input type="email" id="login-email" name="email" 
                               placeholder="your@email.com" required>
                    </div>
                    <div class="form-group">
                        <label for="login-password">비밀번호</label>
                        <input type="password" id="login-password" name="password" 
                               placeholder="••••••••" required>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary" id="login-btn">
                            <span class="btn-text">로그인</span>
                            <span class="btn-loader" style="display: none;">
                                <span class="spinner"></span>
                            </span>
                        </button>
                    </div>
                    <div class="form-footer">
                        <p>계정이 없으신가요? <a href="#" id="show-register">회원가입</a></p>
                    </div>
                </form>
            </div>

            <!-- 회원가입 폼 -->
            <div id="registerForm" class="form-container">
                <h2>회원가입</h2>
                <form id="register-form">
                    <div class="form-group">
                        <label for="register-name">이름</label>
                        <input type="text" id="register-name" name="name" 
                               placeholder="홍길동" required>
                    </div>
                    <div class="form-group">
                        <label for="register-email">이메일</label>
                        <input type="email" id="register-email" name="email" 
                               placeholder="your@email.com" required>
                    </div>
                    <div class="form-group">
                        <label for="register-password">비밀번호</label>
                        <input type="password" id="register-password" name="password" 
                               placeholder="••••••••" required>
                        <small class="form-hint">8자 이상, 대문자, 소문자, 숫자 포함</small>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary" id="register-btn">
                            <span class="btn-text">회원가입</span>
                            <span class="btn-loader" style="display: none;">
                                <span class="spinner"></span>
                            </span>
                        </button>
                    </div>
                    <div class="form-footer">
                        <p>이미 계정이 있으신가요? <a href="#" id="show-login">로그인</a></p>
                    </div>
                </form>
            </div>

            <div id="alert-container"></div>
        </div>

        <!-- 배경 애니메이션 -->
        <div class="background-shapes">
            <div class="shape shape-1"></div>
            <div class="shape shape-2"></div>
            <div class="shape shape-3"></div>
        </div>
    </div>

    <script src="/app.js"></script>
</body>
</html>
```

**HTML 구조 설명:**
- **시맨틱 마크업**: 의미있는 HTML 태그 사용
- **접근성**: label과 input 연결, placeholder 제공
- **반응형**: viewport meta 태그
- **로딩 상태**: 버튼 내 스피너 포함

### 2. CSS 스타일링

`public/styles.css`:

```css
/* CSS 변수 정의 */
:root {
    /* 색상 팔레트 */
    --primary-color: #4F46E5;
    --primary-dark: #4338CA;
    --primary-light: #818CF8;
    --success-color: #10B981;
    --error-color: #EF4444;
    
    /* 중립 색상 */
    --white: #FFFFFF;
    --gray-50: #F9FAFB;
    --gray-100: #F3F4F6;
    --gray-600: #4B5563;
    --gray-900: #111827;
    
    /* 간격 */
    --spacing-sm: 0.75rem;
    --spacing-md: 1rem;
    --spacing-lg: 1.5rem;
    --spacing-xl: 2rem;
    
    /* 그림자 */
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
    
    /* 전환 */
    --transition-normal: 250ms ease-in-out;
}

/* 기본 스타일 */
body {
    font-family: 'Inter', sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* 글라스모피즘 카드 */
.auth-card {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(20px);
    border-radius: 1rem;
    box-shadow: var(--shadow-xl);
    padding: var(--spacing-xl);
    animation: slideUp 0.5s ease-out;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(30px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* 입력 필드 */
.form-group input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 2px solid #E5E7EB;
    border-radius: 0.5rem;
    transition: all var(--transition-normal);
}

.form-group input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.1);
}

/* 버튼 */
.btn-primary {
    background: linear-gradient(135deg, var(--primary-color), var(--primary-dark));
    color: var(--white);
    padding: 0.875rem 1.5rem;
    border: none;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all var(--transition-normal);
}

.btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
}

/* 로딩 스피너 */
.spinner {
    width: 20px;
    height: 20px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: var(--white);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* 배경 애니메이션 */
.shape {
    position: absolute;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    animation: float 20s infinite ease-in-out;
}

@keyframes float {
    0%, 100% { transform: translate(0, 0) scale(1); }
    33% { transform: translate(30px, -30px) scale(1.1); }
    66% { transform: translate(-20px, 20px) scale(0.9); }
}
```

**CSS 기법:**
- **CSS 변수**: 일관된 디자인 시스템
- **Flexbox**: 레이아웃 정렬
- **Transitions**: 부드러운 애니메이션
- **Backdrop-filter**: 글라스모피즘 효과
- **Keyframes**: 복잡한 애니메이션

### 3. JavaScript 로직

`public/app.js`:

```javascript
// API 설정
const API_BASE_URL = window.location.origin;
const API_ENDPOINTS = {
    register: `${API_BASE_URL}/auth/register`,
    login: `${API_BASE_URL}/auth/login`,
    refresh: `${API_BASE_URL}/auth/refresh`,
};

// 토큰 관리
const TokenManager = {
    getAccessToken() {
        return localStorage.getItem('access_token');
    },
    
    setTokens(accessToken, refreshToken) {
        localStorage.setItem('access_token', accessToken);
        localStorage.setItem('refresh_token', refreshToken);
    },
    
    clearTokens() {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
    },
};

// 알림 시스템
const AlertSystem = {
    show(message, type = 'info') {
        const container = document.getElementById('alert-container');
        const alert = document.createElement('div');
        alert.className = `alert alert-${type}`;
        alert.textContent = message;
        
        container.innerHTML = '';
        container.appendChild(alert);
        
        setTimeout(() => {
            alert.style.opacity = '0';
            setTimeout(() => alert.remove(), 300);
        }, 5000);
    },
    
    success(message) { this.show(message, 'success'); },
    error(message) { this.show(message, 'error'); },
};

// 유효성 검사
function validateEmail(email) {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

function validatePassword(password) {
    const hasMinLength = password.length >= 8;
    const hasUpperCase = /[A-Z]/.test(password);
    const hasLowerCase = /[a-z]/.test(password);
    const hasNumber = /\d/.test(password);
    
    return hasMinLength && hasUpperCase && hasLowerCase && hasNumber;
}

// 로그인 핸들러
async function handleLogin(event) {
    event.preventDefault();
    
    const form = event.target;
    const email = form.email.value.trim();
    const password = form.password.value;
    
    if (!validateEmail(email)) {
        AlertSystem.error('올바른 이메일 주소를 입력해주세요.');
        return;
    }
    
    try {
        const response = await fetch(API_ENDPOINTS.login, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        
        if (!response.ok) {
            throw new Error('로그인에 실패했습니다.');
        }
        
        const data = await response.json();
        TokenManager.setTokens(data.access_token, data.refresh_token);
        
        AlertSystem.success('로그인 성공!');
        setTimeout(() => {
            window.location.href = '/dashboard.html';
        }, 1000);
        
    } catch (error) {
        AlertSystem.error(error.message);
    }
}

// 회원가입 핸들러
async function handleRegister(event) {
    event.preventDefault();
    
    const form = event.target;
    const name = form.name.value.trim();
    const email = form.email.value.trim();
    const password = form.password.value;
    
    if (!validatePassword(password)) {
        AlertSystem.error('비밀번호는 8자 이상이며, 대문자, 소문자, 숫자를 포함해야 합니다.');
        return;
    }
    
    try {
        const response = await fetch(API_ENDPOINTS.register, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, email, password })
        });
        
        if (!response.ok) {
            throw new Error('회원가입에 실패했습니다.');
        }
        
        const data = await response.json();
        TokenManager.setTokens(data.access_token, data.refresh_token);
        
        AlertSystem.success('회원가입 성공!');
        setTimeout(() => {
            window.location.href = '/dashboard.html';
        }, 1000);
        
    } catch (error) {
        AlertSystem.error(error.message);
    }
}

// 초기화
document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('login-form').addEventListener('submit', handleLogin);
    document.getElementById('register-form').addEventListener('submit', handleRegister);
});
```

**JavaScript 패턴:**
- **모듈 패턴**: TokenManager, AlertSystem 객체
- **Async/Await**: 비동기 처리
- **Fetch API**: HTTP 요청
- **LocalStorage**: 토큰 저장
- **이벤트 리스너**: 폼 제출 처리

---

## 데이터베이스 설정

### Docker로 PostgreSQL 실행

```bash
# PostgreSQL 컨테이너 실행
docker run -d \
  --name zero2prod-db \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:13

# 데이터베이스 생성
docker exec -it zero2prod-db psql -U postgres -c "CREATE DATABASE zero2prod;"
```

### 마이그레이션 실행

```bash
# DATABASE_URL 환경 변수 설정
export DATABASE_URL="postgres://postgres:password@localhost:5432/zero2prod"

# sqlx-cli 설치 (한 번만)
cargo install sqlx-cli --no-default-features --features postgres

# 마이그레이션 실행
sqlx migrate run
```

### 마이그레이션 파일 생성

```bash
# 새 마이그레이션 생성
sqlx migrate add create_users_table
```

---

## 보안 고려사항

### 1. 비밀번호 보안

**✅ 해야 할 것:**
- Bcrypt로 비밀번호 해싱 (cost factor 12 이상)
- 비밀번호 강도 검증 (길이, 복잡도)
- 평문 비밀번호 절대 저장 금지

**❌ 하지 말아야 할 것:**
- MD5, SHA1 같은 빠른 해시 사용
- 비밀번호를 로그에 기록
- 비밀번호를 URL 파라미터로 전송

### 2. JWT 보안

**✅ 해야 할 것:**
- 짧은 만료 시간 (15분 권장)
- HTTPS로만 전송
- 서버 측 시크릿 키 안전하게 보관
- 리프레시 토큰 회전 (rotation)

**❌ 하지 말아야 할 것:**
- 민감한 정보를 JWT에 저장
- 토큰을 URL에 포함
- 클라이언트에서 토큰 검증

### 3. SQL 인젝션 방지

**✅ 올바른 방법 (파라미터화된 쿼리):**
```rust
sqlx::query("SELECT * FROM users WHERE email = $1")
    .bind(email)
    .fetch_one(&pool)
    .await?;
```

**❌ 잘못된 방법 (문자열 연결):**
```rust
// 절대 이렇게 하지 마세요!
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

### 4. CORS 설정

프로덕션 환경에서는 CORS를 적절히 설정:

```rust
use actix_cors::Cors;

let cors = Cors::default()
    .allowed_origin("https://yourdomain.com")
    .allowed_methods(vec!["GET", "POST"])
    .allowed_headers(vec!["Authorization", "Content-Type"])
    .max_age(3600);

App::new()
    .wrap(cors)
    // ... 나머지 설정
```

---

## 테스트 및 검증

### 1. 서버 시작

```bash
# 개발 모드로 실행
cargo run

# 서버가 http://localhost:8002 에서 실행됨
```

### 2. 회원가입 테스트

**브라우저에서:**
1. `http://localhost:8002` 접속
2. "회원가입" 클릭
3. 정보 입력:
   - 이름: "테스트유저"
   - 이메일: "test@example.com"
   - 비밀번호: "Test1234"
4. 제출 후 대시보드로 리다이렉션 확인

**curl로 테스트:**
```bash
curl -X POST http://localhost:8002/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "테스트유저",
    "email": "test@example.com",
    "password": "Test1234"
  }'
```

**예상 응답:**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 900
}
```

### 3. 로그인 테스트

```bash
curl -X POST http://localhost:8002/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "Test1234"
  }'
```

### 4. 인증된 엔드포인트 테스트

```bash
# 토큰을 변수에 저장
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

# /auth/me 엔드포인트 호출
curl http://localhost:8002/auth/me \
  -H "Authorization: Bearer $TOKEN"
```

**예상 응답:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "test@example.com",
  "name": "테스트유저",
  "created_at": "2025-11-29T10:00:00Z"
}
```

---

## 트러블슈팅

### 문제 1: 데이터베이스 연결 실패

**증상:**
```
Error: Database connection failed
```

**해결 방법:**
1. PostgreSQL이 실행 중인지 확인:
   ```bash
   docker ps | grep postgres
   ```

2. 연결 문자열 확인:
   ```yaml
   # configuration.yaml
   database:
     username: postgres
     password: password
     host: localhost
     port: 5432
     database_name: zero2prod
   ```

3. 데이터베이스 존재 확인:
   ```bash
   docker exec -it zero2prod-db psql -U postgres -l
   ```

### 문제 2: 정적 파일이 로드되지 않음

**증상:**
- `http://localhost:8002`에서 404 에러
- 또는 JSON 에러 응답

**해결 방법:**
1. `public` 디렉토리 존재 확인
2. `startup.rs`에서 정적 파일 서빙 설정 확인:
   ```rust
   .service(fs::Files::new("/", "./public").index_file("index.html"))
   ```
3. 정적 파일 서빙이 라우트 설정의 **마지막**에 위치하는지 확인

### 문제 3: CORS 에러

**증상:**
```
Access to fetch at 'http://localhost:8002/auth/login' from origin 'http://localhost:3000' 
has been blocked by CORS policy
```

**해결 방법:**
```rust
use actix_cors::Cors;

App::new()
    .wrap(
        Cors::permissive() // 개발 환경에서만 사용
    )
```

### 문제 4: JWT 토큰 만료

**증상:**
- 대시보드에서 401 Unauthorized 에러

**해결 방법:**
1. 자동 토큰 갱신 구현 (이미 `dashboard.js`에 포함됨)
2. 또는 다시 로그인

---

## 추가 학습 자료

### Rust 관련
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web 공식 문서](https://actix.rs/)
- [SQLx 가이드](https://github.com/launchbadge/sqlx)

### 웹 보안
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)
- [Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)

### 프론트엔드
- [MDN Web Docs](https://developer.mozilla.org/)
- [CSS Tricks](https://css-tricks.com/)
- [JavaScript.info](https://javascript.info/)

---

## 결론

이 가이드를 통해 다음을 배웠습니다:

1. ✅ **Rust 백엔드 개발**: Actix-web, SQLx 활용
2. ✅ **JWT 인증**: 토큰 생성, 검증, 갱신
3. ✅ **비밀번호 보안**: Bcrypt 해싱
4. ✅ **데이터베이스 설계**: PostgreSQL 스키마
5. ✅ **프론트엔드 개발**: HTML/CSS/JS
6. ✅ **보안 모범 사례**: SQL 인젝션 방지, CORS 등

### 다음 단계

프로젝트를 더 발전시키려면:

1. **이메일 인증**: 회원가입 시 이메일 확인
2. **비밀번호 재설정**: 이메일 기반 재설정
3. **2단계 인증**: TOTP 구현
4. **소셜 로그인**: OAuth2 통합
5. **Rate Limiting**: 무차별 대입 공격 방지
6. **로깅 및 모니터링**: 프로덕션 환경 준비

---

**작성자**: Antigravity AI  
**라이선스**: MIT  
**버전**: 1.0.0  
**최종 업데이트**: 2025-11-29
