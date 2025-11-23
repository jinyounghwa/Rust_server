# 뉴스레터 빠른 시작 가이드

## 5분 안에 시작하기

### 1단계: 모든 구독자에게 이메일 발송

```bash
curl -X POST http://localhost:8000/newsletters/send-all \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "뉴스레터 제목",
    "html_content": "<h1>환영합니다!</h1><p>이것이 뉴스레터입니다</p>"
  }'
```

**응답:**
```json
{
  "message": "Newsletter sent to all subscribers",
  "sent_count": 150,
  "failed_count": 2
}
```

### 2단계: 확인된 구독자에게만 이메일 발송

```bash
curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "확인된 사용자 뉴스레터",
    "html_content": "<h1>회원분께 감사합니다!</h1><p>특별 혜택을 드립니다</p>"
  }'
```

**응답:**
```json
{
  "message": "Newsletter sent to confirmed subscribers",
  "sent_count": 145,
  "failed_count": 0
}
```

## 주요 엔드포인트

| 엔드포인트 | 메서드 | 설명 |
|-----------|--------|------|
| `/newsletters/send-all` | POST | 모든 구독자에게 발송 |
| `/newsletters/send-confirmed` | POST | 확인된 구독자에게만 발송 |

## 요청 형식

### 필수 필드

| 필드 | 타입 | 설명 |
|------|------|------|
| `subject` | string | 이메일 제목 (필수, 비워둘 수 없음) |
| `html_content` | string | 이메일 본문 (필수, 비워둘 수 없음) |

### 예시 요청 본문

```json
{
  "subject": "2024년 11월 뉴스레터",
  "html_content": "<h1>환영합니다!</h1><p>새로운 소식을 전해드립니다.</p>"
}
```

## 응답 형식

### 성공 응답 (200 OK)

```json
{
  "message": "Newsletter sent to all subscribers",
  "sent_count": 150,
  "failed_count": 2
}
```

### 오류 응답 예시

**빈 필드 (400 Bad Request)**
```json
{
  "error_id": "abc123",
  "message": "subject is empty",
  "code": "VALIDATION_ERROR",
  "status": 400,
  "timestamp": "2024-11-23T10:30:00Z"
}
```

**데이터베이스 오류 (500 Internal Server Error)**
```json
{
  "error_id": "xyz789",
  "message": "Database error: connection pool exhausted",
  "code": "DATABASE_ERROR",
  "status": 500,
  "timestamp": "2024-11-23T10:30:00Z"
}
```

## 자주 묻는 질문

### Q: 모든 구독자와 확인된 구독자 중 어느 것을 사용해야 하나요?

**A:** 일반적으로는 `/newsletters/send-confirmed`를 사용하세요.
- **send-confirmed**: 사용자가 명시적으로 이메일을 확인한 사람들에게만 발송
- **send-all**: 모든 사용자에게 발송 (보류 중인 확인 포함)

### Q: 발송 실패가 있으면 어떻게 하나요?

**A:** 응답의 `failed_count`를 확인하세요.
- 로그를 확인하여 어떤 구독자가 실패했는지 확인
- 데이터베이스에서 구독자 데이터 검증
- 이메일 서비스 상태 확인

### Q: 얼마나 많은 구독자에게 발송할 수 있나요?

**A:** 이론상 무제한이지만, 실제로는:
- 이메일 서비스의 제한 확인
- 네트워크 타임아웃 설정 확인
- 대량 발송 시에는 배치 처리 권장

### Q: HTML 형식으로 뭘 포함시킬 수 있나요?

**A:** 표준 HTML을 모두 사용 가능합니다:
```html
<h1>제목</h1>
<p>본문</p>
<a href="https://example.com">링크</a>
<img src="https://example.com/image.jpg" />
<ul>
  <li>항목 1</li>
  <li>항목 2</li>
</ul>
```

### Q: 구독 취소 링크를 포함해야 하나요?

**A:** 네, 권장합니다. HTML 내용에 추가하세요:
```html
<hr />
<p><a href="https://your-domain.com/unsubscribe">구독 취소</a></p>
```

## 프로그래밍 언어별 예시

### JavaScript (Fetch API)

```javascript
async function sendNewsletter() {
  const response = await fetch('http://localhost:8000/newsletters/send-confirmed', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      subject: '뉴스레터',
      html_content: '<h1>안녕하세요!</h1>'
    })
  });

  const result = await response.json();
  console.log(`발송됨: ${result.sent_count}, 실패: ${result.failed_count}`);
}
```

### Python (requests)

```python
import requests

def send_newsletter():
    response = requests.post(
        'http://localhost:8000/newsletters/send-confirmed',
        json={
            'subject': '뉴스레터',
            'html_content': '<h1>안녕하세요!</h1>'
        }
    )
    result = response.json()
    print(f"발송됨: {result['sent_count']}, 실패: {result['failed_count']}")

send_newsletter()
```

### cURL

```bash
#!/bin/bash

curl -X POST http://localhost:8000/newsletters/send-confirmed \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "뉴스레터",
    "html_content": "<h1>안녕하세요!</h1>"
  }' \
  | jq '.sent_count'
```

### Node.js (axios)

```javascript
const axios = require('axios');

async function sendNewsletter() {
  try {
    const response = await axios.post(
      'http://localhost:8000/newsletters/send-confirmed',
      {
        subject: '뉴스레터',
        html_content: '<h1>안녕하세요!</h1>'
      }
    );
    console.log(`발송됨: ${response.data.sent_count}`);
  } catch (error) {
    console.error('오류:', error.response.data.message);
  }
}
```

## 검증 규칙

### Subject 필드
- ✅ 필수
- ✅ 비워둘 수 없음
- ✅ 최대 길이: 이메일 표준 (일반적으로 < 255자)

### HTML Content 필드
- ✅ 필수
- ✅ 비워둘 수 없음
- ✅ 유효한 HTML이어야 함
- ✅ 최대 길이: 이메일 서비스 제한

## 보안 팁

1. **HTTPS 사용**: 프로덕션 환경에서는 항상 HTTPS 사용
2. **인증 추가**: 필요시 API 토큰이나 API 키 추가
3. **레이트 제한**: 너무 많은 요청을 보내지 않기
4. **구독 취소 링크**: 항상 포함하기
5. **개인 정보**: HTML에 민감한 정보 포함하지 않기

## 문제 해결

### 발송되지 않음
```bash
# 1. 구독자 확인
psql -c "SELECT COUNT(*) FROM subscriptions WHERE status = 'confirmed';"

# 2. 이메일 서비스 확인
curl http://email-service:8001/health

# 3. 로그 확인
docker logs zero2prod | grep -i newsletter
```

### 일부 발송 실패
```bash
# 로그에서 실패한 구독자 찾기
docker logs zero2prod | grep "FAILURE" | grep "newsletter"

# 구독자 데이터 검증
psql -c "SELECT id, email, status FROM subscriptions WHERE status != 'pending';"
```

## 성능 팁

- **대량 발송**: 가능하면 `/send-confirmed` 사용 (더 빠름)
- **시간대 선택**: 피크 시간 외에 발송
- **배치 크기**: 대량 발송 시 시간 간격 두기
- **모니터링**: 발송 현황 실시간 모니터링

## 관련 문서

자세한 내용은 다음 문서를 참고하세요:
- [NEWSLETTER_FEATURE.md](./NEWSLETTER_FEATURE.md) - 상세 기능 설명
- [DATA_VALIDATION_GUIDE.md](./DATA_VALIDATION_GUIDE.md) - 검증 시스템
- [ERROR_HANDLING.md](./ERROR_HANDLING.md) - 오류 처리
