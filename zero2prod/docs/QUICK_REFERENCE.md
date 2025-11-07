# Docker & 데이터베이스 빠른 참고서 (Cheat Sheet)

자주 사용하는 명령어와 설정을 한눈에 볼 수 있는 빠른 참고서입니다.

---

## Docker 컨테이너 명령어

### 컨테이너 실행

```bash
# PostgreSQL 컨테이너 시작
docker run -d --name zero2prod-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=newsletter \
  -p 5432:5432 \
  postgres:latest
```

### 컨테이너 관리

```bash
# 실행 중인 컨테이너 확인
docker ps

# 모든 컨테이너 확인 (중지된 것 포함)
docker ps -a

# 컨테이너 중지
docker stop zero2prod-db

# 컨테이너 시작
docker start zero2prod-db

# 컨테이너 재시작
docker restart zero2prod-db

# 컨테이너 삭제
docker rm zero2prod-db

# 강제 삭제 (실행 중인 상태)
docker rm -f zero2prod-db

# 컨테이너 로그 확인
docker logs zero2prod-db

# 실시간 로그 확인
docker logs -f zero2prod-db
```

---

## PostgreSQL 명령어 (Docker 내부)

### 데이터베이스 작업

```bash
# 데이터베이스 접속
docker exec -it zero2prod-db psql -U postgres -d newsletter

# 테이블 목록 확인
docker exec zero2prod-db psql -U postgres -d newsletter -c "\dt"

# 테이블 스키마 확인
docker exec zero2prod-db psql -U postgres -d newsletter -c "\d subscriptions"

# 데이터 확인
docker exec zero2prod-db psql -U postgres -d newsletter -c "SELECT * FROM subscriptions;"

# 데이터 개수 확인
docker exec zero2prod-db psql -U postgres -d newsletter -c "SELECT COUNT(*) FROM subscriptions;"

# 데이터베이스 목록
docker exec zero2prod-db psql -U postgres -c "\l"

# 사용자 목록
docker exec zero2prod-db psql -U postgres -c "\du"
```

### SQL 실행

```bash
# 테이블 생성
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "CREATE TABLE subscriptions(id uuid NOT NULL, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, subscribed_at timestamptz NOT NULL, PRIMARY KEY (id));"

# 데이터 삽입
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "INSERT INTO subscriptions VALUES ('550e8400-e29b-41d4-a716-446655440000', 'user@example.com', 'John Doe', NOW());"

# 데이터 삭제
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "DELETE FROM subscriptions WHERE email = 'user@example.com';"

# 테이블 비우기
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "TRUNCATE subscriptions;"

# 테이블 삭제
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "DROP TABLE subscriptions;"
```

---

## 마이그레이션 설정

### 파일 생성

```bash
# migrations 디렉토리 생성
mkdir migrations

# 마이그레이션 파일 생성
cat > migrations/20251105000001_create_subscriptions.up.sql << 'EOF'
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL,
    PRIMARY KEY (id)
);
EOF
```

### .env 파일

```
DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter
```

### Cargo.toml 의존성

```toml
sqlx = {version = "0.6", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"]}
uuid = {version = "1", features = ["v4", "serde"]}
chrono = {version = "0.4", features = ["serde"]}
```

---

## Rust 프로젝트 명령어

### 프로젝트 관리

```bash
# 프로젝트 빌드
cargo build

# 릴리스 빌드
cargo build --release

# 프로젝트 실행
cargo run

# 테스트 실행
cargo test

# 테스트 실행 (상세 출력)
cargo test -- --nocapture

# 특정 테스트 실행
cargo test health_check_works

# 문법 검사
cargo check

# 의존성 업데이트
cargo update

# 프로젝트 정보 확인
cargo info
```

---

## HTTP 요청 테스트

### cURL 명령어

```bash
# GET 요청 (건강 확인)
curl http://localhost:8080/health_check

# GET 요청 (모든 구독자 조회)
curl http://localhost:8080/subscriptions

# GET 요청 (특정 구독자 조회)
curl http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000

# POST 요청 (새 구독자 추가)
curl -X POST http://localhost:8080/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=John Doe&email=john@example.com"

# PUT 요청 (구독자 정보 업데이트)
curl -X PUT http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name":"Jane Doe","email":"jane@example.com"}'

# DELETE 요청 (구독자 삭제)
curl -X DELETE http://localhost:8080/subscriptions/550e8400-e29b-41d4-a716-446655440000

# 요청 헤더 포함
curl -X POST http://localhost:8080/subscriptions \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com"}'

# 상세 응답 보기
curl -v http://localhost:8080/health_check

# 응답 헤더만 보기
curl -I http://localhost:8080/health_check
```

---

## Git 명령어

```bash
# 상태 확인
git status

# 변경사항 확인
git diff

# 스테이징
git add .

# 커밋
git commit -m "message"

# 로그 확인
git log --oneline

# 브랜치 확인
git branch

# 브랜치 생성
git checkout -b feature/new-feature

# 브랜치 전환
git checkout main

# 변경사항 푸시
git push origin main
```

---

## 환경 변수 설정

### Windows PowerShell

```powershell
# 환경 변수 설정
$env:DATABASE_URL = "postgres://postgres:password@localhost:5432/newsletter"

# 확인
$env:DATABASE_URL
```

### Linux / macOS

```bash
# 환경 변수 설정
export DATABASE_URL="postgres://postgres:password@localhost:5432/newsletter"

# 확인
echo $DATABASE_URL

# .bashrc에 영구 저장
echo 'export DATABASE_URL="postgres://postgres:password@localhost:5432/newsletter"' >> ~/.bashrc
```

---

## 유용한 팁

### 포트 상태 확인

```bash
# Windows: 포트 5432 사용 여부 확인
netstat -ano | findstr :5432

# Linux/macOS: 포트 5432 사용 여부 확인
lsof -i :5432
```

### 컨테이너 상태 확인

```bash
# 컨테이너 상세 정보
docker inspect zero2prod-db

# 컨테이너 통계
docker stats zero2prod-db

# 실행 중인 프로세스
docker top zero2prod-db
```

### 데이터 백업

```bash
# 데이터베이스 백업
docker exec zero2prod-db pg_dump -U postgres -d newsletter > backup.sql

# 데이터베이스 복원
cat backup.sql | docker exec -i zero2prod-db psql -U postgres -d newsletter
```

---

## 문제 해결 팁

### 컨테이너가 시작되지 않음

```bash
# 로그 확인
docker logs zero2prod-db

# 컨테이너 삭제 후 재실행
docker rm -f zero2prod-db
docker run -d --name zero2prod-db -p 5432:5432 postgres:latest
```

### 연결 거부 오류

```bash
# 1. 컨테이너 실행 확인
docker ps | grep zero2prod-db

# 2. 포트 확인
docker port zero2prod-db

# 3. 연결 테스트
docker exec zero2prod-db psql -U postgres -c "SELECT 1;"
```

### sqlx 컴파일 오류

```bash
# Cargo.lock 삭제 후 재빌드
rm Cargo.lock
cargo clean
cargo build
```

---

## 자주 하는 실수와 해결

| 문제 | 원인 | 해결 |
|------|------|------|
| `port 5432 already in use` | 다른 컨테이너가 포트 사용 | `docker rm -f [container]` |
| `connection refused` | 컨테이너 미실행 | `docker start zero2prod-db` |
| `DATABASE_URL not found` | 환경 변수 미설정 | `.env` 파일 확인 |
| `table not found` | 마이그레이션 미실행 | 마이그레이션 SQL 실행 |
| `unique constraint` | 중복 이메일 | 다른 이메일 사용 |
| `permission denied` | Docker 권한 없음 | `sudo usermod -aG docker $USER` |

---

## 한번에 모두 시작하는 스크립트

### Windows PowerShell

```powershell
# setup.ps1
Write-Host "🚀 Setting up database..."

# Docker 컨테이너 시작
docker run -d --name zero2prod-db `
  -e POSTGRES_USER=postgres `
  -e POSTGRES_PASSWORD=password `
  -e POSTGRES_DB=newsletter `
  -p 5432:5432 `
  postgres:latest

Write-Host "⏳ Waiting for database to start..."
Start-Sleep -Seconds 5

# 테이블 생성
docker exec zero2prod-db psql -U postgres -d newsletter -c `
  "CREATE TABLE subscriptions(id uuid NOT NULL, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, subscribed_at timestamptz NOT NULL, PRIMARY KEY (id));"

Write-Host "✓ Database setup complete!"
Write-Host "🎉 Ready to run: cargo run"
```

### Linux / macOS

```bash
#!/bin/bash
# setup.sh

echo "🚀 Setting up database..."

# Docker 컨테이너 시작
docker run -d --name zero2prod-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=newsletter \
  -p 5432:5432 \
  postgres:latest

echo "⏳ Waiting for database to start..."
sleep 5

# 테이블 생성
docker exec zero2prod-db psql -U postgres -d newsletter -c \
  "CREATE TABLE subscriptions(id uuid NOT NULL, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, subscribed_at timestamptz NOT NULL, PRIMARY KEY (id));"

echo "✓ Database setup complete!"
echo "🎉 Ready to run: cargo run"
```

---

## 자주 묻는 질문 (FAQ)

**Q: Docker Desktop이 필요한가?**
A: Windows/macOS에서는 필수입니다. Linux에서는 Docker CLI만 설치해도 됩니다.

**Q: DATABASE_URL을 어디에 저장하나?**
A: `.env` 파일에 저장합니다. `dotenv` crate를 사용하여 로드할 수 있습니다.

**Q: 마이그레이션을 롤백하려면?**
A: `.down.sql` 파일을 생성하여 역순으로 실행합니다.

**Q: 프로덕션에서도 Docker를 사용하나?**
A: 네, Docker Compose를 사용하여 멀티컨테이너 환경을 구성합니다.

**Q: 데이터가 컨테이너 삭제 시 사라지나?**
A: 네, 볼륨을 설정하지 않으면 삭제됩니다. `-v` 옵션으로 볼륨 설정 가능합니다.

---

**마지막 업데이트**: 2025년 11월 5일
