# Layer9 3-Layer Architecture Design

## 🎯 설계 원칙

HAF 철학에 따라 Layer9를 3개의 기본 레이어로 시작합니다. 각 레이어는 명확한 책임과 경계를 가지며, 정보는 상위에서 하위로만 흐릅니다.

```
L3 (Interface) → L2 (Runtime) → L1 (Core)
     ↓               ↓              ↓
   외부 API      실행 환경      순수 로직
```

## 📐 Layer 1: Core (순수 비즈니스 로직)

### 책임
- 순수 함수와 불변 데이터 구조
- 외부 의존성 없음 (no I/O, no side effects)
- 플랫폼 독립적 알고리즘

### 구성 요소
```
L1_core/
├── vdom/
│   ├── node.rs          // VNode 타입 정의
│   ├── diff.rs          // 순수 diff 알고리즘
│   └── patch.rs         // Patch 타입 정의
├── reactive/
│   ├── signal.rs        // Signal 추상화
│   ├── computed.rs      // 계산된 값
│   └── effect.rs        // Effect 정의
├── component/
│   ├── props.rs         // Props 타입
│   ├── lifecycle.rs     // 생명주기 정의
│   └── render.rs        // 렌더 함수 타입
└── style/
    ├── css.rs          // CSS 추상화
    └── theme.rs        // 테마 시스템
```

### 예시 코드
```rust
// L1_core/vdom/diff.rs
pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    // 순수 함수: 입력만으로 출력 결정
    // 부작용 없음, I/O 없음
    match (old, new) {
        (VNode::Text(old_text), VNode::Text(new_text)) => {
            if old_text != new_text {
                vec![Patch::UpdateText(new_text.clone())]
            } else {
                vec![]
            }
        }
        // ... 다른 경우들
    }
}
```

## 🏃 Layer 2: Runtime (실행 환경)

### 책임
- L1의 순수 로직을 실제 환경에서 실행
- 플랫폼별 구현 (WASM, SSR)
- 부작용 관리

### 구성 요소
```
L2_runtime/
├── wasm/
│   ├── dom.rs           // DOM 조작
│   ├── events.rs        // 이벤트 처리
│   └── scheduler.rs     // 렌더 스케줄링
├── server/
│   ├── ssr.rs          // 서버 렌더링
│   ├── hydration.rs    // 하이드레이션
│   └── streaming.rs    // 스트리밍 SSR
├── executor/
│   ├── renderer.rs     // 렌더러 구현
│   ├── reconciler.rs   // 재조정자
│   └── scheduler.rs    // 작업 스케줄링
└── contracts/
    ├── core_to_runtime.rs  // L1→L2 계약
    └── runtime_types.rs    // 런타임 타입
```

### 번역 계약
```rust
// L2_runtime/contracts/core_to_runtime.rs
pub trait CoreToRuntime {
    // L1의 Patch를 L2의 DomOp로 변환
    fn patch_to_dom_op(patch: &core::Patch) -> DomOp;
    
    // L1의 Effect를 L2의 Task로 변환
    fn effect_to_task(effect: &core::Effect) -> Task;
}
```

## 🌐 Layer 3: Framework (외부 인터페이스)

### 책임
- 사용자 대면 API
- HTTP, WebSocket 등 네트워크 프로토콜
- CLI 도구 및 개발자 경험

### 구성 요소
```
L3_framework/
├── api/
│   ├── component_api.rs  // 컴포넌트 매크로 API
│   ├── hooks_api.rs      // use_state 등 훅
│   └── router_api.rs     // 라우팅 API
├── http/
│   ├── server.rs         // HTTP 서버
│   ├── middleware.rs     // 미들웨어 시스템
│   └── websocket.rs      // WebSocket 지원
├── cli/
│   ├── commands.rs       // CLI 명령어
│   ├── dev_server.rs     // 개발 서버
│   └── build.rs          // 빌드 시스템
└── contracts/
    ├── runtime_to_framework.rs  // L2→L3 계약
    └── public_api.rs           // 공개 API
```

### 공개 API 예시
```rust
// L3_framework/api/component_api.rs
#[macro_export]
macro_rules! component {
    ($name:ident, $props:ty, $body:expr) => {
        // L3 매크로가 L2 런타임을 통해 L1 컴포넌트 생성
        pub fn $name(props: $props) -> impl Component {
            runtime::create_component(move |props| {
                core::render($body)
            })
        }
    };
}
```

## 🔄 레이어 간 통신

### 단방향 의존성
```
L3 → L2 → L1  ✅ (허용)
L1 → L2 → L3  ❌ (금지!)
```

### 번역 계약 예시
```rust
// L2가 L1과 L3 사이를 중재
mod runtime {
    use crate::core::{VNode, Patch};
    use crate::framework::{HttpRequest, HttpResponse};
    
    // L1 → L2 번역
    pub fn apply_patches(patches: Vec<Patch>) {
        for patch in patches {
            let dom_op = translate_patch(patch);
            execute_dom_op(dom_op);
        }
    }
    
    // L3 → L2 번역
    pub fn handle_request(req: HttpRequest) -> HttpResponse {
        let vnode = render_app();
        let html = render_to_string(vnode);
        HttpResponse::ok(html)
    }
}
```

## 🎨 서비스별 구조

각 주요 기능은 독립적인 서비스로 구성되며, 각 서비스는 3개 레이어를 가집니다.

### VDOM 서비스
```
services/vdom/
├── L1_algorithm/     # diff/patch 알고리즘
├── L2_executor/      # DOM 조작 실행
└── L3_api/          # 공개 VDOM API
```

### Reactive 서비스
```
services/reactive/
├── L1_core/         # Signal, Computed 로직
├── L2_runtime/      # 반응성 실행 환경
└── L3_hooks/        # use_state 등 훅
```

### Component 서비스
```
services/component/
├── L1_abstract/     # 컴포넌트 추상화
├── L2_lifecycle/    # 생명주기 관리
└── L3_macro/        # component! 매크로
```

## 📦 패키지 구조

```toml
# Cargo.toml
[workspace]
members = [
    "L1_core",
    "L2_runtime", 
    "L3_framework",
    "services/*",
]

[dependencies]
# L1은 순수 Rust만 사용
layer9_core = { path = "L1_core" }

# L2는 L1에만 의존
layer9_runtime = { path = "L2_runtime", deps = ["layer9_core"] }

# L3는 L2에만 의존
layer9_framework = { path = "L3_framework", deps = ["layer9_runtime"] }
```

## 🔍 레이어 강제 방법

### 1. 컴파일 타임 검사
```rust
// 팬텀 타입으로 레이어 표시
pub struct Component<L> {
    _layer: PhantomData<L>,
}

// L1 컴포넌트는 L2/L3 기능 사용 불가
impl Component<L1> {
    pub fn render(&self) -> VNode {
        // 컴파일 에러: http::fetch() 사용 불가
    }
}
```

### 2. 의존성 린터
```toml
[layer9.lints]
deny_upward_deps = true
max_layer_depth = 3
```

### 3. 테스트
```rust
#[test]
fn no_upward_dependencies() {
    let deps = analyze_crate_deps();
    assert!(deps.verify_hierarchy());
}
```

## 📈 성장 경로

### 시작 (3 레이어)
현재 설계로 1-5명의 개발자가 효율적으로 작업

### 성장 시 (5 레이어)
```
L1_types      # 기본 타입
L2_core       # 핵심 로직
L3_runtime    # 실행 환경
L4_services   # 서비스 조합
L5_interface  # 외부 API
```

### 확장 시 (7 레이어)
팀이 50명 이상으로 성장하면 더 세분화

## 🎯 다음 단계

1. **L1 Core 구현** - 순수 함수와 타입 정의
2. **번역 계약 정의** - 레이어 간 명시적 계약
3. **L2 Runtime 구축** - WASM/SSR 실행 환경
4. **L3 Framework 완성** - 개발자 친화적 API

이 3-레이어 구조는 Layer9의 HAF 여정의 시작점이며, 프로젝트가 성장함에 따라 자연스럽게 확장됩니다.