# HAF 실전 가이드 - 파인만 아저씨 스타일

안녕! 소프트웨어가 왜 이렇게 복잡해지는지 궁금하지? 같이 해결해보자.

## 🍝 스파게티 코드의 비밀

너도 이런 경험 있지? 처음엔 깔끔했던 프로젝트가 6개월 후엔 스파게티가 되어 있는 거.

[026_flat_architecture_failure](./zettel_references/026_flat_architecture_failure.md)에서 배운 것처럼, 이건 수학적 필연이야:
- 10개 모듈 = 45개의 잠재적 연결
- 100개 모듈 = 4,950개의 연결!
- 네 뇌: 한 번에 7개만 기억 가능

"아하! 그래서 복잡해지는구나!"

## 🏗️ 자연에서 배우기

[023_memory_hierarchy_analogy](./zettel_references/023_memory_hierarchy_analogy.md)를 보자:
```
컴퓨터 메모리:
Register → Cache → RAM → Disk

왜 이렇게 만들까?
각 층은 이웃하고만 대화해!
```

[024_consciousness_hierarchy_connection](./zettel_references/024_consciousness_hierarchy_connection.md)도 마찬가지야:
```
뉴런 → 생각 → 개념 → 이해 → 지혜
```

자연은 이미 해답을 알고 있었어!

## 🎯 핵심 원리: 두 개의 축

여기 중요한 통찰이 있어. [021_conways_law_vs_dependency_law](./zettel_references/021_conways_law_vs_dependency_law.md):

**X축 - 누가 만드나? (Conway's Law)**
```
my-company/
├── frontend-team/
├── backend-team/
└── mobile-team/
```

**Y축 - 어떻게 쌓나? (Dependency Law)**
```
any-service/
├── L1_domain/    # 순수 로직
├── L2_services/  # 조합
└── L3_api/       # 노출
```

"아! 팀 구조와 코드 구조는 다른 거구나!"

## 🌱 작게 시작하기

[022_adaptive_layer_growth](./zettel_references/022_adaptive_layer_growth.md)의 지혜:

### 1단계: 3개 층으로 시작 (1-5명)
```
my-startup/
├── L1_core/     # 뭐하는 앱이야?
├── L2_service/  # 어떻게 관리해?
└── L3_api/      # 어떻게 쓰는데?
```

### 언제 층을 추가하나?
[028_code_placement_clarity](./zettel_references/028_code_placement_clarity.md)의 신호:
- "이 코드 어디 둬야 해?" 질문이 자주 나옴
- 한 층에 10개 이상 모듈
- 순환 참조 유혹이 생김

### 2단계: 5개 층 (5-50명)
```
L1_types/       # 기본 타입
L2_domain/      # 비즈니스 규칙
L3_services/    # 서비스 조합
L4_transport/   # API/메시징
L5_clients/     # UI/CLI
```

## ⚡ 정보 흐름의 법칙

[027_information_flow_direction](./zettel_references/027_information_flow_direction.md)를 기억해:
```
L3 → L2 → L1  ✅ (허용)
L1 → L2 → L3  ❌ (금지!)
```

"물이 폭포를 거슬러 올라가는 걸 봤어? 정보도 마찬가지야!"

## 🔄 번역 계약

[025_translation_contracts](./zettel_references/025_translation_contracts.md)가 핵심이야:

```typescript
// L2와 L3 사이의 명시적 계약
interface UserTranslator {
  toAPI(domainUser: User): APIUser
  toDomain(apiUser: APIUser): User
}
```

"각 층은 다른 언어를 써. 번역 없이는 대화할 수 없어!"

## 💼 실전 예제: 포인트 시스템

새 기능을 어디 둘지 헷갈려? 이렇게 생각해:

1. **포인트 계산 로직**
   - 순수 비즈니스 규칙? → `L1_domain/point-rules.ts`

2. **포인트 이력 조회**
   - 여러 도메인 조합? → `L2_services/point-history.ts`

3. **포인트 API**
   - 외부 노출? → `L3_api/points-endpoint.ts`

## 🚀 마이그레이션 전략

기존 스파게티 프로젝트가 있다고? 당황하지 마!

### 1주차: 경계 찾기
```bash
# 현재 의존성 시각화
npm run analyze-deps

# 순환 참조 찾기  
npm run find-cycles
```

### 2-3주차: 핵심 도메인 추출
- 가장 중요한 비즈니스 로직부터
- 테스트 있는 것부터
- L1으로 옮기기

### 4-8주차: 서비스 층 구축
- L1을 사용하는 L2 만들기
- 기존 코드는 L3에서 L2 호출하도록

### 검증
[020_empirical_validation](./zettel_references/020_empirical_validation.md)의 결과:
- 의존성 41% 감소
- 빌드 시간 37% 감소
- 온보딩 3주 → 1주

## 🎓 핵심 교훈

1. **계층은 자연스럽게 자란다** - 강제하지 마
2. **두 축을 분리하라** - 팀 ≠ 코드 구조  
3. **정보는 아래로만** - 폭포처럼
4. **경계에서 번역하라** - 명시적으로
5. **작게 시작하라** - 3개 층이면 충분

## 🤔 자주 하는 실수

### "우리는 달라요"
모든 팀이 그렇게 생각해. 하지만 복잡성의 법칙은 보편적이야.

### "9개 층 다 만들어야지!"  
아니야! [022_adaptive_layer_growth](./zettel_references/022_adaptive_layer_growth.md) 기억해? 필요할 때 자라는 거야.

### "이벤트로 우회하면..."
[027_information_flow_direction](./zettel_references/027_information_flow_direction.md): 정보 흐름을 속이지 마. 자연을 거스르면 진다.

## 마치며

HAF는 마법이 아니야. 자연의 원리를 소프트웨어에 적용한 것뿐이지. 

컴퓨터 메모리가 계층적인 이유, 의식이 계층적인 이유, 그리고 네 코드도 계층적이어야 하는 이유 - 모두 같은 원리야.

복잡성은 계층을 통해서만 관리할 수 있어. 이제 네 차례야!

---

*"If you can't explain it simply, you don't understand it well enough."* - 누군가가 물어보면, 이렇게 설명해줘: "층층이 쌓되, 팀이랑 코드는 따로 정리해!"