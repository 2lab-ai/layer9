# 평면 아키텍처의 실패 모드

## 핵심 통찰
평면 구조는 n² 복잡성으로 필연적으로 붕괴한다. 스파게티는 요리법이 아니라 경고다.

## 실패 패턴
```
services/
├── user.js (imports order, payment, notification...)
├── order.js (imports user, payment, inventory...)
├── payment.js (imports user, order, notification...)
└── ... 각 파일이 다른 모든 파일 참조
```

## 증상
1. **순환 참조 지옥**
   - A → B → C → A
   - "일단 any로 하고 나중에..."

2. **변경 폭발**
   - 한 줄 수정 → 10개 파일 영향
   - 리팩토링 불가능

3. **인지 과부하**
   - "이 시스템 전체를 아는 사람?"
   - 6개월 후 자신도 모름

## 수학적 필연성
- n개 모듈 = n(n-1)/2 잠재적 연결
- 10개 모듈 = 45개 연결
- 100개 모듈 = 4,950개 연결
- 인간 작업 기억: 7±2

## 핵심 교훈
"평면 구조는 작은 프로젝트의 특권이다."

#failure #flat #complexity #spaghetti