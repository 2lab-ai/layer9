# Layer9 HAF 리팩토링 마스터 플랜

## 🎯 비전: HAF를 강제하는 웹 아키텍처 플랫폼

Layer9는 단순한 웹 프레임워크가 아닙니다. HAF(Hierarchical Abstraction First) 철학을 코드로 구현하여, 개발자들이 자연스럽게 올바른 아키텍처를 만들도록 유도하는 플랫폼입니다.

## 📊 마일스톤 Overview

### M1: 기능 구현 ✅ (완료)
- 핵심 버그 수정
- 기본 기능 구현
- 작동 가능한 프레임워크

### M2: HAF 리팩토링 🚧 (현재)
- HAF 철학 적용
- 3-레이어 아키텍처 구현
- 계층 강제 시스템 구축

### M3: 생태계 구축 🔮 (미래)
- HAF 개발 도구
- 자동 검증 시스템
- 커뮤니티 생태계

## 🏗️ 현재 구조 분석

### 문제점
```
layer9/
├── crates/
│   ├── core/src/        # 716개 파일이 평면적으로 나열
│   ├── cli/             # 도구와 핵심 로직 혼재
│   └── macro/           # 매크로가 너무 많은 일을 함
└── examples/            # HAF 원칙을 따르지 않음
```

### HAF 위반 사례
1. **평면적 구조**: core/src에 모든 것이 뒤섞임
2. **양방향 의존성**: 컴포넌트가 상위 레이어를 참조
3. **불명확한 경계**: 어디에 무엇을 두어야 할지 모호
4. **번역 계약 부재**: 레이어 간 암묵적 결합

## 🎯 목표 구조

### 3-레이어 아키텍처 (시작점)
```
layer9/
├── L1_core/              # WHAT - 순수 비즈니스 로직
│   ├── vdom/            # Virtual DOM 알고리즘
│   ├── reactive/        # 반응성 시스템
│   └── component/       # 컴포넌트 추상화
├── L2_runtime/          # HOW - 실행 환경
│   ├── wasm/           # WASM 바인딩
│   ├── ssr/            # 서버 사이드 렌더링
│   └── hydration/      # 클라이언트 하이드레이션
└── L3_framework/        # WHEN/WHERE - 사용자 인터페이스
    ├── router/         # 라우팅 시스템
    ├── http/           # HTTP 레이어
    └── cli/            # CLI 도구
```

### 서비스별 구조 (Conway's Law)
```
layer9/
├── services/
│   ├── vdom-service/
│   │   ├── L1_algorithm/    # 순수 diff/patch
│   │   ├── L2_runtime/      # DOM 조작
│   │   └── L3_api/          # 공개 API
│   ├── reactive-service/
│   │   └── L1-L3/
│   └── component-service/
│       └── L1-L3/
└── shared/
    └── contracts/           # 레이어 간 계약
```

## 🔄 Phase별 실행 계획

### Phase 1: 현재 상태 분석 (1주)

#### 1.1 의존성 맵핑
```rust
// 도구 생성
cargo run --bin analyze-deps > dependency-map.json
```

#### 1.2 레이어 위반 식별
- component.rs가 router를 참조? ❌
- vdom이 http를 알고 있음? ❌
- 순환 참조 발견

#### 1.3 자연스러운 경계 찾기
- VDOM: 순수 알고리즘
- Component: 추상화 레이어
- Runtime: 플랫폼 특화

### Phase 2: HAF 아키텍처 설계 (2주)

#### 2.1 L1 Core - 순수 로직
```rust
// L1은 외부를 모릅니다
pub mod l1_core {
    // 순수 함수만
    pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
        // IO 없음, 부작용 없음
    }
}
```

#### 2.2 L2 Runtime - 조율 레이어
```rust
// L2는 L1을 조합합니다
pub mod l2_runtime {
    use crate::l1_core;
    
    pub struct Runtime {
        vdom: l1_core::VDom,
    }
}
```

#### 2.3 L3 Framework - 외부 인터페이스
```rust
// L3는 외부와 소통합니다
pub mod l3_framework {
    use crate::l2_runtime::Runtime;
    
    pub fn mount(selector: &str) {
        // 브라우저 API 사용
    }
}
```

### Phase 3: HAF 강제 시스템 (3주)

#### 3.1 팬텀 타입으로 레이어 표현
```rust
// 컴파일 타임에 레이어 강제
#[phantom]
struct L1;
#[phantom] 
struct L2;
#[phantom]
struct L3;

pub struct Component<L> {
    _layer: PhantomData<L>,
}

// L1 컴포넌트는 L2를 호출할 수 없음
impl Component<L1> {
    pub fn render(&self) -> VNode {
        // 컴파일 에러: self.fetch() 불가능
    }
}
```

#### 3.2 번역 계약 자동 생성
```rust
#[derive(Contract)]
trait VDomToRuntime {
    fn patches_to_ops(patches: Vec<Patch>) -> Vec<DomOp>;
}
```

#### 3.3 의존성 방향 검증
```rust
#[test]
fn verify_no_upward_deps() {
    let deps = analyze_dependencies();
    assert!(deps.verify_hierarchy());
}
```

### Phase 4: 점진적 마이그레이션 (4주)

#### 4.1 핵심 컴포넌트부터
1. VDOM → 3레이어로 분리
2. Component → 순수 추상화
3. Reactive → 레이어별 책임 분리

#### 4.2 HAF 예제 앱
```rust
// examples/haf-todo/
├── L1_domain/
│   └── todo.rs         // 순수 Todo 로직
├── L2_app/
│   └── todo_service.rs // Todo 관리
└── L3_ui/
    └── todo_component.rs // UI 컴포넌트
```

#### 4.3 개발자 경험
```rust
// HAF를 따르지 않으면 컴파일 에러
use layer9::haf::*;

// 자동으로 올바른 레이어 추론
#[component]
fn TodoItem(todo: &Todo) -> VNode {
    // L3 컴포넌트로 자동 태깅
}
```

## 🛠️ HAF 개발 도구

### 1. Layer9 HAF CLI
```bash
layer9 haf check        # 레이어 위반 검사
layer9 haf visualize    # 의존성 시각화
layer9 haf fix         # 자동 수정 제안
```

### 2. VS Code Extension
- 레이어 위반 실시간 하이라이트
- 올바른 레이어 제안
- 번역 계약 자동 완성

### 3. 빌드 타임 검증
```toml
[haf]
strict = true           # 레이어 위반시 빌드 실패
max_layer_depth = 3     # 최대 레이어 수 제한
```

## 📈 성공 지표

### 정량적 지표
- 의존성 엣지 40% 감소
- 빌드 시간 30% 단축
- 번들 크기 25% 감소
- 온보딩 시간 50% 단축

### 정성적 지표
- "이 코드 어디 둬요?" 질문 90% 감소
- 아키텍처 리뷰 시간 60% 단축
- 신규 기능 추가 속도 2배 향상

## 🚀 실행 우선순위

### Week 1-2: 분석 및 설계
- [ ] 현재 구조 분석 완료
- [ ] HAF 아키텍처 설계 문서
- [ ] 팀 교육 및 합의

### Week 3-4: 핵심 시스템 구축
- [ ] HAF 타입 시스템 구현
- [ ] 레이어 강제 메커니즘
- [ ] 번역 계약 시스템

### Week 5-8: 마이그레이션
- [ ] VDOM 서비스 리팩토링
- [ ] Component 서비스 리팩토링
- [ ] 예제 앱 재작성

### Week 9-10: 도구 및 문서
- [ ] HAF CLI 도구 개발
- [ ] 개발자 가이드 작성
- [ ] 커뮤니티 공개

## 🎯 최종 목표

Layer9를 사용하는 개발자들이:
1. HAF를 자연스럽게 따르도록
2. 복잡성을 계층으로 관리하도록
3. 지속 가능한 아키텍처를 만들도록

"복잡성은 오직 계층을 통해서만 관리된다" - 이것이 Layer9의 핵심 가치입니다.