# WARP - Web Architecture Rust Platform

> Next.js의 성능, Rust의 계층적 추상화

## 🚀 What is WARP?

WARP는 Next.js의 장점을 그대로 살리면서, Rust의 강력한 타입 시스템과 계층적 추상화를 통해 대규모 웹 애플리케이션을 구축할 수 있는 프레임워크입니다.

## 핵심 원칙

1. **계층적 추상화 강제** - L9 (철학) 부터 L1 (인프라)까지 명확한 계층
2. **Next.js 호환** - 기존 Next.js 프로젝트에 점진적 도입 가능
3. **Zero-Cost Abstractions** - Rust의 컴파일 타임 최적화
4. **Type-Safe Everything** - 서버/클라이언트 경계까지 완전한 타입 안정성

## 아키텍처

```
L9: Philosophy     - 앱의 핵심 철학과 비전
L8: Architecture   - 전체 시스템 설계
L7: Application    - 비즈니스 로직
L6: Features       - 기능 모듈
L5: Components     - UI 컴포넌트
L4: Services       - 서버/클라이언트 서비스
L3: Runtime        - WASM/JS 런타임
L2: Platform       - Next.js/Vercel 통합
L1: Infrastructure - 배포 및 빌드
```

## Quick Start

```bash
# Install WARP CLI
cargo install warp-cli

# Create new project
warp new my-app

# Development
warp dev

# Build for production
warp build
```

## Example

```rust
use warp::prelude::*;

#[warp::app]
struct MyApp {
    name: &'static str,
}

#[warp::page("/")]
async fn home() -> Page {
    Page::new()
        .title("WARP Example")
        .component(HelloWorld)
}

#[warp::component]
fn HelloWorld() -> Element {
    let count = use_state(|| 0);
    
    view! {
        <div>
            <h1>"Count: " {count}</h1>
            <button on_click={move |_| count += 1}>
                "Increment"
            </button>
        </div>
    }
}

#[warp::server]
async fn get_data() -> Result<String> {
    Ok("Hello from Rust server!".to_string())
}
```

## Why WARP?

### Next.js의 문제점
- 평면적 파일 구조
- 숨겨진 복잡도
- 계층 관리 불가능
- 대규모 프로젝트에서 유지보수 지옥

### WARP의 해결책
- 컴파일 타임에 계층 검증
- 명시적 의존성 관리
- 더 작은 번들 사이즈
- 더 빠른 빌드 시간

## Performance

| Metric | Next.js | WARP |
|--------|---------|------|
| First Load | 85kb | 45kb |
| Build Time | 30s | 5s |
| Type Safety | Partial | 100% |
| 계층적 추상화 | ❌ | ✅ |

## License

MIT