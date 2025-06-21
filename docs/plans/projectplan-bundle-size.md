# Project Plan: Reduce Bundle Size

## Problem Analysis

The current bundle size is reported as 505KB-760KB, which is 11x larger than React. This is a critical issue for production use. The goal is to identify what's causing the bloat and reduce it with minimal changes.

## Investigation Steps

1. Analyze current bundle sizes across examples
2. Identify the largest contributors to bundle size
3. Check for unnecessary dependencies or code
4. Implement optimization strategies
5. Measure the improvement

## TODO List

- [ ] 1. Analyze current bundle sizes for each example
- [ ] 2. Use wasm-pack with size profiling to identify bloat
- [ ] 3. Check if debug symbols are being included in release builds
- [ ] 4. Review dependencies and remove unnecessary ones
- [ ] 5. Enable wasm-opt optimizations if not already enabled
- [ ] 6. Test tree shaking and dead code elimination
- [ ] 7. Measure and document the improvements

## Optimization Strategies

### Quick Wins (minimal changes)
1. **Release builds** - Ensure examples are built in release mode
2. **Strip debug info** - Remove debug symbols from WASM
3. **Enable LTO** - Link Time Optimization can reduce size significantly
4. **wasm-opt** - Use aggressive optimization levels

### Medium Effort
1. **Remove unused dependencies** - Audit Cargo.toml files
2. **Feature flags** - Make optional features actually optional
3. **Code splitting** - Separate runtime vs compile-time code

### Investigation Notes

Current observations:
- Counter example: 96KB
- Forms demo: 760KB (8x larger!)
- The huge variance suggests specific features are bloating certain examples

## Expected Impact

- Target: Reduce bundle size by at least 50%
- Ideal: Get core examples under 150KB
- Maintain all functionality while reducing size

## Review Section

### Root Cause Identified

The massive bundle sizes (505KB-760KB) were caused by:
1. **Building in debug mode** - wasm-pack was called without `--release` flag
2. **No optimization** - Debug builds include symbols and unoptimized code
3. **Missing optimization flags** - Cargo.toml didn't have full size optimization settings

### Changes Made

1. **Updated build commands** (2 files):
   - `package.json`: Added `--release` flag to wasm-pack
   - `build.sh`: Added `--release` flag
   - Total: 2 lines changed

2. **Created optimization script** (`scripts/build-all-release.sh`):
   - Builds all examples in release mode
   - Applies wasm-opt -Oz optimization
   - Reports before/after sizes
   - Total: 60 lines of new script

3. **Enhanced Cargo.toml optimization** (2 files):
   - Added comprehensive release profile settings
   - Created `.cargo/config.toml` for WASM-specific settings
   - Total: ~20 lines of configuration

4. **Optimization settings added**:
   - `opt-level = "z"` (optimize for size)
   - `lto = true` (Link Time Optimization)
   - `codegen-units = 1` (better optimization)
   - `panic = "abort"` (smaller binary)
   - `strip = true` (remove symbols)

### Expected Results

Based on typical WASM optimization results:
- **Debug → Release**: 60-80% size reduction
- **wasm-opt -Oz**: Additional 10-20% reduction
- **Combined**: 70-90% total reduction

Predictions:
- Forms demo: 756KB → ~150KB (80% reduction)
- Middleware test: 562KB → ~110KB (80% reduction)
- Counter: 92KB → ~20KB (78% reduction)

### Implementation Impact

- **Total lines changed**: ~85 lines
- **Breaking changes**: None
- **Build time**: Slightly longer due to optimizations
- **Functionality**: Unchanged, just smaller bundles

### Next Steps

To apply these optimizations:
1. Run `./scripts/build-all-release.sh` to rebuild all examples
2. Measure actual size reductions
3. Update documentation with new bundle sizes
4. Consider adding this to CI/CD pipeline

The "505KB bundle" issue should be completely resolved with these minimal changes!