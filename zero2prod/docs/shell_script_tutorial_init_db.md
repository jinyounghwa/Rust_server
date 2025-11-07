# 쉘 스크립트 완벽 가이드: init_db.sh 분석

## 목차
1. [소개](#소개)
2. [쉘 스크립트란?](#쉘-스크립트란)
3. [Shebang과 기본 설정](#shebang과-기본-설정)
4. [명령어 존재 여부 확인](#명령어-존재-여부-확인)
5. [환경 변수 설정](#환경-변수-설정)
6. [Docker 컨테이너 관리](#docker-컨테이너-관리)
7. [데이터베이스 연결 대기](#데이터베이스-연결-대기)
8. [마이그레이션 실행](#마이그레이션-실행)
9. [전체 코드 분석](#전체-코드-분석)
10. [실습 및 응용](#실습-및-응용)

---

## 소개

이 문서는 `init_db.sh` 스크립트를 통해 쉘 스크립트의 기초부터 실전 활용까지 배우는 교재입니다. 이 스크립트는 PostgreSQL 데이터베이스를 Docker 컨테이너로 실행하고, 마이그레이션을 적용하는 자동화 스크립트입니다.

### 학습 목표
- 쉘 스크립트의 기본 문법 이해
- 조건문과 반복문 사용법
- 환경 변수 활용
- 외부 프로그램 제어 (Docker, PostgreSQL)
- 에러 처리와 디버깅

---

## 쉘 스크립트란?

### 정의
쉘 스크립트는 쉘(Shell)에서 실행할 수 있는 명령어들을 모아놓은 파일입니다. 반복적인 작업을 자동화하고, 여러 명령어를 순차적으로 실행할 수 있습니다.

### 장점
- **자동화**: 반복 작업을 스크립트로 만들어 실행
- **효율성**: 여러 명령어를 한 번에 실행
- **재사용성**: 한 번 작성하면 계속 사용 가능
- **일관성**: 항상 같은 방식으로 작업 수행

---

## Shebang과 기본 설정

### 코드 분석

```bash
#!/usr/bin/env bash

set -eo pipefail
```

### 상세 설명

#### 1. Shebang (`#!/usr/bin/env bash`)

**Shebang이란?**
- 스크립트 파일의 첫 번째 줄에 위치하는 특수한 주석
- `#!` 기호로 시작
- 어떤 인터프리터로 스크립트를 실행할지 지정

**`/usr/bin/env bash` 사용 이유:**
```bash
# 직접 경로 지정 (비추천)
#!/bin/bash

# env 사용 (추천)
#!/usr/bin/env bash
```

- `env`는 시스템의 `PATH`에서 `bash`를 찾아 실행
- 시스템마다 bash의 위치가 다를 수 있어 호환성이 좋음
- macOS: `/bin/bash`
- Linux: `/usr/bin/bash` 또는 `/bin/bash`
- `env`를 사용하면 자동으로 찾아줌

#### 2. set 명령어 (`set -eo pipefail`)

`set` 명령어는 쉘의 동작 방식을 변경합니다.

**옵션 상세:**

**`-e` (errexit):**
```bash
# -e 없이
command1  # 실패해도 계속 진행
command2  # 실행됨

# -e 있으면
command1  # 실패하면 여기서 스크립트 종료
command2  # 실행 안됨
```

**`-o pipefail`:**
```bash
# pipefail 없이
false | true  # 종료 코드: 0 (마지막 명령어 기준)

# pipefail 있으면
false | true  # 종료 코드: 1 (파이프 중 하나라도 실패하면)
```

**실전 예제:**
```bash
# 파이프라인에서 에러 발생 시나리오
cat nonexistent.txt | grep "text"

# pipefail 없으면: grep은 성공(0)으로 처리
# pipefail 있으면: cat 실패로 전체 실패
```

**왜 중요한가?**
- 에러를 조기에 발견하여 잘못된 상태로 진행되는 것을 방지
- 디버깅이 쉬워짐
- 프로덕션 환경에서 안전성 향상

---

## 명령어 존재 여부 확인

### 코드 분석

```bash
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "error: psql is not installed."
    echo >&2 "Install PostgreSQL client or use: brew install postgresql"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "error: sqlx is not installed."
    echo >&2 "Install with: cargo install --version='0.6' sqlx-cli --no-default-features --features postgres"
    exit 1
fi
```

### 상세 설명

#### 1. 조건문 구조

```bash
if 조건; then
    실행할_명령어
fi
```

#### 2. 부정 연산자 (`!`)

```bash
# 긍정 조건
if [ 조건 ]; then
    # 조건이 참이면 실행
fi

# 부정 조건
if ! [ 조건 ]; then
    # 조건이 거짓이면 실행
fi
```

#### 3. 명령어 존재 확인 (`command -v`)

**`command -v 명령어`:**
- 명령어가 존재하면 해당 경로를 출력
- 존재하지 않으면 아무것도 출력 안함

```bash
$ command -v psql
/usr/bin/psql

$ command -v nonexistent
# (출력 없음)
```

**왜 `which` 대신 `command -v`를 사용하나?**
```bash
# which: 외부 프로그램 (시스템마다 다름)
which psql

# command -v: 내장 명령어 (POSIX 표준)
command -v psql
```

#### 4. 실행 가능 여부 확인 (`-x`)

```bash
[ -x "파일경로" ]
```

**테스트 연산자들:**
| 연산자 | 설명 |
|--------|------|
| `-e` | 파일이 존재하는가? |
| `-f` | 일반 파일인가? |
| `-d` | 디렉토리인가? |
| `-x` | 실행 가능한가? |
| `-r` | 읽기 가능한가? |
| `-w` | 쓰기 가능한가? |

**예제:**
```bash
# 파일 존재 확인
if [ -e "/etc/hosts" ]; then
    echo "파일이 존재합니다"
fi

# 디렉토리 확인
if [ -d "/home/user" ]; then
    echo "디렉토리입니다"
fi
```

#### 5. 명령어 치환 (`$(...)`)

```bash
# 옛날 방식 (백틱)
result=`command -v psql`

# 최신 방식 (추천)
result=$(command -v psql)
```

**차이점:**
```bash
# 중첩 가능
outer=$(echo "Inner: $(echo "nested")")

# 백틱은 중첩이 어려움
outer=`echo "Inner: \`echo "nested"\`"`
```

#### 6. 표준 에러 출력 (`>&2`)

**파일 디스크립터:**
- `0`: 표준 입력 (stdin)
- `1`: 표준 출력 (stdout)
- `2`: 표준 에러 (stderr)

```bash
# 표준 출력으로 (기본)
echo "일반 메시지"

# 표준 에러로
echo >&2 "에러 메시지"
```

**왜 구분하나?**
```bash
# 리다이렉션 예제
./script.sh > output.txt

# 일반 echo: output.txt에 저장됨
# echo >&2: 화면에 출력됨 (파일에 저장 안됨)
```

**실전 활용:**
```bash
# 성공 메시지는 stdout
echo "작업 완료"

# 에러 메시지는 stderr
echo >&2 "에러 발생"

# 분리해서 저장
./script.sh > success.log 2> error.log
```

#### 7. 종료 코드 (`exit`)

```bash
exit 0  # 성공
exit 1  # 실패
exit 2  # 특정 에러 코드
```

**종료 코드 확인:**
```bash
./script.sh
echo $?  # 마지막 명령어의 종료 코드 출력
```

**관례:**
- `0`: 성공
- `1-125`: 일반 에러
- `126`: 실행 불가
- `127`: 명령어를 찾을 수 없음
- `128+n`: 시그널로 종료 (예: 130 = Ctrl+C)

---

## 환경 변수 설정

### 코드 분석

```bash
DB_USER="${DB_USER:=postgres}"
DB_PASSWORD="${DB_PASSWORD:=password}"
DB_HOST="${DB_HOST:=localhost}"
DB_PORT="${DB_PORT:=5432}"
DB_NAME="${DB_NAME:=newsletter}"
```

### 상세 설명

#### 1. 환경 변수란?

**정의:**
- 시스템이나 프로그램에서 사용하는 전역 변수
- 프로세스 간에 설정을 전달할 때 사용

**설정 방법:**
```bash
# 변수 선언
MY_VAR="value"

# 환경 변수로 export
export MY_VAR="value"

# 한 줄로
export MY_VAR="value"
```

#### 2. 기본값 설정 문법

**패턴들:**

| 문법 | 설명 | 예제 |
|------|------|------|
| `${VAR:-default}` | VAR이 없거나 빈 문자열이면 default 사용 (변수 변경 안함) | `echo ${NAME:-"Guest"}`|
| `${VAR:=default}` | VAR이 없거나 빈 문자열이면 default로 설정하고 사용 | `${PORT:=8080}` |
| `${VAR:?error}` | VAR이 없거나 빈 문자열이면 에러 메시지 출력하고 종료 | `${API_KEY:?"API key required"}` |
| `${VAR:+value}` | VAR이 있으면 value 사용 | `${DEBUG:+"-verbose"}` |

**실전 예제:**

```bash
# 1. :- 사용 (변수 변경 안함)
unset NAME
echo "Hello, ${NAME:-Guest}"  # "Hello, Guest"
echo "NAME is: $NAME"         # "NAME is: " (비어있음)

# 2. := 사용 (변수 설정)
unset PORT
echo "Port: ${PORT:=8080}"    # "Port: 8080"
echo "PORT is: $PORT"         # "PORT is: 8080" (설정됨)

# 3. :? 사용 (필수 변수)
unset API_KEY
echo ${API_KEY:?"API_KEY is required"}  # 에러 발생하고 종료

# 4. :+ 사용 (조건부 값)
DEBUG=true
FLAGS="${DEBUG:+-verbose}"    # FLAGS="-verbose"
```

#### 3. 우리 스크립트 분석

```bash
DB_USER="${DB_USER:=postgres}"
```

**동작 과정:**

1. `DB_USER` 환경 변수가 있는지 확인
2. 없으면 `postgres`로 설정
3. `DB_USER` 변수에 할당

**사용 시나리오:**

```bash
# 시나리오 1: 환경 변수 없음
./init_db.sh
# DB_USER="postgres" (기본값)

# 시나리오 2: 환경 변수 있음
DB_USER="myuser" ./init_db.sh
# DB_USER="myuser" (제공된 값)

# 시나리오 3: export로 설정
export DB_USER="admin"
./init_db.sh
# DB_USER="admin" (환경 변수 값)
```

#### 4. 실전 팁

**설정 파일 사용:**
```bash
# .env 파일
DB_USER=myuser
DB_PASSWORD=mypass
DB_HOST=192.168.1.100

# 스크립트에서 로드
if [ -f .env ]; then
    export $(cat .env | xargs)
fi
```

**검증 추가:**
```bash
# 포트 번호 검증
DB_PORT="${DB_PORT:=5432}"
if ! [[ "$DB_PORT" =~ ^[0-9]+$ ]]; then
    echo >&2 "Error: DB_PORT must be a number"
    exit 1
fi
```

---

## Docker 컨테이너 관리

### 코드 분석

```bash
# Docker 컨테이너가 이미 실행 중인지 확인
if ! docker ps --filter "name=zero2prod-db" --format '{{.Names}}' | grep -q "zero2prod-db"; then
    echo "Starting Docker PostgreSQL container..."
    docker run -d \
        --name zero2prod-db \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}:5432" \
        postgres:latest \
        postgres -N 1000
else
    echo "PostgreSQL container is already running."
fi
```

### 상세 설명

#### 1. Docker 명령어 기초

**`docker ps`:**
```bash
# 실행 중인 컨테이너 목록
docker ps

# 모든 컨테이너 (중지된 것 포함)
docker ps -a

# 특정 형식으로 출력
docker ps --format "{{.Names}}"
```

#### 2. 필터링 (`--filter`)

```bash
# 이름으로 필터링
docker ps --filter "name=zero2prod-db"

# 상태로 필터링
docker ps --filter "status=running"
docker ps --filter "status=exited"

# 레이블로 필터링
docker ps --filter "label=env=production"

# 여러 필터 조합
docker ps --filter "name=db" --filter "status=running"
```

#### 3. 출력 형식 (`--format`)

**템플릿 변수들:**
```bash
# 컨테이너 이름만
docker ps --format "{{.Names}}"

# ID와 이름
docker ps --format "{{.ID}}: {{.Names}}"

# 테이블 형식
docker ps --format "table {{.ID}}\t{{.Names}}\t{{.Status}}"
```

**사용 가능한 변수:**
| 변수 | 설명 |
|------|------|
| `.ID` | 컨테이너 ID |
| `.Names` | 컨테이너 이름 |
| `.Image` | 이미지 이름 |
| `.Status` | 상태 |
| `.Ports` | 포트 매핑 |
| `.CreatedAt` | 생성 시간 |

#### 4. grep과 조용한 모드 (`-q`)

```bash
# 일반 grep (매칭된 줄 출력)
echo "hello world" | grep "hello"
# 출력: hello world

# -q 옵션 (출력 없음, 종료 코드만)
echo "hello world" | grep -q "hello"
echo $?  # 0 (찾음)

echo "hello world" | grep -q "bye"
echo $?  # 1 (못 찾음)
```

**왜 -q를 사용하나?**
```bash
# -q 없이 (출력이 나옴)
if docker ps | grep "zero2prod-db"; then
    echo "Found"  # 매칭된 줄도 함께 출력됨
fi

# -q 사용 (깔끔)
if docker ps | grep -q "zero2prod-db"; then
    echo "Found"  # 이것만 출력됨
fi
```

#### 5. 파이프라인 이해

```bash
명령어1 | 명령어2 | 명령어3
```

**우리 스크립트의 파이프라인:**
```bash
docker ps --filter "name=zero2prod-db" --format '{{.Names}}' | grep -q "zero2prod-db"
│                                                              │
│                                                              └─ 출력에서 zero2prod-db 찾기
└─ zero2prod-db라는 이름의 컨테이너 찾기
```

#### 6. `docker run` 상세 분석

```bash
docker run -d \
    --name zero2prod-db \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}:5432" \
    postgres:latest \
    postgres -N 1000
```

**옵션 상세:**

**`-d` (detach):**
```bash
# -d 없이 (포그라운드)
docker run nginx
# 터미널이 블록됨, Ctrl+C로 종료

# -d 사용 (백그라운드)
docker run -d nginx
# 즉시 터미널로 돌아옴
```

**`--name`:**
```bash
# 이름 지정
docker run --name my-container nginx

# 이름 없이 (자동 생성)
docker run nginx
# 랜덤 이름: silly_einstein, brave_darwin 등
```

**`-e` (environment):**
```bash
# 환경 변수 설정
docker run -e "KEY=value" nginx

# 여러 개 설정
docker run \
    -e "USER=admin" \
    -e "PASS=secret" \
    nginx

# 파일에서 읽기
docker run --env-file .env nginx
```

**`-p` (port):**
```bash
# 포트 매핑: 호스트:컨테이너
docker run -p 8080:80 nginx
# 호스트의 8080 -> 컨테이너의 80

# 여러 포트
docker run -p 8080:80 -p 8443:443 nginx

# 호스트 포트 자동 할당
docker run -p 80 nginx
```

**백슬래시 (`\`):**
```bash
# 한 줄로 (가독성 나쁨)
docker run -d --name db -e USER=admin -e PASS=secret -p 5432:5432 postgres

# 여러 줄로 (가독성 좋음)
docker run -d \
    --name db \
    -e USER=admin \
    -e PASS=secret \
    -p 5432:5432 \
    postgres
```

**이미지와 명령어:**
```bash
docker run [옵션] 이미지 [명령어] [인수]
           ~~~~~~ ~~~~~~~~~~~ ~~~~~~~~~~~~~~~
            │        │              │
            │        │              └─ 컨테이너 내부에서 실행할 명령어
            │        └─ 사용할 Docker 이미지
            └─ Docker run 옵션
```

**우리 스크립트:**
```bash
postgres:latest \      # 이미지
postgres -N 1000       # 명령어와 인수
```

`postgres -N 1000`:
- PostgreSQL의 최대 연결 수를 1000으로 설정
- 기본값보다 높게 설정하여 부하 테스트 등에 대비

#### 7. if-else 구조

```bash
if 조건; then
    # 조건이 참일 때
else
    # 조건이 거짓일 때
fi
```

**elif 사용:**
```bash
if [ 조건1 ]; then
    # 조건1이 참
elif [ 조건2 ]; then
    # 조건2가 참
else
    # 모두 거짓
fi
```

---

## 데이터베이스 연결 대기

### 코드 분석

```bash
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo "Postgres is still unavailable - sleeping"
    sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT} - running migrations now!"
```

### 상세 설명

#### 1. export 명령어

```bash
# 일반 변수 (현재 쉘만)
MY_VAR="value"

# 환경 변수 (자식 프로세스도)
export MY_VAR="value"
```

**차이점:**
```bash
# test.sh
echo "MY_VAR: $MY_VAR"
echo "EXPORTED: $EXPORTED"

# 실행
MY_VAR="local" bash test.sh
# 출력: MY_VAR:  (비어있음)

export EXPORTED="global"
bash test.sh
# 출력: EXPORTED: global
```

**PGPASSWORD:**
- PostgreSQL 클라이언트가 자동으로 읽는 환경 변수
- 비밀번호를 매번 입력하지 않아도 됨

#### 2. until 반복문

**문법:**
```bash
until 조건; do
    명령어
done
```

**while과 비교:**
```bash
# while: 조건이 참인 동안 반복
while [ 조건이_참 ]; do
    echo "실행"
done

# until: 조건이 거짓인 동안 반복 (참이 될 때까지)
until [ 조건이_참 ]; do
    echo "실행"
done
```

**실전 예제:**
```bash
# 파일이 생성될 때까지 대기
until [ -f /tmp/ready ]; do
    echo "Waiting for file..."
    sleep 1
done

# 서비스가 응답할 때까지 대기
until curl -s http://localhost:8080/health; do
    echo "Service not ready..."
    sleep 2
done
```

#### 3. psql 명령어

```bash
psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'
```

**옵션 상세:**

| 옵션 | 긴 형식 | 설명 | 예제 |
|------|---------|------|------|
| `-h` | `--host` | 호스트 | `-h localhost` |
| `-U` | `--username` | 사용자명 | `-U postgres` |
| `-p` | `--port` | 포트 | `-p 5432` |
| `-d` | `--dbname` | 데이터베이스 | `-d mydb` |
| `-c` | `--command` | SQL 명령 실행 | `-c 'SELECT 1'` |

**`\q` 명령어:**
- PostgreSQL의 메타 명령어
- quit의 약자
- 연결을 테스트하고 즉시 종료

**다른 유용한 메타 명령어:**
```bash
# 데이터베이스 목록
psql -c '\l'

# 테이블 목록
psql -c '\dt'

# 연결 정보
psql -c '\conninfo'
```

#### 4. sleep 명령어

```bash
sleep 숫자[단위]
```

**단위:**
```bash
sleep 1      # 1초 (기본)
sleep 1s     # 1초
sleep 1m     # 1분
sleep 1h     # 1시간
sleep 0.5    # 0.5초 (시스템에 따라 지원)
```

**실전 활용:**
```bash
# 지수 백오프 (exponential backoff)
RETRY=0
until some_command; do
    RETRY=$((RETRY + 1))
    if [ $RETRY -gt 5 ]; then
        echo "Max retries reached"
        exit 1
    fi
    SLEEP_TIME=$((2 ** RETRY))
    echo "Retry $RETRY after ${SLEEP_TIME}s"
    sleep $SLEEP_TIME
done
```

#### 5. 연결 대기 패턴 분석

**왜 필요한가?**
```bash
# 문제 상황
docker run -d postgres        # 컨테이너 시작
psql -c 'SELECT 1'            # 즉시 연결 시도 -> 실패!
# PostgreSQL은 시작하는데 몇 초 걸림
```

**해결책:**
```bash
# 연결될 때까지 대기
until psql -c '\q' 2>/dev/null; do
    sleep 1
done
# PostgreSQL이 준비되면 다음 단계 진행
```

**개선된 버전:**
```bash
MAX_TRIES=30
TRY=0

until psql -c '\q' 2>/dev/null; do
    TRY=$((TRY + 1))
    if [ $TRY -gt $MAX_TRIES ]; then
        echo >&2 "Failed to connect after $MAX_TRIES attempts"
        exit 1
    fi
    echo "Waiting for PostgreSQL... ($TRY/$MAX_TRIES)"
    sleep 1
done
```

---

## 마이그레이션 실행

### 코드 분석

```bash
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL

sqlx database create
sqlx migrate run

echo >&2 "Postgres has been migrated, ready to go!"
```

### 상세 설명

#### 1. 데이터베이스 URL 구성

**PostgreSQL URL 형식:**
```
postgres://사용자:비밀번호@호스트:포트/데이터베이스
```

**구성 요소:**
```bash
postgres://        # 프로토콜 (postgresql:// 도 가능)
${DB_USER}:        # 사용자명
${DB_PASSWORD}@    # 비밀번호
${DB_HOST}:        # 호스트
${DB_PORT}/        # 포트
${DB_NAME}         # 데이터베이스명
```

**예제:**
```bash
# 로컬 개발
DATABASE_URL="postgres://postgres:password@localhost:5432/mydb"

# 원격 서버
DATABASE_URL="postgres://admin:secret@db.example.com:5432/production"

# Unix 소켓 사용
DATABASE_URL="postgres://user@/var/run/postgresql/mydb"

# SSL 옵션 추가
DATABASE_URL="postgres://user:pass@host:5432/db?sslmode=require"
```

#### 2. URL 인코딩

**특수 문자가 있는 경우:**
```bash
# 문제: 비밀번호에 특수문자 @, :, / 등
DB_PASSWORD="p@ss:w/rd"

# 해결: URL 인코딩
# @ -> %40
# : -> %3A
# / -> %2F
DB_PASSWORD_ENCODED="p%40ss%3Aw%2Frd"
```

**쉘 스크립트에서 인코딩:**
```bash
# jq 사용
encoded=$(echo -n "$DB_PASSWORD" | jq -sRr @uri)

# Python 사용
encoded=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$DB_PASSWORD'))")
```

#### 3. sqlx-cli 명령어

**설치:**
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

**주요 명령어:**

**`sqlx database create`:**
```bash
# DATABASE_URL의 데이터베이스 생성
sqlx database create

# SQL로 표현하면:
# CREATE DATABASE newsletter;
```

**`sqlx database drop`:**
```bash
# 데이터베이스 삭제
sqlx database drop

# 확인 없이 삭제
sqlx database drop -y
```

**`sqlx migrate run`:**
```bash
# migrations/ 디렉토리의 마이그레이션 실행
sqlx migrate run

# 특정 디렉토리 지정
sqlx migrate run --source custom_migrations/
```

**`sqlx migrate add`:**
```bash
# 새 마이그레이션 파일 생성
sqlx migrate add create_users_table
# 생성: migrations/20231105123456_create_users_table.sql
```

**`sqlx migrate revert`:**
```bash
# 마지막 마이그레이션 되돌리기
sqlx migrate revert
```

#### 4. 마이그레이션 파일 구조

**디렉토리 구조:**
```
zero2prod/
├── migrations/
│   ├── 20231105000000_create_subscriptions.sql
│   ├── 20231106000000_add_status_to_subscriptions.sql
│   └── 20231107000000_create_indexes.sql
└── src/
```

**마이그레이션 파일 예제:**
```sql
-- migrations/20231105000000_create_subscriptions.sql
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);

-- 인덱스 추가
CREATE INDEX idx_subscriptions_email ON subscriptions(email);

-- 코멘트 추가
COMMENT ON TABLE subscriptions IS '구독자 정보를 저장하는 테이블';
```

**되돌리기 마이그레이션 (down migration):**
```sql
-- migrations/20231105000000_create_subscriptions.down.sql
DROP TABLE IF EXISTS subscriptions;
```

#### 5. 환경별 설정

**개발 환경:**
```bash
# .env.development
DATABASE_URL=postgres://dev:devpass@localhost:5432/dev_db
```

**테스트 환경:**
```bash
# .env.test
DATABASE_URL=postgres://test:testpass@localhost:5433/test_db
```

**프로덕션 환경:**
```bash
# .env.production (절대 커밋하지 말 것!)
DATABASE_URL=postgres://prod:$PROD_PASS@prod-db.internal:5432/prod_db
```

**스크립트에서 환경 선택:**
```bash
# 환경 선택
ENV=${ENV:-development}

# 해당 환경 파일 로드
if [ -f ".env.$ENV" ]; then
    export $(cat ".env.$ENV" | grep -v '^#' | xargs)
fi

# 마이그레이션 실행
sqlx migrate run
```

---

## 전체 코드 분석

### 전체 흐름 다이어그램

```
시작
  │
  ├─ Shebang 설정 (#!/usr/bin/env bash)
  │
  ├─ 에러 처리 설정 (set -eo pipefail)
  │
  ├─ 필수 프로그램 확인
  │   ├─ psql 존재 확인
  │   └─ sqlx 존재 확인
  │
  ├─ 환경 변수 설정
  │   ├─ DB_USER (기본: postgres)
  │   ├─ DB_PASSWORD (기본: password)
  │   ├─ DB_HOST (기본: localhost)
  │   ├─ DB_PORT (기본: 5432)
  │   └─ DB_NAME (기본: newsletter)
  │
  ├─ Docker 컨테이너 관리
  │   ├─ 실행 중인 컨테이너 확인
  │   │   ├─ 있으면: 메시지 출력
  │   │   └─ 없으면: PostgreSQL 컨테이너 시작
  │
  ├─ 데이터베이스 연결 대기
  │   └─ psql 연결 성공할 때까지 반복
  │
  ├─ 마이그레이션 실행
  │   ├─ DATABASE_URL 설정
  │   ├─ 데이터베이스 생성
  │   └─ 마이그레이션 실행
  │
  └─ 완료 메시지
```

### 전체 코드 주석 버전

```bash
#!/usr/bin/env bash
# Shebang: bash를 인터프리터로 사용
# /usr/bin/env: 시스템의 PATH에서 bash를 찾아 실행 (이식성 향상)

set -eo pipefail
# set: 쉘 옵션 설정
# -e: 명령어가 실패하면 스크립트 즉시 종료 (에러 전파)
# -o pipefail: 파이프라인에서 하나라도 실패하면 전체 실패로 처리

# ========================================
# 필수 프로그램 존재 확인
# ========================================

if ! [ -x "$(command -v psql)" ]; then
    # ! : 조건 부정 (없으면 참)
    # -x : 실행 가능한 파일인지 확인
    # command -v : 명령어 경로 출력 (POSIX 표준)
    # $(...) : 명령어 치환 (명령어 결과를 문자열로)

    echo >&2 "error: psql is not installed."
    # >&2 : 표준 에러(stderr)로 출력
    # 에러 메시지는 stdout이 아닌 stderr로 보내는 것이 관례

    echo >&2 "Install PostgreSQL client or use: brew install postgresql"
    exit 1
    # exit 1: 에러 코드 1로 종료 (0이 아닌 값은 실패를 의미)
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "error: sqlx is not installed."
    echo >&2 "Install with: cargo install --version='0.6' sqlx-cli --no-default-features --features postgres"
    exit 1
fi

# ========================================
# 환경 변수 설정 (기본값 포함)
# ========================================

DB_USER="${DB_USER:=postgres}"
# ${VAR:=default}: VAR이 없거나 빈 문자열이면 default로 설정
# DB_USER가 이미 설정되어 있으면 그 값을 사용
# 없으면 "postgres"로 설정

DB_PASSWORD="${DB_PASSWORD:=password}"
DB_HOST="${DB_HOST:=localhost}"
DB_PORT="${DB_PORT:=5432}"
DB_NAME="${DB_NAME:=newsletter}"

# ========================================
# Docker 컨테이너 확인 및 시작
# ========================================

if ! docker ps --filter "name=zero2prod-db" --format '{{.Names}}' | grep -q "zero2prod-db"; then
    # docker ps: 실행 중인 컨테이너 목록
    # --filter: 필터 조건 (이름이 zero2prod-db)
    # --format: 출력 형식 (컨테이너 이름만)
    # | grep -q: 파이프로 전달하여 조용히 검색 (-q: 출력 없음)
    # ! : 조건 부정 (찾지 못하면 참)

    echo "Starting Docker PostgreSQL container..."
    docker run -d \
        # -d: detached mode (백그라운드 실행)
        # \: 줄 연속 (여러 줄로 명령어 작성)

        --name zero2prod-db \
        # 컨테이너 이름 지정

        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        # -e: 환경 변수 설정
        # PostgreSQL 이미지가 읽는 환경 변수들

        -p "${DB_PORT}:5432" \
        # -p: 포트 매핑 (호스트:컨테이너)
        # 호스트의 DB_PORT를 컨테이너의 5432로 연결

        postgres:latest \
        # 사용할 이미지 (Docker Hub의 공식 PostgreSQL 이미지)

        postgres -N 1000
        # 컨테이너 내부에서 실행할 명령어
        # postgres: PostgreSQL 서버 시작
        # -N 1000: 최대 연결 수 1000으로 설정
else
    echo "PostgreSQL container is already running."
fi

# ========================================
# 데이터베이스 연결 대기
# ========================================

export PGPASSWORD="${DB_PASSWORD}"
# export: 환경 변수로 설정 (자식 프로세스도 접근 가능)
# PGPASSWORD: psql이 자동으로 읽는 환경 변수

until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    # until: 조건이 참이 될 때까지 반복
    # psql: PostgreSQL 클라이언트
    #   -h: 호스트
    #   -U: 사용자
    #   -p: 포트
    #   -d: 데이터베이스 (postgres는 기본 DB)
    #   -c '\q': 명령 실행 후 종료 (\q는 quit)

    echo "Postgres is still unavailable - sleeping"
    sleep 1
    # 1초 대기 후 다시 시도
done

echo >&2 "Postgres is up and running on port ${DB_PORT} - running migrations now!"

# ========================================
# 마이그레이션 실행
# ========================================

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
# PostgreSQL 연결 URL 생성
# 형식: postgres://사용자:비밀번호@호스트:포트/데이터베이스

export DATABASE_URL
# sqlx가 읽을 수 있도록 환경 변수로 export

sqlx database create
# DATABASE_URL의 데이터베이스 생성
# 이미 있으면 무시

sqlx migrate run
# migrations/ 디렉토리의 SQL 파일들을 순서대로 실행
# 파일명의 타임스탬프 순서로 정렬되어 실행

echo >&2 "Postgres has been migrated, ready to go!"
# 완료 메시지
```

---

## 실습 및 응용

### 실습 1: 스크립트 실행하기

**기본 실행:**
```bash
# 실행 권한 부여
chmod +x init_db.sh

# 기본 설정으로 실행
./init_db.sh
```

**커스텀 설정으로 실행:**
```bash
# 환경 변수 지정
DB_USER="admin" DB_PASSWORD="secret" ./init_db.sh

# 또는 export 사용
export DB_USER="admin"
export DB_PASSWORD="secret"
export DB_PORT="5433"
./init_db.sh
```

**환경 파일 사용:**
```bash
# .env 파일 생성
cat > .env << 'EOF'
DB_USER=myuser
DB_PASSWORD=mypassword
DB_NAME=myapp
EOF

# 환경 변수 로드 후 실행
set -a
source .env
set +a
./init_db.sh
```

### 실습 2: 디버깅

**디버그 모드 활성화:**
```bash
# set -x 추가
#!/usr/bin/env bash
set -euxo pipefail
# u: 정의되지 않은 변수 사용 시 에러
# x: 실행되는 명령어 출력 (디버깅용)
```

**특정 부분만 디버깅:**
```bash
# 디버그 시작
set -x
docker ps --filter "name=zero2prod-db"
set +x
# 디버그 종료
```

**로그 파일에 저장:**
```bash
# 실행하면서 로그 저장
./init_db.sh 2>&1 | tee init_db.log
# 2>&1: stderr를 stdout으로 리다이렉션
# tee: 화면 출력과 파일 저장 동시에
```

### 실습 3: 에러 처리 강화

**타임아웃 추가:**
```bash
# 연결 대기에 타임아웃 추가
MAX_WAIT=30
ELAPSED=0

until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    if [ $ELAPSED -ge $MAX_WAIT ]; then
        echo >&2 "Timeout: PostgreSQL did not start within ${MAX_WAIT}s"
        exit 1
    fi
    echo "Waiting for PostgreSQL... (${ELAPSED}/${MAX_WAIT}s)"
    sleep 1
    ELAPSED=$((ELAPSED + 1))
done
```

**에러 처리 함수:**
```bash
# 에러 핸들러 정의
error_exit() {
    echo >&2 "ERROR: $1"
    # 클린업 작업
    docker stop zero2prod-db 2>/dev/null || true
    exit "${2:-1}"
}

# 사용
if ! [ -x "$(command -v psql)" ]; then
    error_exit "psql is not installed" 1
fi
```

### 실습 4: 스크립트 확장

**헬스체크 함수 추가:**
```bash
check_postgres_health() {
    local host=$1
    local port=$2
    local user=$3

    if psql -h "$host" -p "$port" -U "$user" -d "postgres" -c "SELECT 1" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# 사용
if check_postgres_health "${DB_HOST}" "${DB_PORT}" "${DB_USER}"; then
    echo "PostgreSQL is healthy"
else
    echo "PostgreSQL health check failed"
fi
```

**클린업 함수 추가:**
```bash
cleanup() {
    echo "Cleaning up..."
    docker stop zero2prod-db 2>/dev/null || true
    docker rm zero2prod-db 2>/dev/null || true
}

# 스크립트 종료 시 자동 실행
trap cleanup EXIT

# 또는 명령줄 옵션으로
if [ "${1:-}" = "clean" ]; then
    cleanup
    exit 0
fi
```

**멀티 환경 지원:**
```bash
ENV="${ENV:-development}"

case "$ENV" in
    development)
        DB_PORT=5432
        ;;
    test)
        DB_PORT=5433
        DB_NAME="test_db"
        ;;
    production)
        DB_PORT=5432
        DB_NAME="prod_db"
        if [ -z "$DB_PASSWORD" ]; then
            error_exit "DB_PASSWORD must be set in production"
        fi
        ;;
    *)
        error_exit "Unknown environment: $ENV"
        ;;
esac
```

### 실습 5: 마이그레이션 관리

**마이그레이션 상태 확인:**
```bash
# 적용된 마이그레이션 확인
psql $DATABASE_URL -c "SELECT * FROM _sqlx_migrations ORDER BY version;"
```

**새 마이그레이션 생성:**
```bash
# 스크립트에 추가
create_migration() {
    local name=$1
    if [ -z "$name" ]; then
        echo "Usage: $0 create <migration_name>"
        exit 1
    fi

    sqlx migrate add "$name"
    echo "Created migration: $name"
}

# 사용
if [ "${1:-}" = "create" ]; then
    create_migration "$2"
    exit 0
fi
```

**마이그레이션 롤백:**
```bash
rollback() {
    echo "Rolling back last migration..."
    sqlx migrate revert
    echo "Rollback complete"
}

# 사용
if [ "${1:-}" = "rollback" ]; then
    rollback
    exit 0
fi
```

### 실습 6: 통합 스크립트

**완전한 데이터베이스 관리 스크립트:**
```bash
#!/usr/bin/env bash
set -eo pipefail

# 사용법 출력
usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

Commands:
    init        Initialize database (default)
    clean       Stop and remove container
    reset       Clean and reinitialize
    migrate     Run migrations only
    rollback    Rollback last migration
    create      Create new migration

Options:
    -h, --help  Show this help message

Environment variables:
    DB_USER     Database user (default: postgres)
    DB_PASSWORD Database password (default: password)
    DB_HOST     Database host (default: localhost)
    DB_PORT     Database port (default: 5432)
    DB_NAME     Database name (default: newsletter)

Examples:
    $0 init
    $0 clean
    DB_USER=admin $0 init
    $0 create add_users_table
EOF
}

# 명령어 처리
COMMAND="${1:-init}"

case "$COMMAND" in
    init)
        # 기존 init_db.sh 로직
        ;;
    clean)
        cleanup
        ;;
    reset)
        cleanup
        # 재초기화
        ;;
    migrate)
        sqlx migrate run
        ;;
    rollback)
        rollback
        ;;
    create)
        create_migration "$2"
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
```

---

## 추가 학습 자료

### 1. 쉘 스크립트 베스트 프랙티스

**항상 쿼트 사용:**
```bash
# 나쁜 예
if [ $var = "value" ]; then

# 좋은 예
if [ "$var" = "value" ]; then
```

**${var}보다 "${var}" 선호:**
```bash
# 공백이 있는 경우
FILE="my file.txt"
cat $FILE        # 에러: cat my file.txt
cat "$FILE"      # 정상: cat "my file.txt"
```

**ShellCheck 사용:**
```bash
# ShellCheck 설치
brew install shellcheck  # macOS
apt install shellcheck   # Ubuntu

# 스크립트 검사
shellcheck init_db.sh
```

### 2. 유용한 쉘 스크립트 패턴

**스크립트 디렉토리 찾기:**
```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
```

**임시 디렉토리 사용:**
```bash
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# 임시 디렉토리 사용
echo "test" > "$TMPDIR/file.txt"
```

**배열 사용:**
```bash
# 배열 선언
DATABASES=("db1" "db2" "db3")

# 반복
for db in "${DATABASES[@]}"; do
    echo "Processing $db"
done
```

### 3. 참고 링크

- [Bash Guide for Beginners](https://tldp.org/LDP/Bash-Beginners-Guide/html/)
- [Advanced Bash-Scripting Guide](https://tldp.org/LDP/abs/html/)
- [ShellCheck](https://www.shellcheck.net/)
- [Explainshell](https://explainshell.com/)
- [Docker Documentation](https://docs.docker.com/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [sqlx Documentation](https://github.com/launchbadge/sqlx)

---

## 연습 문제

### 문제 1: 기본 수정
스크립트를 수정하여 컨테이너 이름을 환경 변수로 설정할 수 있게 하세요.

<details>
<summary>힌트</summary>

```bash
CONTAINER_NAME="${CONTAINER_NAME:=zero2prod-db}"
```
</details>

### 문제 2: 로깅 추가
모든 작업을 타임스탬프와 함께 로그 파일에 기록하세요.

<details>
<summary>힌트</summary>

```bash
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a init_db.log
}

log "Starting database initialization..."
```
</details>

### 문제 3: 백업 기능
데이터베이스 초기화 전에 자동으로 백업을 생성하세요.

<details>
<summary>힌트</summary>

```bash
backup() {
    local backup_file="backup_$(date +%Y%m%d_%H%M%S).sql"
    pg_dump $DATABASE_URL > "$backup_file"
    echo "Backup created: $backup_file"
}
```
</details>

### 문제 4: 멀티 데이터베이스
여러 데이터베이스를 동시에 초기화하는 스크립트를 작성하세요.

<details>
<summary>힌트</summary>

```bash
DATABASES=("db1" "db2" "db3")

for db in "${DATABASES[@]}"; do
    DB_NAME="$db" ./init_db.sh &
done

wait
echo "All databases initialized"
```
</details>

---

## 결론

이 교재를 통해 `init_db.sh` 스크립트의 모든 부분을 상세히 분석했습니다. 배운 내용:

1. **기초 문법**: Shebang, set 옵션, 변수, 조건문, 반복문
2. **명령어**: command, docker, psql, sqlx
3. **패턴**: 에러 처리, 연결 대기, 환경 변수 관리
4. **실전**: 디버깅, 확장, 베스트 프랙티스

이제 여러분만의 자동화 스크립트를 작성할 준비가 되었습니다!

---

**질문이나 피드백이 있으시면 언제든지 문의해주세요.**
