# Layer9 HAF Violation Examples

## Concrete Code Examples of HAF Violations

### Example 1: Middleware (L3) Importing Auth (L7)

**File**: `crates/core/src/middleware.rs`
```rust
use crate::auth::User;  // L3 importing from L7 - VIOLATION!
use crate::prelude::*;
```

**Why it's wrong**: Middleware is infrastructure (L3) but Auth contains business logic (L7). Lower layers cannot depend on higher layers.

**Correct approach**:
```rust
// L3 should define a contract
pub trait UserProvider {
    type User;
    fn get_user(&self) -> Option<Self::User>;
}

// L7 implements the contract
impl UserProvider for AuthService {
    type User = auth::User;
    // ...
}
```

### Example 2: VDOM (L4) Knowing About Components (L5)

**File**: `crates/core/src/vdom.rs`
```rust
use crate::component::{Component, Element}; // L4 importing from L5 - VIOLATION!

pub fn diff_and_patch(old: &Element, new: &Element) {
    match (old, new) {
        (Element::Component(old_comp), Element::Component(new_comp)) => {
            // VDOM shouldn't know about Component trait
        }
    }
}
```

**Why it's wrong**: VDOM is runtime infrastructure but Component is a higher-level abstraction.

**Correct approach**:
```rust
// L4 defines a generic node contract
pub trait VNode {
    fn node_type(&self) -> NodeType;
    fn children(&self) -> &[Box<dyn VNode>];
}

// L5 implements VNode for Components
impl VNode for Component {
    // ...
}
```

### Example 3: Database Layer Mixed With Business Logic

**File**: `crates/core/src/db.rs`
```rust
// Database layer (L2/L3) but contains business logic
pub struct User {
    pub id: i32,
    pub email: String,
    pub is_premium: bool,  // Business logic in DB layer!
}

impl User {
    pub fn can_access_feature(&self) -> bool {  // Business logic!
        self.is_premium
    }
}
```

**Why it's wrong**: Database entities should be pure data structures. Business logic belongs in L7.

**Correct approach**:
```rust
// L3: Pure data entity
pub struct UserEntity {
    pub id: i32,
    pub email: String,
    pub subscription_type: String,
}

// L7: Business model with logic
pub struct User {
    entity: UserEntity,
}

impl User {
    pub fn can_access_feature(&self) -> bool {
        self.entity.subscription_type == "premium"
    }
}
```

### Example 4: Router Directly Creating Components

**File**: `crates/core/src/router.rs`
```rust
use crate::component::{Component, Element, Props}; // L7 tightly coupled to L5

pub struct Page {
    pub component: Box<dyn Component>, // Direct dependency!
}

impl Router {
    fn render_page(&self, page: Page) {
        let element = page.component.render(); // Router rendering components!
    }
}
```

**Why it's wrong**: Router (application layer) shouldn't directly handle component rendering.

**Correct approach**:
```rust
// L7: Router works with abstract handlers
pub trait RouteHandler {
    fn handle(&self, params: RouteParams) -> Response;
}

// L5: Component layer provides handler implementation
pub struct ComponentHandler<C: Component> {
    component: C,
}

impl<C: Component> RouteHandler for ComponentHandler<C> {
    fn handle(&self, params: RouteParams) -> Response {
        // Component-specific handling
    }
}
```

### Example 5: Form System Without Clear Boundaries

**Files**: `form.rs`, `form_builder.rs`, `form_traits.rs`
```rust
// form.rs - Mixed concerns
pub struct Form {
    fields: Vec<Field>,        // Data structure (L3)
    validation: Validation,    // Business logic (L7)
    component: FormComponent,  // UI (L5)
    submit_handler: Handler,   // HTTP (L4)
}
```

**Why it's wrong**: Form system spans multiple layers without clear separation.

**Correct approach**:
```rust
// L3: Form data structure
pub struct FormData {
    fields: HashMap<String, Value>,
}

// L4: Form submission service
pub trait FormSubmitter {
    async fn submit(&self, data: FormData) -> Result<Response>;
}

// L5: Form UI component
pub struct FormComponent {
    data: FormData,
    submitter: Box<dyn FormSubmitter>,
}

// L7: Form validation rules
pub struct FormValidator {
    rules: Vec<ValidationRule>,
}
```

### Example 6: Flat Imports in lib.rs

**File**: `crates/core/src/lib.rs`
```rust
pub mod prelude {
    // Everything at the same level - no hierarchy!
    pub use crate::api_docs::*;
    pub use crate::app::*;
    pub use crate::auth::*;
    pub use crate::cache::*;
    pub use crate::component::*;
    pub use crate::db::*;
    pub use crate::router::*;
    // ... 40+ more modules
}
```

**Why it's wrong**: Prelude exports everything at once, breaking layer isolation.

**Correct approach**:
```rust
// Each layer has its own prelude
pub mod l3_prelude {
    pub use crate::runtime::*;
    pub use crate::vdom::*;
}

pub mod l5_prelude {
    pub use crate::components::*;
    pub use crate::props::*;
}

pub mod l7_prelude {
    pub use crate::app::*;
    pub use crate::routes::*;
}
```

### Example 7: Image System Scattered Across Layers

**Files**: `image.rs`, `image_lazy.rs`, `image_transform.rs`, `image_handler.rs`
```rust
// image.rs - Component (L5)
pub struct Image { 
    src: String,
    lazy: bool,  // Mixing feature concern
}

// image_transform.rs - Service (L4)
pub fn transform_image(data: &[u8]) -> Vec<u8> {
    // Should be in L4 service
}

// image_lazy.rs - Feature (L6) mixed with component
use crate::component::Component;  // Feature importing component!
```

**Why it's wrong**: Image functionality scattered without clear ownership.

**Correct approach**:
```rust
// L4: Image service
pub mod image_service {
    pub trait ImageProcessor {
        fn process(&self, data: &[u8]) -> Result<Vec<u8>>;
    }
}

// L5: Image component
pub mod image_component {
    pub struct Image {
        src: String,
        processor: Box<dyn ImageProcessor>,
    }
}

// L6: Lazy loading feature
pub mod lazy_loading {
    pub trait LazyLoadable {
        fn should_load(&self) -> bool;
    }
}
```

## Summary of Violations

1. **27 files** have cross-layer imports
2. **No clear contracts** between layers
3. **Business logic** mixed with infrastructure
4. **UI concerns** mixed with data
5. **Services** directly coupled to implementations
6. **Features** not properly isolated
7. **Flat structure** prevents proper layering

These examples demonstrate why the current architecture makes it difficult to:
- Test layers in isolation
- Swap implementations
- Understand boundaries
- Maintain the codebase
- Scale the framework