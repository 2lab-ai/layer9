# HAF Migration Guide for Layer9

## Overview

This guide helps you migrate existing Layer9 code to the HAF (Hierarchical Architecture First) system. HAF enforces architectural constraints at compile time, ensuring clean separation between layers.

## Understanding HAF Layers

### L1: Core (Pure Business Logic)
- No side effects, no I/O
- Pure data structures and algorithms
- Examples: VNode, diff algorithms, routing logic

### L2: Runtime (Execution Environment)
- Manages side effects and lifecycle
- Orchestrates between L1 and L3
- Examples: Component runtime, effect scheduling, patch application

### L3: Framework (External Interfaces)
- User-facing APIs and I/O operations
- DOM manipulation, HTTP requests, WebSocket connections
- Examples: App builders, hooks, DOM renderer

## Migration Strategy

### Phase 1: Assessment

1. **Identify Layer Violations**
   ```rust
   // Before: Mixed concerns
   pub struct Component {
       vdom: VNode,           // L1: Pure data
       dom_ref: web_sys::Element,  // L3: DOM reference
       state: RefCell<State>, // L2: Runtime state
   }
   
   // After: Separated by layer
   pub struct L1Component { vdom: VNode }
   pub struct L2Instance { state: ComponentState }
   pub struct L3Renderer { dom_refs: HashMap<Id, Element> }
   ```

2. **Map Dependencies**
   - Use `cargo tree` to understand current dependencies
   - Identify which modules belong to which layer
   - Note cross-layer dependencies that need refactoring

### Phase 2: Gradual Migration

#### Step 1: Enable HAF Feature Flag
```toml
# Cargo.toml
[features]
haf-components = []
```

#### Step 2: Migrate Leaf Components First

```rust
// Old component
impl Component for Button {
    fn render(&self) -> Element {
        Element::Node {
            tag: "button".to_string(),
            props: Props { 
                class: Some("btn".to_string()),
                on_click: Some(self.handler.clone()),
                ..Default::default()
            },
            children: vec![Element::Text(self.label.clone())],
        }
    }
}

// HAF component
haf_component! {
    pub struct Button {
        props: ButtonProps,
    }
    
    impl render {
        VNode::Element {
            tag: "button".to_string(),
            props: VProps {
                class: Some("btn".to_string()),
                events: vec![("click".to_string(), props.click_id)],
                ..Default::default()
            },
            children: vec![VNode::Text(props.label.clone())],
        }
    }
}
```

#### Step 3: Use Compatibility Layer

```rust
use layer9::haf::compat::{element_to_vnode, migrate_component};

// Quick migration with macro
migrate_component!(MyOldComponent);

// Or manual conversion
let old_element = old_component.render();
let haf_vnode = element_to_vnode(&old_element);
```

### Phase 3: Refactor Core Systems

#### Component System Migration

1. **Extract Pure Logic to L1**
   ```rust
   // L1: Pure component definition
   pub trait PureComponent<L: Layer> {
       type Props: Clone + 'static;
       fn render(&self, props: &Self::Props) -> VNode<L>;
   }
   ```

2. **Move Runtime to L2**
   ```rust
   // L2: Component lifecycle and state
   pub struct ComponentRuntime<L: Layer> {
       instances: Vec<ComponentInstance<L>>,
       effects: Vec<Effect<L>>,
   }
   ```

3. **Isolate DOM Operations in L3**
   ```rust
   // L3: DOM manipulation
   pub struct DomRenderer<L: Layer> {
       document: web_sys::Document,
       nodes: HashMap<NodeId, web_sys::Node>,
   }
   ```

#### State Management Migration

```rust
// Before: Mixed state and effects
pub struct State<T> {
    value: Rc<RefCell<T>>,
    subscribers: Vec<Box<dyn Fn()>>, // L3 callbacks in L1!
}

// After: Separated by layer
// L1: Pure state
pub struct Signal<T> { value: T }

// L2: Effect scheduling
pub struct EffectScheduler {
    effects: Vec<Effect>,
    queue: VecDeque<EffectId>,
}

// L3: DOM updates
pub struct DomUpdater {
    renderer: DomRenderer,
}
```

### Phase 4: Enforce Contracts

#### Define Layer Contracts
```rust
// L1 → L2 Contract
pub struct VNodeToRuntimeContract;
impl L1ToL2Contract for VNodeToRuntimeContract {
    type L1Type = VNode;
    type L2Type = ComponentInstance;
    
    fn translate(vnode: VNode) -> ComponentInstance {
        // Transform pure VNode to runtime instance
    }
}

// L2 → L3 Contract  
pub struct RuntimeToDomContract;
impl L2ToL3Contract for RuntimeToDomContract {
    type L2Type = DomOperation;
    type L3Type = web_sys::Element;
    
    fn translate(op: DomOperation) -> web_sys::Element {
        // Execute DOM operation
    }
}
```

## Common Migration Patterns

### Pattern 1: Extracting Side Effects

```rust
// Before: Side effects in component
fn TodoItem(props: TodoProps) -> Element {
    // Side effect in render!
    web_sys::console::log_1(&"Rendering todo".into());
    
    Element::Node { /* ... */ }
}

// After: Pure component with effects in runtime
// L1: Pure component
fn TodoItem(props: TodoProps) -> VNode<L1> {
    VNode::Element { /* ... */ }
}

// L2: Effect management
impl ComponentRuntime<L2> {
    fn schedule_effect(&mut self, effect: Effect<L2>) {
        self.effects.push(effect);
    }
}
```

### Pattern 2: Separating Data from Behavior

```rust
// Before: Data and behavior mixed
struct Router {
    routes: HashMap<String, Component>,
    current: String,
    
    fn navigate(&mut self, path: &str) {
        self.current = path.to_string();
        // DOM manipulation here
        window().history().push_state(/* ... */);
    }
}

// After: Separated by layer
// L1: Pure routing logic
struct Route {
    path: String,
    component: ComponentDef,
}

fn match_route(path: &str, routes: &[Route]) -> Option<&Route> {
    routes.iter().find(|r| r.path == path)
}

// L3: Browser API interaction
struct BrowserRouter {
    fn navigate(&self, path: &str) -> Result<(), JsValue> {
        window().history().push_state(/* ... */)
    }
}
```

### Pattern 3: Dependency Injection

```rust
// Before: Direct dependencies
struct App {
    db: Database,      // Direct L3 dependency
    auth: AuthService, // Another L3 dependency
}

// After: Layer-appropriate dependencies
// L1: Pure app logic
struct AppLogic {
    fn process_request(req: Request) -> Response { /* ... */ }
}

// L2: Service orchestration
struct AppRuntime<L: Layer> {
    services: ServiceRegistry<L>,
}

// L3: External services
struct AppServices {
    db: Box<dyn DatabaseService>,
    auth: Box<dyn AuthService>,
}
```

## Testing HAF Code

### Layer Isolation Tests

```rust
#[test]
fn test_l1_component_is_pure() {
    // L1 components should be deterministic
    let props = ButtonProps { label: "Click".into() };
    let vnode1 = Button::render(&props);
    let vnode2 = Button::render(&props);
    assert_eq!(vnode1, vnode2);
}

#[test]
fn test_layer_boundaries() {
    // This should NOT compile:
    // fn l1_function() {
    //     window().alert("L1 can't access browser APIs!");
    // }
}
```

### Contract Tests

```rust
#[test]
fn test_vnode_to_dom_contract() {
    let vnode = VNode::Text("Hello".into());
    let contract = VNodeToDomContract::translate(vnode);
    assert!(contract.is_valid());
}
```

## Troubleshooting

### Common Errors

1. **"trait bound `L1: CanDepend<L3>` is not satisfied"**
   - You're trying to use L3 functionality in L1 code
   - Solution: Move the L3 dependency to a higher layer

2. **"cannot find type `Window` in L1 scope"**
   - Browser APIs are L3 only
   - Solution: Pass data down, events up through contracts

3. **"Component doesn't implement Default"**
   - HAF components need Default for initialization
   - Solution: Add `#[derive(Default)]` or implement manually

### Migration Checklist

- [ ] Identify all components to migrate
- [ ] Map current layer violations
- [ ] Enable HAF feature flag
- [ ] Migrate leaf components first
- [ ] Extract side effects to L2
- [ ] Move I/O operations to L3
- [ ] Define layer contracts
- [ ] Add HAF tests
- [ ] Remove compatibility layer
- [ ] Document architectural decisions

## Resources

- [HAF Philosophy](./PLAN/00-HAF-PHILOSOPHY.md)
- [HAF Implementation](./crates/core/src/haf/README.md)
- [Example HAF App](./examples/haf-todo/)
- [Layer9 Architecture Docs](./ARCHITECTURE.md)

## Getting Help

If you encounter issues during migration:

1. Check the [HAF Violations Analysis](./HAF_VIOLATIONS_ANALYSIS.md)
2. Review [example migrations](./examples/haf-todo/MIGRATION.md)
3. Run `cargo check` for compile-time feedback
4. Use the HAF linter (when available)

Remember: HAF's compile-time enforcement means if it compiles, your architecture is correct!