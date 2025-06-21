#!/bin/bash

# Test coverage measurement script for Layer9
set -e

echo "🧪 Layer9 Test Coverage Report"
echo "=============================="

# Clean previous coverage data
rm -rf coverage
mkdir -p coverage

# Install required tools
echo "📦 Installing coverage tools..."
cargo install cargo-tarpaulin --quiet 2>/dev/null || true

# Run tests with coverage for core crate
echo "🔍 Running tests with coverage measurement..."
cd crates/core

# Run Rust tests with tarpaulin
echo "📊 Measuring Rust test coverage..."
cargo tarpaulin \
    --out Html \
    --out Json \
    --output-dir ../../coverage \
    --exclude-files "*/tests/*" \
    --exclude-files "*/examples/*" \
    --exclude-files "*/target/*" \
    --ignore-panics \
    --timeout 300 \
    2>&1 | tee ../../coverage/tarpaulin.log

# Extract coverage percentage
COVERAGE=$(grep -oE '[0-9]+\.[0-9]+%' ../../coverage/tarpaulin.log | tail -1 | sed 's/%//')

echo ""
echo "📈 Coverage Summary"
echo "=================="
echo "Total Coverage: ${COVERAGE}%"

# Check if we meet the target
TARGET=80
if (( $(echo "$COVERAGE >= $TARGET" | bc -l) )); then
    echo "✅ Target coverage of ${TARGET}% achieved!"
else
    echo "❌ Below target coverage of ${TARGET}%"
    MISSING=$(echo "$TARGET - $COVERAGE" | bc -l)
    echo "   Need ${MISSING}% more coverage"
fi

# Generate detailed report
echo ""
echo "📄 Detailed Report"
echo "================="

# List files with low coverage
echo "Files with coverage < 80%:"
if [ -f "../../coverage/tarpaulin-report.json" ]; then
    # Parse JSON report for detailed file coverage
    echo "(Detailed file coverage available in coverage/index.html)"
fi

echo ""
echo "🌐 Open coverage/index.html to view detailed HTML report"

# Count total tests
echo ""
echo "📊 Test Statistics"
echo "================="
UNIT_TESTS=$(cargo test --lib 2>&1 | grep -oE 'test result.*[0-9]+ passed' | grep -oE '[0-9]+ passed' | awk '{print $1}' || echo "0")
echo "Unit Tests: ${UNIT_TESTS}"

cd ../..

# E2E test count
E2E_TESTS=$(find tests/e2e -name "*.test.js" -type f | wc -l)
echo "E2E Test Files: ${E2E_TESTS}"

# Calculate uncovered lines
echo ""
echo "🔍 Coverage Analysis"
echo "==================="
echo "Key areas needing more tests:"
echo "- Router (router_v2.rs) - routing logic"
echo "- WebSocket (websocket.rs) - real-time features"
echo "- Forms (form.rs) - validation and submission"
echo "- Middleware (middleware_v2.rs) - request pipeline"
echo "- Database (db.rs, db_api.rs) - data layer"

# Create coverage badge
echo ""
echo "🏷️  Coverage Badge"
echo "================"
if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
    COLOR="brightgreen"
elif (( $(echo "$COVERAGE >= 60" | bc -l) )); then
    COLOR="yellow"
else
    COLOR="red"
fi

echo "[![Coverage](https://img.shields.io/badge/coverage-${COVERAGE}%25-${COLOR})](coverage/index.html)"

echo ""
echo "✨ Coverage report complete!"