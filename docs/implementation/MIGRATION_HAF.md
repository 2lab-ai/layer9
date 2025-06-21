# HAF Migration Guide for Layer9

## Table of Contents
1. [Overview](#overview)
2. [Pre-Migration Assessment](#pre-migration-assessment)
3. [Migration Strategy](#migration-strategy)
4. [Step-by-Step Migration](#step-by-step-migration)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)
7. [Verification](#verification)

## Overview

This guide helps you migrate your existing Layer9 application to the HAF (Hierarchical Architecture First) system. HAF enforces clean architecture through compile-time guarantees, ensuring proper separation of concerns across three layers:

- **L1 (Core)**: Pure business logic, no side effects
- **L2 (Runtime)**: Effect management and orchestration
- **L3 (Framework)**: External interfaces and I/O

## Pre-Migration Assessment

### 1. Run HAF Linter

First, assess your current codebase:

```bash
layer9 haf-lint --strict
```

This will identify:
- Layer dependency violations
- Side effects in pure code
- Missing layer annotations
- Mixed concerns in modules

### 2. Analyze Violations

Save the linter output to a file for reference:

```bash
layer9 haf-lint > haf-violations.txt
```

Common violations:
- **L1 → L2/L3 dependencies**: Pure code importing runtime/framework modules
- **Side effects in L1**: Console logging, file I/O, or mutations in pure functions
- **Mixed concerns**: Structs with both pure data and runtime state

### 3. Create Migration Plan

Based on violations, prioritize:
1. **Critical**: L1 code with side effects (breaks purity)
2. **High**: Wrong layer dependencies
3. **Medium**: Missing annotations
4. **Low**: Code organization issues

## Migration Strategy

### Option 1: Gradual Migration (Recommended)

Use the compatibility layer to migrate incrementally:

```toml
# Cargo.toml
[features]
haf-migration = ["layer9/haf-compat"]
```

### Option 2: Big Bang Migration

For smaller codebases, migrate everything at once:

```bash
layer9 haf-refactor --dry-run  # Preview changes
layer9 haf-refactor             # Apply refactoring
```

## Step-by-Step Migration

### Step 1: Add Layer Annotations

Add layer annotations to all modules:

```rust
//! L1: Core - Pure business logic

// or

//! L2: Runtime - Effect management

// or  

//! L3: Framework - External interfaces
```

### Step 2: Separate Pure Logic (L1)

Extract pure functions and data structures:

```rust
// Before: Mixed concerns
pub struct TodoList {
    items: Vec<Todo>,
    db: Database,  // ❌ I/O reference in L1
}

impl TodoList {
    pub fn add_todo(&mut self, text: String) {
        let todo = Todo::new(text);
        self.items.push(todo.clone());
        self.db.save(&todo).unwrap(); // ❌ Side effect
    }
}

// After: Separated layers
// L1: Pure logic
pub struct TodoList {
    items: Vec<Todo>,
}

impl TodoList {
    pub fn add_todo(&self, text: String) -> TodoList {
        let mut items = self.items.clone();
        items.push(Todo::new(text));
        TodoList { items }
    }
}

// L2: Runtime
pub struct TodoRuntime {
    state: TodoList,
}

impl TodoRuntime {
    pub fn add_todo(&mut self, text: String) -> Contract<TodoList, SaveCommand> {
        let new_state = self.state.add_todo(text);
        let command = SaveCommand { 
            todo: new_state.items.last().unwrap().clone() 
        };
        self.state = new_state.clone();
        Contract::new(new_state, command)
    }
}

// L3: Framework
pub struct TodoService {
    runtime: TodoRuntime,
    db: Database,
}

impl TodoService {
    pub async fn add_todo(&mut self, text: String) -> Result<(), Error> {
        let contract = self.runtime.add_todo(text);
        self.db.save(&contract.output.todo).await?;
        Ok(())
    }
}
```

### Step 3: Migrate Components

Convert old components to HAF components:

```rust
// Before: Old component
impl Component for Counter {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                on_click: Some(Rc::new(|| {
                    println!("Clicked!"); // ❌ Side effect in render
                })),
                ..Default::default()
            },
            children: vec![],
        }
    }
}

// After: HAF component
// L1: Pure component
impl PureComponent<L1> for Counter {
    type Props = CounterProps;
    
    fn render(&self, props: &Self::Props) -> VNode<L1> {
        VNode::Element {
            tag: "div".to_string(),
            props: VProps {
                events: vec![("click".to_string(), EventId(1))],
                ..Default::default()
            },
            children: vec![
                VNode::Text(format!("Count: {}", props.count))
            ],
        }
    }
}

// L2: Effect handling
impl ComponentRuntime<L2> {
    fn handle_event(&mut self, event_id: EventId) -> Vec<Effect> {
        match event_id.0 {
            1 => vec![Effect::Log("Clicked!".to_string())],
            _ => vec![],
        }
    }
}
```

### Step 4: Create Contracts

Define contracts for cross-layer communication:

```rust
// L1 → L2 Contract
pub struct StateUpdateContract;

impl L1ToL2Contract for StateUpdateContract {
    type L1Type = AppState;
    type L2Type = UpdateCommand;
    
    fn translate(state: Self::L1Type) -> Self::L2Type {
        UpdateCommand {
            new_state: state,
            timestamp: now(),
        }
    }
}

// L2 → L3 Contract  
pub struct ApiContract;

impl L2ToL3Contract for ApiContract {
    type L2Type = UpdateCommand;
    type L3Type = HttpRequest;
    
    fn translate(cmd: Self::L2Type) -> Self::L3Type {
        HttpRequest {
            method: "POST".to_string(),
            url: "/api/state".to_string(),
            body: serde_json::to_string(&cmd.new_state).unwrap(),
        }
    }
}
```

### Step 5: Migrate State Management

Convert mutable state to immutable transformations:

```rust
// Before: Mutable state
pub struct AppState {
    count: RefCell<i32>,
}

impl AppState {
    pub fn increment(&self) {
        *self.count.borrow_mut() += 1;
    }
}

// After: Immutable state
// L1: Pure state
#[derive(Clone)]
pub struct AppState {
    count: i32,
}

impl AppState {
    pub fn increment(&self) -> AppState {
        AppState {
            count: self.count + 1,
        }
    }
}

// L2: State management
pub struct StateManager {
    current: AppState,
    history: Vec<AppState>,
}

impl StateManager {
    pub fn update<F>(&mut self, f: F) -> Contract<AppState, StateChange>
    where
        F: FnOnce(&AppState) -> AppState,
    {
        let new_state = f(&self.current);
        let change = StateChange {
            from: self.current.clone(),
            to: new_state.clone(),
        };
        
        self.history.push(self.current.clone());
        self.current = new_state.clone();
        
        Contract::new(new_state, change)
    }
}
```

### Step 6: Update Imports

Fix import paths to respect layer boundaries:

```rust
// Before
use crate::{
    component::Component,
    websocket::WebSocket,  // ❌ L3 import in L1
    db::query,             // ❌ L3 import in L1
};

// After
// L1 file
use crate::haf::{
    layers::L1,
    component::{PureComponent, VNode},
};

// L2 file  
use crate::haf::{
    layers::{L1, L2},
    runtime::{Effect, StateManager},
    l1_core::AppState, // ✅ L2 can import from L1
};

// L3 file
use crate::haf::{
    layers::{L1, L2, L3},
    framework::{WebSocket, Database},
    l2_runtime::StateManager, // ✅ L3 can import from L2
    l1_core::AppState,       // ✅ L3 can import from L1
};
```

## Common Patterns

### Pattern 1: Effect Extraction

```rust
// Before: Effect in pure function
fn calculate_and_log(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Result: {}", result); // ❌ Side effect
    result
}

// After: Pure function + effect descriptor
// L1
fn calculate(a: i32, b: i32) -> (i32, Vec<Effect>) {
    let result = a + b;
    let effects = vec![Effect::Log(format!("Result: {}", result))];
    (result, effects)
}

// L2
fn execute_calculation(a: i32, b: i32) -> i32 {
    let (result, effects) = calculate(a, b);
    for effect in effects {
        runtime.execute_effect(effect);
    }
    result
}
```

### Pattern 2: Dependency Injection

```rust
// Before: Direct dependencies
struct UserService {
    db: Database,
    cache: Cache,
    http: HttpClient,
}

// After: Layer-appropriate dependencies
// L1
struct UserLogic;
impl UserLogic {
    fn validate_user(user: &User) -> Result<(), ValidationError> {
        // Pure validation
    }
}

// L2  
struct UserRuntime {
    logic: UserLogic,
    cache: CacheHandle, // Runtime cache reference
}

// L3
struct UserService {
    runtime: UserRuntime,
    db: Database,
    http: HttpClient,
}
```

### Pattern 3: Event Handling

```rust
// Before: Direct event handling
button.on_click(|e| {
    fetch("/api/data").then(|data| {
        update_ui(data);
    });
});

// After: Layer-separated handling
// L1: Event definition
#[derive(Clone)]
pub enum AppEvent {
    ButtonClick,
    DataFetched(Data),
}

// L2: Event processing
impl EventProcessor {
    fn process(&mut self, event: AppEvent) -> Vec<Command> {
        match event {
            AppEvent::ButtonClick => vec![Command::FetchData],
            AppEvent::DataFetched(data) => vec![Command::UpdateUI(data)],
        }
    }
}

// L3: Event execution
impl EventExecutor {
    async fn execute(&mut self, command: Command) {
        match command {
            Command::FetchData => {
                let data = fetch("/api/data").await;
                self.processor.process(AppEvent::DataFetched(data));
            }
            Command::UpdateUI(data) => {
                update_ui(data);
            }
        }
    }
}
```

## Troubleshooting

### Common Errors

1. **"trait bound `L1: CanDepend<L2>` is not satisfied"**
   - You're trying to use L2 functionality in L1 code
   - Solution: Move the code to L2 or extract pure logic

2. **"cannot find type `WebSocket` in L1 scope"**
   - WebSocket is L3-only
   - Solution: Create abstraction in L2, implement in L3

3. **"Component doesn't implement Default"**
   - HAF components need Default
   - Solution: Add `#[derive(Default)]` or implement manually

### Migration Checklist

- [ ] All modules have layer annotations
- [ ] No side effects in L1 code
- [ ] No upward dependencies (L1→L2, L2→L3)
- [ ] All cross-layer communication uses contracts
- [ ] State transformations are immutable
- [ ] Effects are properly tracked and executed
- [ ] Tests pass with HAF enabled

## Verification

### 1. Run HAF Linter

```bash
layer9 haf-lint --strict
```

Should show: "✓ No HAF violations found!"

### 2. Run Tests

```bash
cargo test --features haf-strict
```

### 3. Build with HAF

```bash
cargo build --features haf-strict
```

### 4. Runtime Verification

```rust
#[test]
fn test_haf_compliance() {
    // Verify layer separation
    assert_layer_purity::<L1>();
    assert_no_upward_deps();
    assert_contracts_valid();
}
```

## Next Steps

After migration:

1. **Enable HAF enforcement in CI**
   ```yaml
   - run: layer9 haf-lint --strict
   - run: cargo test --features haf-strict
   ```

2. **Document architectural decisions**
   - Why certain code is in each layer
   - Contract design rationale
   - Effect handling strategies

3. **Train team on HAF principles**
   - Layer responsibilities
   - Pure function benefits
   - Contract-based design

4. **Monitor and maintain**
   - Regular HAF linting
   - Refactor violations immediately
   - Keep contracts up to date

## Resources

- [HAF Philosophy](./PLAN/00-HAF-PHILOSOPHY.md)
- [HAF Examples](./examples/haf-todo/)
- [HAF API Docs](./crates/core/src/haf/README.md)
- [Layer9 Architecture](./ARCHITECTURE.md)

Remember: HAF's compile-time enforcement means if it compiles, your architecture is correct!