# WARP + Next.js Integration

This example shows how to use WARP components within an existing Next.js application.

## How it works

1. WARP components compile to WASM
2. Next.js loads WASM modules dynamically
3. WARP handles its own routing within designated areas
4. Server functions integrate with Next.js API routes

## Setup

```bash
# In your Next.js project
npm install @warp/nextjs

# In next.config.js
module.exports = {
  experimental: {
    warp: {
      // WARP components directory
      components: './warp-components',
      // Auto-generate API routes
      serverFunctions: true,
    }
  }
}
```

## Usage

```typescript
// app/page.tsx
import { WarpComponent } from '@warp/nextjs';

export default function Home() {
  return (
    <div>
      <h1>Next.js Page</h1>
      <WarpComponent 
        module="counter" 
        props={{ initial: 0 }}
      />
    </div>
  );
}
```

## Server Functions

WARP server functions automatically become Next.js API routes:

```rust
// counter.rs
#[warp::server]
async fn get_count() -> i32 {
    42
}
```

Becomes:

```
/api/warp/get_count
```

## Benefits

1. **Gradual Migration**: Start with one component
2. **Performance**: Critical paths in Rust/WASM
3. **Type Safety**: End-to-end types
4. **Hierarchical Structure**: Even within Next.js chaos