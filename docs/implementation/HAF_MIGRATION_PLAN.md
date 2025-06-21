# Layer9 HAF Migration Plan

## Phase 1: Establish Layer Structure (Week 1)

### 1.1 Create Layer Crates
```bash
# Create new layer-based crates
crates/l1-infrastructure/
crates/l2-platform/
crates/l3-runtime/
crates/l4-services/
crates/l5-components/
crates/l6-features/
crates/l7-application/
```

### 1.2 Define Layer Contracts
Create contract modules at each layer boundary:

```rust
// crates/l3-runtime/src/contracts/services.rs
pub trait ServiceRegistry {
    fn register_service(&mut self, name: &str, service: Box<dyn Service>);
    fn get_service(&self, name: &str) -> Option<&dyn Service>;
}

// crates/l4-services/src/contracts/components.rs
pub trait ComponentFactory {
    type Props;
    type Output;
    fn create(&self, props: Self::Props) -> Self::Output;
}
```

## Phase 2: Move Core Infrastructure (Week 2)

### 2.1 Layer 1 - Infrastructure
Move to `crates/l1-infrastructure/`:
- CLI (`crates/cli/`)
- Deployment configs
- Build tools

### 2.2 Layer 2 - Platform
Move to `crates/l2-platform/`:
- `server.rs` → `platform/server/mod.rs`
- `env.rs` → `platform/env/mod.rs`
- WASM bindings

### 2.3 Layer 3 - Runtime
Move to `crates/l3-runtime/`:
- `vdom.rs` → `runtime/vdom/mod.rs`
- `reactive_v2.rs` → `runtime/reactive/mod.rs`
- `hooks.rs` → `runtime/hooks/mod.rs`

## Phase 3: Establish Service Layer (Week 3)

### 3.1 Auth Service
Create `crates/l4-services/auth/`:
```
auth/
├── src/
│   ├── lib.rs
│   ├── jwt.rs
│   ├── providers/
│   │   ├── oauth.rs
│   │   └── local.rs
│   └── middleware.rs
```

### 3.2 Database Service
Create `crates/l4-services/database/`:
```
database/
├── src/
│   ├── lib.rs
│   ├── connection.rs
│   ├── query_builder.rs
│   └── drivers/
│       ├── sqlite.rs
│       └── postgres.rs
```

### 3.3 API Service
Create `crates/l4-services/api/`:
```
api/
├── src/
│   ├── lib.rs
│   ├── rest.rs
│   ├── graphql.rs
│   └── websocket.rs
```

## Phase 4: Component System Migration (Week 4)

### 4.1 Core Components
Move to `crates/l5-components/core/`:
- `component.rs` → `core/component.rs`
- `props.rs` (extracted from component.rs)
- `element.rs` (extracted from component.rs)

### 4.2 Form Components
Move to `crates/l5-components/forms/`:
- `form.rs` → `forms/mod.rs`
- `form_builder.rs` → `forms/builder.rs`
- `form_traits.rs` → `forms/traits.rs`

### 4.3 UI Components
Move to `crates/l5-components/ui/`:
- `ui.rs` → `ui/mod.rs`
- Create submodules for different UI components

## Phase 5: Feature Layer Organization (Week 5)

### 5.1 Image Optimization Feature
Create `crates/l6-features/image-optimization/`:
```
image-optimization/
├── src/
│   ├── lib.rs
│   ├── handler.rs
│   ├── lazy.rs
│   └── transform.rs
```

### 5.2 Internationalization Feature
Create `crates/l6-features/i18n/`:
```
i18n/
├── src/
│   ├── lib.rs
│   ├── locale.rs
│   └── translations.rs
```

### 5.3 Styling Feature
Create `crates/l6-features/styling/`:
```
styling/
├── src/
│   ├── lib.rs
│   ├── css_runtime.rs
│   ├── styled_components.rs
│   └── themes.rs
```

## Phase 6: Application Layer (Week 6)

### 6.1 Routing
Move to `crates/l7-application/routing/`:
- `router_v2.rs` → `routing/mod.rs`
- Extract route definitions
- Create navigation service

### 6.2 State Management
Move to `crates/l7-application/state/`:
- `state.rs` → `state/mod.rs`
- `cache.rs` → `state/cache.rs`
- Create state synchronization

## Phase 7: Fix Dependencies (Week 7-8)

### 7.1 Remove Upward Dependencies
- Middleware should not import Auth directly
- Router should not import Component directly
- VDOM should not know about specific component types

### 7.2 Implement Contracts
Replace direct imports with contract interfaces:

```rust
// Before (violation)
use crate::auth::User;

// After (contract-based)
use l4_services::contracts::AuthService;
```

### 7.3 Add Translation Layers
Create DTOs for cross-layer communication:

```rust
// l4-services/src/dto/user.rs
pub struct UserDTO {
    pub id: String,
    pub email: String,
}

// l5-components/src/models/user.rs
pub struct UserModel {
    pub id: String,
    pub display_name: String,
}

impl From<UserDTO> for UserModel {
    // Translation logic
}
```

## Phase 8: Testing and Validation (Week 9)

### 8.1 Layer Isolation Tests
- Each layer should compile independently
- Mock contracts for testing
- No cross-layer imports

### 8.2 Integration Tests
- Test contract implementations
- Verify translation layers
- End-to-end scenarios

### 8.3 Performance Validation
- Measure overhead of contracts
- Optimize hot paths
- Profile layer boundaries

## Phase 9: Documentation (Week 10)

### 9.1 Architecture Guide
- Layer responsibilities
- Contract specifications
- Migration guide for new features

### 9.2 Developer Guide
- How to add new services
- Component development
- Feature integration

### 9.3 API Documentation
- Public contracts
- Internal APIs
- Extension points

## Success Metrics

1. **Compile-time Isolation**: Each layer compiles independently
2. **Clear Dependencies**: `cargo tree` shows clean hierarchy
3. **No Circular Deps**: `cargo-deny` passes
4. **Test Coverage**: 80%+ per layer
5. **Documentation**: 100% public API coverage

## Rollback Plan

If migration causes issues:
1. Keep `crates/core` as fallback
2. Gradual migration with feature flags
3. Maintain compatibility layer
4. Incremental rollout

## Post-Migration

1. Remove old `crates/core`
2. Update all examples
3. Deprecate v1 APIs
4. Performance optimization
5. Community feedback incorporation