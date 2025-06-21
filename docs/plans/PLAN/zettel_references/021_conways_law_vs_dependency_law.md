# Conway's Law와 Dependency Law의 직교성

## 핵심 통찰
프로젝트 구조와 소스코드 구조는 완전히 다른 차원이다. X축과 Y축처럼 직교한다.

## Conway's Law (WHO - 누가)
"시스템을 설계하는 조직은 그 조직의 의사소통 구조를 따르는 설계를 만들어낸다."
- Frontend 팀 → frontend/ 폴더
- Backend 팀 → backend/ 폴더  
- DevOps 팀 → infra/ 폴더

## Dependency Law (HOW - 어떻게)
"정보는 한 방향으로만 흐른다. 상위 계층은 하위 계층에 의존하지만, 그 반대는 불가능하다."
- L3 → L2 → L1
- 역방향 참조 금지
- 계층 건너뛰기 금지

## 실제 적용
```
my-project/
├── services/           # Conway's Law
│   ├── user-service/   # User 팀 소유
│   │   ├── L1_domain/  # Dependency Law
│   │   ├── L2_logic/   # 내부 계층
│   │   └── L3_api/     # 구조
│   └── order-service/  # Order 팀 소유
│       └── (L1-L3)/    # 독립적 계층
```

## 왜 중요한가?
많은 프로젝트가 실패하는 이유: 이 두 구조를 혼동한다. 전체 프로젝트를 L1-L9로 나누려 하면 L5에 모든 코드가 쌓인다.

#structure #conway #dependency #orthogonal