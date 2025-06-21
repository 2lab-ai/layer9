# 계층 간 번역 계약

## 핵심 통찰
각 계층 경계는 명시적 번역 계약을 가진다. 이것이 복잡성을 제어하는 핵심이다.

## 번역의 본질
```
L1: User(id=123, email="...")
    ↓ [번역]
L2: UserService.getById(123)
    ↓ [번역]
L3: GET /users/123
```

## 왜 번역인가?
- 각 층은 다른 언어를 사용
- L1: 도메인 언어 (User, Order)
- L2: 서비스 언어 (create, update, delete)
- L3: HTTP 언어 (GET, POST, PUT)

## 명시적 계약
```typescript
interface L1ToL2Contract {
  // L1의 User를 L2가 이해하는 형태로
  toService(user: DomainUser): ServiceUser
  // L2의 결과를 L1이 이해하는 형태로
  toDomain(user: ServiceUser): DomainUser
}
```

## 핵심 원칙
"암묵적 이해는 버그의 온상이다. 모든 경계에 명시적 번역을."

#translation #contracts #boundaries #explicit