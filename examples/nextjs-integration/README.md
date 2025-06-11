# Layer9 + Next.js Integration

This example shows how to use Layer9 components within an existing Next.js application.

## How it works

1. Layer9 components compile to WASM
2. Next.js loads WASM modules dynamically
3. Layer9 handles its own routing within designated areas
4. Server functions integrate with Next.js API routes

## Setup

```bash
# In your Next.js project
npm install @layer9/nextjs

# In next.config.js
module.exports = {
  experimental: {
    layer9: {
      // Layer9 components directory
      components: './layer9-components',
      // Auto-generate API routes
      serverFunctions: true,
    }
  }
}
```

## Usage

```typescript
// app/page.tsx
import { Layer9Component } from '@layer9/nextjs';

export default function Home() {
  return (
    <div>
      <h1>Next.js Page</h1>
      <Layer9Component 
        module="counter" 
        props={{ initial: 0 }}
      />
    </div>
  );
}
```

## Server Functions

Layer9 server functions automatically become Next.js API routes:

```rust
// counter.rs
#[layer9::server]
async fn get_count() -> i32 {
    42
}
```

Becomes:

```
/api/layer9/get_count
```

## Benefits

1. **Gradual Migration**: Start with one component
2. **Performance**: Critical paths in Rust/WASM
3. **Type Safety**: End-to-end types
4. **Hierarchical Structure**: Even within Next.js chaos