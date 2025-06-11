# Layer9 Testing Infrastructure

## Overview

Comprehensive testing suite for Layer9 framework with automated health checks, E2E tests, and CI/CD integration.

## Test Components

### 1. Health Check System (`layer9-health-check.js`)
- Automatic port conflict resolution
- Server startup validation
- WASM loading verification
- Performance benchmarking
- Memory stability testing

**Run:** `npm run health-check`

### 2. E2E Test Suite (`e2e/layer9-counter.test.js`)
- Full user interaction testing
- State management validation
- Performance metrics collection
- Console error monitoring

**Run:** `npm run test:e2e`

### 3. CI Test Suite (`ci-test-suite.js`)
- Unit tests
- Integration tests
- End-to-end tests
- Performance tests
- Security tests
- Accessibility tests

**Run:** `npm run test:ci`

### 4. Development Server (`../scripts/dev-server.js`)
- Smart port management
- Process cleanup
- Health monitoring
- Interactive port conflict resolution

**Run:** `npm run dev`

## Quick Start

```bash
# Run all tests
npm run test:all

# Start development server
npm run dev

# Run health check
npm run health-check

# Run CI suite
npm run test:ci
```

## CI/CD Integration

GitHub Actions workflow included in `.github/workflows/ci.yml`:
- Multi-OS testing (Ubuntu, macOS, Windows)
- Rust toolchain matrix (stable, beta)
- Automated benchmarking
- Security audits
- Preview deployments

## Port Management

The dev server automatically:
1. Detects port conflicts
2. Finds available ports
3. Optionally kills conflicting processes
4. Monitors server health

## Performance Thresholds

- First Paint: < 1000ms
- DOM Ready: < 2000ms
- WASM Load: < 3000ms
- Total Load: < 5000ms
- Memory Growth: < 10MB per 100 operations

## Test Reports

CI tests generate detailed JSON reports:
- Test results by suite
- Performance metrics
- Environment information
- Failed test details

Reports saved to: `test-report.json`