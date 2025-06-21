# MIRA HAF 마이그레이션 계획 (Zettelkasten 기반)

## 현재 상황 분석

MIRA는 현재 [026_flat_architecture_failure](./zettel_references/026_flat_architecture_failure.md)의 전형적인 증상을 보이고 있습니다:
- 716MB가 L5_implementation에 집중
- 9개 계층이 실제로는 팀 경계와 코드 계층을 혼동
- 순환 참조 가능성 높음

## 핵심 통찰 적용

### 1. 두 축의 분리 ([021_conways_law_vs_dependency_law](./zettel_references/021_conways_law_vs_dependency_law.md))

현재 MIRA 구조는 Conway's Law와 Dependency Law를 혼동하고 있습니다:

**문제의 구조:**
```
MIRA/
├── L1_prompts/       # 혼동: 이건 데이터? 코드?
├── L5_implementation/ # 716MB - 모든 게 여기 쌓임
└── L9_research/      # 연구는 계층이 아니라 팀 활동
```

**제안하는 구조:**
```
MIRA/
├── services/          # Conway's Law (WHO)
│   ├── memory/        # Memory 팀
│   │   ├── L1_core/   # Dependency Law (HOW)
│   │   ├── L2_logic/
│   │   └── L3_api/
│   ├── retrieval/     # Retrieval 팀  
│   │   └── L1-L3/
│   └── mcp-server/    # MCP 팀
│       └── L1-L3/
├── shared/           # 공유 라이브러리
└── research/         # 연구 결과물
```

### 2. 적응형 성장 ([022_adaptive_layer_growth](./zettel_references/022_adaptive_layer_growth.md))

MIRA는 9개 층을 강제할 필요 없습니다. 각 서비스는 3개 층으로 시작:

**Memory Service (3 layers):**
```
memory/
├── L1_store/      # 순수 저장 로직
├── L2_index/      # 인덱싱 & 검색
└── L3_interface/  # MCP 인터페이스
```

### 3. 명확한 코드 배치 ([028_code_placement_clarity](./zettel_references/028_code_placement_clarity.md))

각 기능의 위치가 명확해집니다:
- 벡터 연산 → `memory/L1_store/`
- 검색 알고리즘 → `retrieval/L1_core/`
- MCP 프로토콜 → `mcp-server/L3_interface/`

## 단계별 마이그레이션

### Phase 1: 서비스 경계 정의 (1주)

[027_information_flow_direction](./zettel_references/027_information_flow_direction.md)에 따라 정보 흐름 분석:

```bash
# 현재 의존성 맵 생성
npm run analyze-dependencies > deps.json

# 자연스러운 경계 찾기
npm run find-clusters
```

### Phase 2: 핵심 도메인 추출 (2-3주)

각 서비스의 L1 만들기:

```typescript
// memory/L1_store/vector-store.ts
export class VectorStore {
  // 순수 벡터 저장 로직
  // 외부 의존성 없음
}
```

### Phase 3: 번역 계약 구축 ([025_translation_contracts](./zettel_references/025_translation_contracts.md)) (3-4주)

```typescript
// memory/L2_index/contracts.ts
interface StoreToIndexContract {
  toIndexFormat(vector: Vector): IndexedVector
  toStoreFormat(indexed: IndexedVector): Vector
}
```

### Phase 4: 점진적 이동 (4-8주)

1. **테스트 작성**
   - 각 L1 컴포넌트에 대한 단위 테스트
   - 계약 테스트로 경계 검증

2. **기존 코드 리팩토링**
   ```typescript
   // 이전: hierarchy/L5_implementation/memory/index.ts
   // 이후: services/memory/L2_index/memory-index.ts
   ```

3. **의존성 역전**
   - 상위 계층이 하위 계층에 의존하도록
   - 인터페이스를 통한 추상화

## 성공 지표

[020_empirical_validation](./zettel_references/020_empirical_validation.md)를 참고한 목표:

- **단기 (3개월)**
  - 716MB → 서비스별 100MB 미만
  - 빌드 시간 30% 감소
  - 테스트 커버리지 80% 이상

- **중기 (6개월)**
  - 새 기능 추가 시간 50% 감소
  - 온보딩 시간 2주 → 1주
  - 서비스 간 의존성 제거

## 위험 요소 및 대응

### 1. "모든 것이 연결되어 있어요"
- [026_flat_architecture_failure](./zettel_references/026_flat_architecture_failure.md)의 전형적 증상
- 해결: 작은 부분부터 시작, 점진적 분리

### 2. "9개 층이 표준 아닌가요?"
- [022_adaptive_layer_growth](./zettel_references/022_adaptive_layer_growth.md) 원칙 교육
- 각 서비스는 필요에 따라 성장

### 3. "MCP가 모든 걸 알아야 해요"
- [021_conways_law_vs_dependency_law](./zettel_references/021_conways_law_vs_dependency_law.md) 적용
- MCP는 인터페이스만 알면 됨

## 장기 비전

### 6개월 후
```
MIRA/
├── services/
│   ├── memory/      (3-5 layers)
│   ├── retrieval/   (3 layers)
│   ├── mcp-server/  (3 layers)
│   └── analytics/   (신규, 3 layers)
├── packages/        (공유 라이브러리)
└── docs/           (zettelkasten 기반!)
```

### 1년 후
- 각 서비스가 독립적으로 배포 가능
- 새 서비스 추가가 기존 서비스에 영향 없음
- [024_consciousness_hierarchy_connection](./zettel_references/024_consciousness_hierarchy_connection.md)처럼 자연스러운 계층 형성

## 핵심 교훈

1. **팀 구조 ≠ 코드 구조**: 두 축을 분리하라
2. **3개 층부터 시작**: 필요시 성장
3. **정보는 아래로만**: 역류 금지
4. **명시적 계약**: 암묵적 결합 제거
5. **점진적 마이그레이션**: 빅뱅은 실패한다

이 계획은 MIRA를 [023_memory_hierarchy_analogy](./zettel_references/023_memory_hierarchy_analogy.md)의 원리에 따라 재구성하여, 복잡성을 O(n²)에서 O(n log n)으로 줄입니다.

---

*"The best architectures are discovered, not designed."* - MIRA가 자연스럽게 진화하도록 하세요.