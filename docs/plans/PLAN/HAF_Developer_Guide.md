# HAF Developer Guide for Layer9

## 🎯 Introduction

Welcome to the HAF (Hierarchical Architecture First) developer guide for Layer9. This guide will help you build maintainable, scalable applications by enforcing architectural boundaries at compile time.

## 📚 Table of Contents

1. [Core Concepts](#core-concepts)
2. [Getting Started](#getting-started)
3. [Layer Definitions](#layer-definitions)
4. [Writing HAF Components](#writing-haf-components)
5. [Translation Contracts](#translation-contracts)
6. [Best Practices](#best-practices)
7. [Common Patterns](#common-patterns)
8. [Migration Guide](#migration-guide)
9. [Troubleshooting](#troubleshooting)
10. [Tools and Automation](#tools-and-automation)

## 🌟 Core Concepts

### The HAF Philosophy

> "Complexity is managed only through hierarchy"

HAF reduces software complexity from O(n²) to O(n log n) by enforcing strict hierarchical organization.

### The Three Fundamental Rules

1. **Dependencies flow downward only**: L3 → L2 → L1
2. **Each layer has a single responsibility**
3. **Explicit contracts at layer boundaries**

### Layer Overview

```
┌─────────────────────────────────────┐
│      L3: Framework (Interface)      │  ← User-facing APIs, HTTP, CLI
├─────────────────────────────────────┤
│       L2: Runtime (Execution)       │  ← State management, Effects
├─────────────────────────────────────┤
│        L1: Core (Pure Logic)        │  ← Business rules, Algorithms
└─────────────────────────────────────┘
```

## 🚀 Getting Started

### 1. Add HAF to Your Project

```toml
# Cargo.toml
[dependencies]
layer9-core = { version = "0.1", features = ["haf"] }
```

### 2. Import HAF Types

```rust
use layer9_core::haf::{
    layers::{L1, L2, L3},
    Component, Service,
    haf_component, haf_service,
    L1ToL2Contract, L2ToL3Contract,
};
```

### 3. Create Your First HAF Component

```rust
// L1: Pure business logic
#[derive(Clone, Debug)]
pub struct Counter {
    pub value: i32,
}

impl Counter {
    pub fn increment(self) -> Counter {
        Counter { value: self.value + 1 }
    }
}

// L2: State management
haf_service!(L2, counter_service, {
    pub fn create_counter() -> Rc<RefCell<Counter>> {
        Rc::new(RefCell::new(Counter { value: 0 }))
    }
    
    pub fn increment(counter: &RefCell<Counter>) -> () {
        let new_value = counter.borrow().clone().increment();
        *counter.borrow_mut() = new_value;
    }
});

// L3: UI Component
haf_component!(L3, CounterButton, CounterProps, {
    VNode::Element {
        tag: "button".to_string(),
        props: Props {
            attributes: vec![
                ("onclick".to_string(), "increment".to_string()),
            ],
        },
        children: vec![
            VNode::Text(format!("Count: {}", props.value)),
        ],
    }
});
```

## 📋 Layer Definitions

### Layer 1: Core (Pure Business Logic)

**Responsibilities:**
- Data structures and types
- Pure functions and algorithms
- Business rules and validations
- Domain models

**Restrictions:**
- No I/O operations
- No side effects
- No external dependencies
- No framework-specific code

**Example:**
```rust
// ✅ GOOD: Pure function
pub fn calculate_discount(price: f64, percentage: f64) -> f64 {
    price * (1.0 - percentage / 100.0)
}

// ❌ BAD: Side effect
pub fn save_order(order: Order) {
    database::save(order); // No I/O in L1!
}
```

### Layer 2: Runtime (Execution Environment)

**Responsibilities:**
- State management
- Side effect coordination
- Resource management
- Platform abstraction

**Can Use:**
- L1 types and functions
- Platform APIs (with abstraction)
- Async operations

**Example:**
```rust
// L2 service managing state and effects
haf_service!(L2, order_service, {
    pub fn place_order(items: Vec<Item>) -> Result<OrderId, Error> {
        // Use L1 pure logic
        let order = Order::new(items);
        let total = order.calculate_total();
        
        // Perform side effects
        let id = database::save_order(&order)?;
        email::send_confirmation(&order)?;
        
        Ok(id)
    }
});
```

### Layer 3: Framework (External Interface)

**Responsibilities:**
- User interfaces (CLI, GUI, API)
- HTTP handling
- External service integration
- Framework-specific code

**Can Use:**
- L1 and L2 functionality
- External libraries
- Framework features

**Example:**
```rust
// L3 HTTP handler
haf_service!(L3, api_service, {
    pub fn handle_order_request(req: HttpRequest) -> HttpResponse {
        let items = parse_items(req.body);
        
        match order_service::place_order(items) {
            Ok(id) => HttpResponse::ok(json!({ "order_id": id })),
            Err(e) => HttpResponse::error(e.to_string()),
        }
    }
});
```

## 🔧 Writing HAF Components

### Component Structure

```rust
// Define props
#[derive(Clone)]
pub struct TodoItemProps {
    pub todo: Todo,
    pub on_toggle: Rc<dyn Fn(usize)>,
}

// Create component with layer tag
haf_component!(L3, TodoItem, TodoItemProps, {
    let class = if props.todo.completed { 
        "completed" 
    } else { 
        "active" 
    };
    
    VNode::Element {
        tag: "li".to_string(),
        props: Props {
            attributes: vec![
                ("class".to_string(), class.to_string()),
            ],
        },
        children: vec![
            VNode::Text(props.todo.title.clone()),
        ],
    }
});
```

### Service Definition

```rust
haf_service!(L2, todo_store, {
    pub fn add_todo(title: String) -> Result<Todo, Error> {
        // Validate using L1 logic
        validate_title(&title)?;
        
        // Create todo
        let todo = Todo::new(title);
        
        // Save to storage (side effect)
        storage::save(&todo)?;
        
        Ok(todo)
    }
    
    pub fn get_todos() -> Vec<Todo> {
        storage::load_all()
    }
});
```

## 🔄 Translation Contracts

Contracts make layer boundaries explicit and type-safe.

### Defining a Contract

```rust
// L1 → L2 Contract
pub struct ActionToEffectContract;

impl L1ToL2Contract for ActionToEffectContract {
    type L1Type = UserAction;
    type L2Type = Vec<Effect>;
    
    fn translate(action: Self::L1Type) -> Self::L2Type {
        match action {
            UserAction::Login { username, password } => vec![
                Effect::Authenticate { username, password },
                Effect::LoadUserData,
                Effect::UpdateUI,
            ],
            UserAction::Logout => vec![
                Effect::ClearSession,
                Effect::RedirectToHome,
            ],
        }
    }
}
```

### Using Contracts

```rust
// In L2 runtime
pub fn handle_action(action: UserAction) {
    let effects = ActionToEffectContract::translate(action);
    
    for effect in effects {
        execute_effect(effect);
    }
}
```

## ✨ Best Practices

### 1. Start with L1

Always begin by modeling your domain in L1:
```rust
// Define your domain first
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub price: Money,
}

// Pure business logic
impl Product {
    pub fn apply_discount(self, discount: Discount) -> Product {
        Product {
            price: self.price.apply(discount),
            ..self
        }
    }
}
```

### 2. Keep Layers Thin

Each layer should do one thing well:
- L1: What (domain logic)
- L2: How (orchestration)
- L3: When/Where (interface)

### 3. Use Type-Safe IDs

```rust
// Type-safe IDs prevent mixing
pub struct UserId(Uuid);
pub struct ProductId(Uuid);

// Can't accidentally use ProductId where UserId is expected
```

### 4. Explicit Error Handling

```rust
// L1: Domain errors
pub enum DomainError {
    InvalidPrice,
    OutOfStock,
}

// L2: Runtime errors
pub enum RuntimeError {
    Domain(DomainError),
    Database(DbError),
    Network(NetworkError),
}

// L3: API errors
pub enum ApiError {
    BadRequest(String),
    Internal(RuntimeError),
}
```

## 🎨 Common Patterns

### Repository Pattern (L2)

```rust
haf_service!(L2, user_repository, {
    pub fn find_by_id(id: UserId) -> Result<User, Error> {
        let data = database::query("SELECT * FROM users WHERE id = ?", &[id])?;
        Ok(User::from_row(data))
    }
    
    pub fn save(user: &User) -> Result<(), Error> {
        database::execute(
            "INSERT OR UPDATE users SET ...", 
            user.to_params()
        )
    }
});
```

### Command Pattern (L1 → L2)

```rust
// L1: Pure command definition
pub enum Command {
    CreateUser { name: String, email: Email },
    UpdateProfile { id: UserId, changes: ProfileChanges },
    DeleteAccount { id: UserId },
}

// L2: Command handler
pub fn handle_command(cmd: Command) -> Result<(), Error> {
    match cmd {
        Command::CreateUser { name, email } => {
            let user = User::new(name, email);
            user_repository::save(&user)?;
            email_service::send_welcome(&user)?;
            Ok(())
        }
        // ... other commands
    }
}
```

### Event Sourcing (L1 + L2)

```rust
// L1: Pure event definitions
pub enum Event {
    UserCreated { id: UserId, name: String },
    ProfileUpdated { id: UserId, changes: ProfileChanges },
}

// L1: Pure state reduction
pub fn apply_event(state: AppState, event: Event) -> AppState {
    match event {
        Event::UserCreated { id, name } => {
            state.add_user(User::new(id, name))
        }
        // ... other events
    }
}

// L2: Event store
haf_service!(L2, event_store, {
    pub fn append(event: Event) -> Result<(), Error> {
        let serialized = serde_json::to_string(&event)?;
        database::insert("events", &serialized)?;
        Ok(())
    }
});
```

## 🔄 Migration Guide

### Step 1: Identify Layers

Map your existing code to layers:
```
src/
├── models/        → L1 (if pure)
├── services/      → L2 (if has side effects)
├── controllers/   → L3 (HTTP handlers)
├── views/         → L3 (UI components)
└── utils/         → Depends on what they do
```

### Step 2: Extract Pure Logic

Move business logic to L1:
```rust
// Before (mixed concerns)
pub async fn process_order(order: Order) -> Result<Receipt> {
    let total = order.items.iter().map(|i| i.price).sum(); // Pure
    database::save_order(&order).await?; // Side effect
    let receipt = Receipt::new(order.id, total); // Pure
    email::send_receipt(&receipt).await?; // Side effect
    Ok(receipt)
}

// After (separated)
// L1: Pure logic
pub fn calculate_total(items: &[Item]) -> Money {
    items.iter().map(|i| i.price).sum()
}

pub fn create_receipt(order_id: OrderId, total: Money) -> Receipt {
    Receipt::new(order_id, total)
}

// L2: Orchestration
pub async fn process_order(order: Order) -> Result<Receipt> {
    let total = calculate_total(&order.items);
    database::save_order(&order).await?;
    let receipt = create_receipt(order.id, total);
    email::send_receipt(&receipt).await?;
    Ok(receipt)
}
```

### Step 3: Define Contracts

Create explicit boundaries:
```rust
// Contract for order processing
impl L1ToL2Contract for OrderProcessingContract {
    type L1Type = Order;
    type L2Type = ProcessingSteps;
    
    fn translate(order: Self::L1Type) -> Self::L2Type {
        ProcessingSteps {
            validate: ValidationStep::new(&order),
            persist: PersistStep::new(&order),
            notify: NotificationStep::new(&order),
        }
    }
}
```

## 🐛 Troubleshooting

### Common Errors

#### 1. "Cannot use L2 types in L1"
```rust
// ❌ ERROR: L1 cannot depend on L2
use crate::l2_runtime::DatabaseConnection; // In L1 file

// ✅ SOLUTION: Pass data, not connections
pub fn process_data(data: Vec<Record>) -> ProcessedData {
    // Work with data, not database
}
```

#### 2. "Circular dependency detected"
```rust
// ❌ ERROR: Circular dependency
// L2 module
use crate::l3_api::ApiResponse;

// L3 module  
use crate::l2_runtime::RuntimeState;

// ✅ SOLUTION: Use contracts
// Define shared types in L1 or use translation contracts
```

#### 3. "Side effect in pure layer"
```rust
// ❌ ERROR: I/O in L1
pub fn validate_user(email: &str) -> bool {
    database::exists("users", email) // No I/O in L1!
}

// ✅ SOLUTION: Return validation rules
pub fn email_validation_rules() -> ValidationRules {
    ValidationRules {
        pattern: EMAIL_REGEX,
        min_length: 5,
        max_length: 255,
    }
}
```

### Debugging Tips

1. **Use layer tags in logs:**
   ```rust
   log::debug!("[L2] Processing order: {:?}", order);
   ```

2. **Validate layer boundaries in tests:**
   ```rust
   #[test]
   fn test_no_upward_dependencies() {
       let deps = analyze_dependencies();
       assert!(deps.verify_haf_compliance());
   }
   ```

3. **Use type markers:**
   ```rust
   pub struct L1Data<T> {
       _marker: PhantomData<L1>,
       data: T,
   }
   ```

## 🛠️ Tools and Automation

### HAF CLI

```bash
# Check for layer violations
layer9 haf check

# Visualize dependencies
layer9 haf graph

# Generate layer template
layer9 haf new --layer L1 --name domain

# Auto-fix common violations
layer9 haf fix
```

### VS Code Extension

Install the Layer9 HAF extension for:
- Real-time layer violation detection
- Auto-completion for HAF macros
- Go-to-definition across layers
- Inline contract documentation

### Build-Time Validation

```toml
# Cargo.toml
[package.metadata.haf]
strict = true
max_layer_depth = 3
auto_fix = false

[[package.metadata.haf.rules]]
name = "no-io-in-l1"
severity = "error"

[[package.metadata.haf.rules]]
name = "explicit-contracts"
severity = "warning"
```

### Continuous Integration

```yaml
# .github/workflows/haf.yml
name: HAF Compliance
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run HAF checks
        run: |
          cargo install layer9-cli
          layer9 haf check --fail-on-warnings
          layer9 haf test
```

## 📖 Advanced Topics

### Dynamic Layer Growth

As your application grows, add layers:

```
3 layers (1-5 developers):
L1: Domain
L2: Runtime  
L3: Interface

5 layers (5-50 developers):
L1: Types
L2: Domain
L3: Services
L4: Transport
L5: Clients

7 layers (50-500 developers):
[Add infrastructure and platform layers]
```

### Cross-Service Communication

When services need to communicate:

```rust
// Service A (L2)
haf_service!(L2, service_a, {
    pub fn get_data() -> ServiceAData {
        // ...
    }
});

// Service B (L2) 
haf_service!(L2, service_b, {
    pub fn process() -> Result<(), Error> {
        // Use contract to translate between services
        let data = ServiceAToB::translate(service_a::get_data());
        // ...
    }
});
```

### Performance Considerations

1. **Layer boundaries have zero runtime cost** - they're compile-time only
2. **Use batch operations** to minimize layer crossings
3. **Cache computed values** in L2 when appropriate

## 🎓 Learning Resources

1. **HAF Katas**: Practice exercises at `/examples/haf-katas/`
2. **Video Tutorials**: [YouTube Playlist](https://youtube.com/layer9-haf)
3. **Community**: [Discord](https://discord.gg/layer9)
4. **Blog Posts**: [layer9.dev/blog/haf](https://layer9.dev/blog/haf)

## 🤝 Contributing

When contributing to Layer9:

1. **Follow HAF principles** in all code
2. **Update contracts** when changing layer interfaces
3. **Add tests** for layer boundary compliance
4. **Document** layer decisions in code comments

---

*Remember: "Complexity is managed only through hierarchy." Let HAF guide you to cleaner, more maintainable code.*