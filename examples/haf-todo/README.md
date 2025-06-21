# HAF Todo Example

This example demonstrates how to build applications using HAF (Hierarchical Architecture First) principles with Layer9.

## 🏗️ Architecture Overview

The application is organized into three distinct layers:

```
┌─────────────────────────────────────┐
│         L3: UI (Interface)          │
│  • User components (TodoHeader)     │
│  • Event handlers                   │
│  • Browser APIs                     │
└──────────────┬──────────────────────┘
               │ depends on
┌──────────────▼──────────────────────┐
│      L2: Runtime (Execution)        │
│  • State management (TodoStore)     │
│  • Side effects (Storage)           │
│  • Action dispatching               │
└──────────────┬──────────────────────┘
               │ depends on
┌──────────────▼──────────────────────┐
│      L1: Domain (Pure Logic)        │
│  • Todo data structures             │
│  • Pure reducer functions           │
│  • Business rules                   │
└─────────────────────────────────────┘
```

## 🎯 Key HAF Principles

### 1. **Unidirectional Dependencies**
- L3 can import from L2 and L1
- L2 can import from L1 only
- L1 cannot import from L2 or L3

### 2. **Pure Core**
The L1 domain layer contains only pure functions:
```rust
// Pure reducer - no side effects
pub fn reduce(self, action: TodoAction) -> TodoList {
    match action {
        TodoAction::Add { title } => self.add_todo(title),
        TodoAction::Toggle { id } => self.toggle_todo(id),
        // ...
    }
}
```

### 3. **Explicit Contracts**
Layer boundaries have explicit translation contracts:
```rust
// L1 → L2 Contract
impl L1ToL2Contract for ActionToCommandContract {
    type L1Type = TodoAction;
    type L2Type = Vec<Command>;
    
    fn translate(action: Self::L1Type) -> Self::L2Type {
        // Convert domain actions to runtime commands
    }
}
```

### 4. **Effects at the Edges**
All side effects are pushed to L2 (runtime) and L3 (UI):
- L1: Pure business logic only
- L2: Storage, logging, analytics
- L3: DOM manipulation, HTTP requests

## 📁 Project Structure

```
haf-todo/
├── src/
│   ├── l1_domain/      # Pure business logic
│   │   └── mod.rs      # Todo, TodoList, TodoAction
│   ├── l2_runtime/     # State and effects
│   │   └── mod.rs      # TodoStore, Storage, Analytics
│   ├── l3_ui/          # User interface
│   │   └── mod.rs      # Components, rendering
│   └── lib.rs          # Main entry point
├── Cargo.toml
├── index.html
└── README.md
```

## 🚀 Running the Example

1. **Build the WASM module:**
   ```bash
   wasm-pack build --target web --out-dir pkg
   ```

2. **Serve the files:**
   ```bash
   python3 -m http.server 8080
   ```

3. **Open in browser:**
   ```
   http://localhost:8080
   ```

## 🔍 Code Walkthrough

### L1: Domain Layer
Pure business logic with no dependencies:

```rust
// l1_domain/mod.rs
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
}

impl TodoList {
    // Pure function - returns new state
    pub fn reduce(self, action: TodoAction) -> TodoList {
        // No I/O, no side effects
    }
}
```

### L2: Runtime Layer
Manages state and coordinates effects:

```rust
// l2_runtime/mod.rs
pub struct TodoStore {
    state: RefCell<TodoList>,
    subscribers: RefCell<Vec<Box<dyn Fn(&TodoList)>>>,
}

impl TodoStore {
    pub fn dispatch(&self, action: TodoAction) {
        // Apply pure reducer
        let new_state = self.state.borrow().clone().reduce(action);
        
        // Side effects happen here
        Storage::save(&new_state);
        
        // Notify subscribers
        self.notify_subscribers(&new_state);
    }
}
```

### L3: UI Layer
Handles user interactions and rendering:

```rust
// l3_ui/mod.rs
haf_component!(L3, TodoItem, ItemProps, {
    // Component can use L2 and L1
    VNode::Element {
        tag: "li".to_string(),
        // ...
    }
});
```

## 📊 Benefits of HAF

1. **Clear Dependencies**: Always know where code belongs
2. **Testability**: Pure L1 logic is trivial to test
3. **Maintainability**: Changes don't ripple unexpectedly
4. **Onboarding**: New developers understand structure immediately
5. **Scalability**: Add layers as complexity grows (3→5→7→9)

## 🧪 Testing Strategy

Each layer has different testing approaches:

- **L1**: Unit tests with pure functions
- **L2**: Integration tests with mocked effects
- **L3**: Component tests with virtual DOM

```rust
#[test]
fn test_pure_domain_logic() {
    let list = TodoList::new();
    let list = list.reduce(TodoAction::Add { 
        title: "Test".to_string() 
    });
    assert_eq!(list.todos.len(), 1);
}
```

## 🔧 Compile-Time Enforcement

Layer9's HAF system prevents architectural violations at compile time:

```rust
// This won't compile - L1 can't use L2
// use crate::l2_runtime::TodoStore; // ❌ Error!

// This compiles - L2 can use L1
use crate::l1_domain::Todo; // ✅ OK
```

## 📚 Learn More

- [HAF Paper](../../PLAN/HAF_Paper_Composed.md)
- [HAF Runbook](../../PLAN/HAF_Runbook_Feynman_Composed.ko.md)
- [Layer9 3-Layer Architecture](../../PLAN/Layer9_3Layer_Architecture.md)

## 🤝 Contributing

When adding features, follow HAF principles:

1. Start with L1 - define pure data and logic
2. Add L2 runtime support for effects
3. Finally, create L3 UI components
4. Ensure dependencies flow downward only

---

*"Complexity is managed only through hierarchy"* - HAF Philosophy