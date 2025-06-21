# Authentication Migration Guide

This guide helps you migrate from the old MockAuthProvider default to the new JWT-based authentication system.

## Overview

The authentication system now uses JWT authentication by default instead of MockAuthProvider. This makes the system production-ready out of the box while maintaining backward compatibility for testing.

## Key Changes

### 1. Default AuthService Behavior

**Before:**
```rust
// Always returned MockAuthProvider
let auth_service = AuthService::default();
```

**After:**
```rust
// Returns JwtAuthProvider by default, MockAuthProvider only when configured
let auth_service = AuthService::default();
```

### 2. Configuration System

A new configuration system has been added to manage authentication settings:

```rust
use layer9_core::config::{Config, ConfigBuilder};

// Use global configuration
let config = Config::get_or_init();

// Or create custom configuration
let config = ConfigBuilder::new()
    .jwt_secret("your-secret-key".to_string())
    .use_mock_auth(false)
    .build();
```

## Migration Steps

### Step 1: Update Environment Variables

Add the following to your `.env` file:

```env
# JWT secret for production (minimum 32 characters recommended)
LAYER9_JWT_SECRET=your-production-jwt-secret-key-here

# Set to true only for testing/development
LAYER9_USE_MOCK_AUTH=false
```

### Step 2: Update Test Code

For tests that require MockAuthProvider, explicitly enable it:

```rust
// Option 1: Use mock provider directly
let auth_service = AuthService::with_mock_provider();

// Option 2: Use configuration
let config = ConfigBuilder::new()
    .use_mock_auth(true)
    .build();
let auth_service = AuthService::with_config(&config);

// Option 3: Set environment variable in test
std::env::set_var("LAYER9_USE_MOCK_AUTH", "true");
let auth_service = AuthService::default();
```

### Step 3: Configure Browser Applications

For browser-based applications, configure via JavaScript:

```html
<script>
window.LAYER9_CONFIG = {
    jwtSecret: "your-production-jwt-secret",
    useMockAuth: false  // Set to true for development
};
</script>
```

### Step 4: Update Production Code

No changes needed for production code using `AuthService::default()`. It will automatically use JWT authentication with the configured secret.

## Security Considerations

1. **JWT Secret**: Always use a strong, unique secret in production (minimum 32 characters)
2. **Environment Variables**: Never commit `.env` files with real secrets
3. **Default Secret**: The system provides a default secret for development only. A warning is logged when using it.
4. **Mock Auth**: Never enable `LAYER9_USE_MOCK_AUTH` in production

## Backward Compatibility

- Existing code using `AuthService::with_mock_provider()` continues to work
- Existing code using `AuthService::with_jwt_provider(secret)` continues to work
- Tests can still use MockAuthProvider by setting the configuration

## Examples

### Production Setup

```rust
// Automatically uses JWT with configured secret
let auth_service = AuthService::default();
```

### Development Setup

```rust
// Use mock auth for easier development
std::env::set_var("LAYER9_USE_MOCK_AUTH", "true");
let auth_service = AuthService::default();
```

### Custom Configuration

```rust
let config = ConfigBuilder::new()
    .jwt_secret("custom-secret".to_string())
    .build();
let auth_service = AuthService::with_config(&config);
```

## Troubleshooting

### "Using default JWT secret" Warning

This warning appears when no JWT secret is configured. To fix:

1. Set `LAYER9_JWT_SECRET` environment variable
2. Configure via `window.LAYER9_CONFIG.jwtSecret` in browser
3. Use `ConfigBuilder` to set a custom secret

### Authentication Failures

If authentication fails after migration:

1. Ensure JWT secret is properly configured
2. Check that `LAYER9_USE_MOCK_AUTH` is not accidentally enabled in production
3. Verify user credentials are properly set up with JwtAuthProvider

### Test Failures

If tests fail after migration:

1. Enable mock auth for test environments
2. Use `AuthService::with_mock_provider()` in tests
3. Set `LAYER9_USE_MOCK_AUTH=true` in test configuration