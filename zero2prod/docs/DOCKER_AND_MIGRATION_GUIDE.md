# Docker 및 데이터베이스 마이그레이션 가이드

이 문서는 Rust 웹 프로젝트에서 Docker를 사용하여 PostgreSQL 데이터베이스를 설정하고 마이그레이션을 실행하는 방법을 설명합니다.

---

## 목차

1. [개요](#개요)
2. [필수 도구 설치](#필수-도구-설치)
3. [Docker로 PostgreSQL 실행](#docker로-postgresql-실행)
4. [마이그레이션 설정](#마이그레이션-설정)
5. [데이터베이스 마이그레이션 실행](#데이터베이스-마이그레이션-실행)
6. [데이터베이스 연결 확인](#데이터베이스-연결-확인)
7. [문제 해결](#문제-해결)

---

## 개요

이 프로젝트는 다음과 같은 구조로 동작합니다:

- **웹 프레임워크**: Actix-web (Rust 비동기 웹 프레임워크)
- **데이터베이스**: PostgreSQL (Docker 컨테이너)
- **데이터베이스 클라이언트**: sqlx (Rust 라이브러리)
- **포트**: 8080 (애플리케이션), 5432 (데이터베이스)

```
┌─────────────────────┐
│   Rust Application  │
│   (Actix-web)       │
│   Port: 8080        │
└──────────┬──────────┘
           │
           │ TCP Connection
           │ localhost:5432
           ▼
┌─────────────────────┐
│  PostgreSQL (Docker)│
│  Container          │
│  Port: 5432         │
└─────────────────────┘
```

---

## 필수 도구 설치

### 1. Docker Desktop 설치

**Windows / macOS:**
- [Docker Desktop](https://www.docker.com/products/docker-desktop) 다운로드 및 설치
- 설치 후 시작 메뉴에서 "Docker Desktop" 실행

**Linux:**
```bash
sudo apt-get install docker.io
sudo systemctl start docker
```

### 2. PostgreSQL 클라이언트 설치

데이터베이스 명령어를 실행하기 위해 필요합니다.

**Windows (Chocolatey 사용):**
```bash
choco install postgresql
```

**macOS:**
```bash
brew install postgresql
```

**Linux:**
```bash
sudo apt-get install postgresql-client
```

### 3. Rust 및 Cargo 설치

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Docker로 PostgreSQL 실행

### 단계 1: Docker Desktop 시작

Windows/macOS 사용자는 Docker Desktop 애플리케이션을 실행해야 합니다.

### 단계 2: 기존 컨테이너 정리 (선택사항)

포트 5432가 이미 사용 중인 경우:

```bash
# 포트 5432를 사용하는 컨테이너 확인
docker ps -a | grep postgres

# 기존 컨테이너 중지 및 제거
docker rm -f postgres-typeorm  # 또는 해당 컨테이너명
```

### 단계 3: PostgreSQL 컨테이너 실행

```bash
docker run -d \
  --name zero2prod-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=newsletter \
  -p 5432:5432 \
  postgres:latest
```

**옵션 설명:**
- `-d`: 백그라운드에서 실행
- `--name zero2prod-db`: 컨테이너 이름
- `-e POSTGRES_USER`: PostgreSQL 사용자 이름
- `-e POSTGRES_PASSWORD`: PostgreSQL 비밀번호
- `-e POSTGRES_DB`: 기본 데이터베이스명
- `-p 5432:5432`: 포트 매핑 (호스트:컨테이너)

### 단계 4: 컨테이너 실행 확인

```bash
docker ps | grep zero2prod-db
```

출력 예:
```
cb6b89685d3e   postgres:latest   "docker-entrypoint..."   2 minutes ago   Up 2 minutes   0.0.0.0:5432->5432/tcp   zero2prod-db
```

---

## 마이그레이션 설정

### 단계 1: 마이그레이션 디렉토리 생성

프로젝트 루트 디렉토리에 마이그레이션 폴더를 생성합니다:

```bash
mkdir migrations
```

디렉토리 구조:
```
zero2prod/
├── src/
├── tests/
├── docs/
├── migrations/        ← 새로 생성
│   └── 마이그레이션 SQL 파일들
├── Cargo.toml
└── .env
```

### 단계 2: 마이그레이션 SQL 파일 생성

`migrations/20251105000001_create_subscriptions.up.sql` 파일 생성:

```sql
-- Create subscriptions table
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL,
    PRIMARY KEY (id)
);
```

**파일 이름 규칙:**
- 형식: `YYYYMMDDHHMMSS_설명.up.sql`
- 예: `20251105000001_create_subscriptions.up.sql`
- `up`: 마이그레이션 적용
- `down`: 마이그레이션 롤백 (선택사항)

### 단계 3: Cargo.toml에 sqlx 의존성 추가

```toml
[dependencies]
actix-web = "4"
tokio = {version = "1", features = ["macros", "rt-multi-thread"]}
serde = {version = "1", features = ["derive"]}
sqlx = {version = "0.6", features = ["postgres", "runtime-tokio-native-tls"]}
```

### 단계 4: .env 파일 생성

프로젝트 루트에 `.env` 파일 생성:

```
DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter
```

**형식:**
```
postgres://[사용자]:[비밀번호]@[호스트]:[포트]/[데이터베이스명]
```

---

## 데이터베이스 마이그레이션 실행

### 방법 1: Docker 명령어로 직접 실행 (간단함)

```bash
docker exec zero2prod-db psql -U postgres -d newsletter -c "CREATE TABLE subscriptions(id uuid NOT NULL, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, subscribed_at timestamptz NOT NULL, PRIMARY KEY (id));"
```

### 방법 2: SQL 파일로 실행

파일을 Docker 컨테이너로 복사하고 실행:

```bash
# Windows PowerShell
docker exec zero2prod-db psql -U postgres -d newsletter < C:\Users\user\Documents\Rust_server\zero2prod\migrations\20251105000001_create_subscriptions.up.sql
```

### 방법 3: sqlx-cli 사용 (권장)

sqlx-cli 설치:

```bash
cargo install sqlx-cli --features postgres
```

마이그레이션 실행:

```bash
sqlx migrate run --database-url "postgres://postgres:password@localhost:5432/newsletter"
```

---

## 데이터베이스 연결 확인

### 단계 1: 데이터베이스 연결 테스트

```bash
docker exec zero2prod-db psql -U postgres -d newsletter -c "SELECT 1;"
```

성공 시 출력:
```
 ?column?
----------
        1
(1 row)
```

### 단계 2: 테이블 목록 확인

```bash
docker exec zero2prod-db psql -U postgres -d newsletter -c "\dt"
```

성공 시 출력:
```
             List of relations
 Schema |     Name      | Type  |  Owner
--------+---------------+-------+----------
 public | subscriptions | table | postgres
(1 row)
```

### 단계 3: 테이블 스키마 확인

```bash
docker exec zero2prod-db psql -U postgres -d newsletter -c "\d subscriptions"
```

성공 시 출력:
```
                       Table "public.subscriptions"
    Column     |           Type           | Collation | Nullable | Default
---------------+--------------------------+-----------+----------+---------
 id            | uuid                     |           | not null |
 email         | text                     |           | not null |
 name          | text                     |           | not null |
 subscribed_at | timestamp with time zone |           | not null |
Indexes:
    "subscriptions_pkey" PRIMARY KEY, btree (id)
    "subscriptions_email_key" UNIQUE CONSTRAINT, btree (email)
```

---

## Rust 코드에서 데이터베이스 연결

### 단계 1: sqlx 연결 풀 생성

```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = "postgres://postgres:password@localhost:5432/newsletter";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    println!("Database connected!");
    Ok(())
}
```

### 단계 2: 데이터 삽입

```rust
use uuid::Uuid;
use chrono::Utc;

let id = Uuid::new_v4();
let email = "user@example.com";
let name = "John Doe";
let subscribed_at = Utc::now();

sqlx::query(
    "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"
)
.bind(id)
.bind(email)
.bind(name)
.bind(subscribed_at)
.execute(&pool)
.await?;
```

### 단계 3: 데이터 조회

```rust
let result = sqlx::query_as::<_, (String, String)>(
    "SELECT email, name FROM subscriptions WHERE id = $1"
)
.bind(id)
.fetch_one(&pool)
.await?;

println!("Email: {}, Name: {}", result.0, result.1);
```

---

## 문제 해결

### 문제 1: Docker Desktop이 실행되지 않음

**증상:**
```
error during connect: Head "http://%2F%2F.%2Fpipe%2FdockerDesktopLinuxEngine/_ping":
open //./pipe/dockerDesktopLinuxEngine: The system cannot find the file specified.
```

**해결 방법:**
1. Windows 시작 메뉴에서 "Docker Desktop" 검색
2. Docker Desktop 애플리케이션 실행
3. 시스템 트레이에서 Docker 아이콘이 나타날 때까지 대기

### 문제 2: 포트 5432가 이미 사용 중

**증상:**
```
error: port is already allocated
```

**해결 방법:**
```bash
# 포트를 사용하는 컨테이너 확인
docker ps -a | grep postgres

# 컨테이너 제거
docker rm -f [컨테이너명]

# 또는 다른 포트 사용
docker run -d --name zero2prod-db -p 5433:5432 postgres:latest
```

### 문제 3: PostgreSQL 컨테이너가 시작되지 않음

**증상:**
```
docker ps -a
→ Status: Exited (1)
```

**해결 방법:**
```bash
# 컨테이너 로그 확인
docker logs zero2prod-db

# 컨테이너 제거 후 다시 시작
docker rm zero2prod-db
docker run -d --name zero2prod-db -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=newsletter -p 5432:5432 postgres:latest
```

### 문제 4: 데이터베이스 연결 실패

**증상:**
```
could not connect to server: Connection refused
```

**해결 방법:**
1. Docker 컨테이너 실행 확인: `docker ps | grep zero2prod-db`
2. 포트 확인: `5432`가 올바른지 확인
3. 방화벽 설정 확인: PostgreSQL 포트가 차단되지 않았는지 확인

### 문제 5: sqlx-cli 설치 실패

**증상:**
```
error: failed to compile `sqlx-cli v0.8.6`
```

**해결 방법:**
```bash
# 대신 프로젝트의 Cargo.toml에서 sqlx를 의존성으로 추가
# 그 다음 Docker 명령어로 마이그레이션 실행
docker exec zero2prod-db psql -U postgres -d newsletter -c "..."
```

---

## 유용한 명령어 모음

```bash
# Docker 컨테이너 관리
docker ps                              # 실행 중인 컨테이너 목록
docker ps -a                           # 모든 컨테이너 목록
docker logs zero2prod-db               # 컨테이너 로그 확인
docker stop zero2prod-db               # 컨테이너 중지
docker start zero2prod-db              # 컨테이너 시작
docker rm zero2prod-db                 # 컨테이너 삭제

# PostgreSQL 명령어
docker exec zero2prod-db psql -U postgres -d newsletter -c "\dt"      # 테이블 목록
docker exec zero2prod-db psql -U postgres -d newsletter -c "\d subscriptions"  # 테이블 스키마
docker exec zero2prod-db psql -U postgres -d newsletter -c "SELECT * FROM subscriptions;"  # 데이터 조회

# Rust 프로젝트 명령어
cargo build                            # 프로젝트 빌드
cargo run                              # 프로젝트 실행
cargo test                             # 테스트 실행
```

---

## 다음 단계

1. **데이터 삽입**: 애플리케이션에서 구독자 정보를 저장하기
2. **데이터 검증**: 입력 데이터의 유효성 검사 (이메일, 이름 등)
3. **API 엔드포인트**: POST `/subscriptions` 엔드포인트 구현
4. **테스트**: 데이터베이스 통합 테스트 작성
5. **배포**: Docker Compose를 사용한 멀티컨테이너 배포

---

## 참고 자료

- [Docker 공식 문서](https://docs.docker.com/)
- [PostgreSQL 공식 문서](https://www.postgresql.org/docs/)
- [sqlx Rust 라이브러리](https://github.com/launchbadge/sqlx)
- [Actix-web 프레임워크](https://actix.rs/)

---

**마지막 업데이트**: 2025년 11월 5일
