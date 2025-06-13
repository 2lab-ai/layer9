# Layer9 Ultrathink Makefile
# AI-Driven Feature Development Pipeline

.PHONY: all help update-feature-ultrathink ultrathink update-feature-ultrathink-old dev test test-quick build clean server validate ultra todo-list todo-status watch fmt lint bench new-component commit-ai install ci deploy perf audit docs version

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[1;37m
NC := \033[0m # No Color

# Default target
all: lint build test
	@echo ""
	@echo "$(GREEN)═══════════════════════════════════════════════════════════════$(NC)"
	@echo "$(GREEN)✅ ALL CHECKS PASSED! Layer9 is production ready!$(NC)"
	@echo "$(GREEN)═══════════════════════════════════════════════════════════════$(NC)"

# Help command
help:
	@echo "$(CYAN)╔══════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║               Layer9 Ultrathink Makefile                     ║$(NC)"
	@echo "$(CYAN)╚══════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(YELLOW)AI-Driven Development:$(NC)"
	@echo "  $(GREEN)make update-feature-ultrathink$(NC) - AI selects TODO, implements, tests until success"
	@echo "  $(GREEN)make ultrathink$(NC)               - Alias for update-feature-ultrathink"
	@echo "  $(GREEN)make todo-list$(NC)                - Show all TODOs from README"
	@echo "  $(GREEN)make todo-status$(NC)              - Show implementation status"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  $(GREEN)make dev$(NC)        - Start development server (Rust)"
	@echo "  $(GREEN)make build$(NC)      - Build WASM examples"
	@echo "  $(GREEN)make test$(NC)       - Run all tests"
	@echo "  $(GREEN)make validate$(NC)   - Run validation tests"
	@echo "  $(GREEN)make ultra$(NC)      - Run ultra validator"
	@echo ""
	@echo "$(YELLOW)Server:$(NC)"
	@echo "  $(GREEN)make server$(NC)     - Run Rust server directly"
	@echo "  $(GREEN)make clean$(NC)      - Clean build artifacts"

# Main AI-driven feature implementation
update-feature-ultrathink:
	@echo "$(PURPLE)╔══════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(PURPLE)║          🧠 ULTRATHINK FEATURE IMPLEMENTATION 🧠             ║$(NC)"
	@echo "$(PURPLE)╚══════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(CYAN)Phase 1: Analyzing README for TODOs...$(NC)"
	@node scripts/ultrathink/parse-todos.js
	@echo ""
	@echo "$(CYAN)Phase 2: Selecting highest priority TODO...$(NC)"
	@TODO=$$(node scripts/ultrathink/select-todo.js) && \
	echo "$(YELLOW)Selected: $$TODO$(NC)" && \
	echo "" && \
	echo "$(CYAN)Phase 3: Implementing feature automatically...$(NC)" && \
	node scripts/ultrathink/implement-feature-auto.js "$$TODO" && \
	echo "" && \
	echo "$(CYAN)Phase 4: Testing until success...$(NC)" && \
	node scripts/ultrathink/test-until-success.js && \
	echo "" && \
	echo "$(CYAN)Phase 5: Updating README...$(NC)" && \
	node scripts/ultrathink/update-readme.js "$$TODO" && \
	echo "" && \
	echo "$(GREEN)✅ Feature implementation complete!$(NC)"

# Alias for convenience
ultrathink: update-feature-ultrathink

# Original implementation without Claude (for comparison/fallback)
update-feature-ultrathink-old:
	@echo "$(PURPLE)╔══════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(PURPLE)║       🧠 ULTRATHINK FEATURE PLANNING (OLD) 🧠                ║$(NC)"
	@echo "$(PURPLE)╚══════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(CYAN)Phase 1: Analyzing README for TODOs...$(NC)"
	@node scripts/ultrathink/parse-todos.js
	@echo ""
	@echo "$(CYAN)Phase 2: Selecting highest priority TODO...$(NC)"
	@TODO=$$(node scripts/ultrathink/select-todo.js) && \
	echo "$(YELLOW)Selected: $$TODO$(NC)" && \
	echo "" && \
	echo "$(CYAN)Phase 3: Planning feature implementation...$(NC)" && \
	node scripts/ultrathink/implement-feature.js "$$TODO" && \
	echo "" && \
	echo "$(CYAN)Phase 4: Testing until success...$(NC)" && \
	node scripts/ultrathink/test-until-success.js && \
	echo "" && \
	echo "$(CYAN)Phase 5: Updating README...$(NC)" && \
	node scripts/ultrathink/update-readme.js "$$TODO" && \
	echo "" && \
	echo "$(GREEN)✅ Feature planning complete!$(NC)"

# Show all TODOs
todo-list:
	@echo "$(CYAN)📋 Layer9 TODO List$(NC)"
	@echo "$(CYAN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@node scripts/ultrathink/parse-todos.js --list

# Show implementation status
todo-status:
	@echo "$(CYAN)📊 Layer9 Implementation Status$(NC)"
	@echo "$(CYAN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@node scripts/ultrathink/parse-todos.js --status

# Development server
dev:
	@echo "$(CYAN)🚀 Starting Layer9 Development Server$(NC)"
	npm run dev

# Build WASM
build:
	@echo "$(CYAN)🔨 Building WASM examples$(NC)"
	npm run build:example

# Run tests
test:
	@echo "$(CYAN)🧪 Running Comprehensive Test Suite$(NC)"
	@node test/quick-comprehensive-test.js

# Quick test (just final validation)
test-quick:
	@echo "$(CYAN)⚡ Running Quick Test$(NC)"
	@npm run test:quick

# Full test with compilation (slower)
test-full:
	@echo "$(CYAN)🧪 Running Full Test Suite (with compilation)$(NC)"
	@node test/comprehensive-test-runner.js

# Run validation
validate:
	@echo "$(CYAN)🔍 Running validation$(NC)"
	npm run validate

# Run ultra validator
ultra:
	@echo "$(PURPLE)⚡ Running Ultra Validator$(NC)"
	npm run ultra

# Run Rust server directly
server:
	@echo "$(CYAN)🦀 Starting Rust server$(NC)"
	cargo run --manifest-path crates/layer9-server/Cargo.toml -- --dir examples/counter --port 8080

# Clean build artifacts
clean:
	@echo "$(YELLOW)🧹 Cleaning build artifacts$(NC)"
	rm -rf target/
	rm -rf examples/counter/pkg/
	rm -rf node_modules/.cache/
	@echo "$(GREEN)✅ Clean complete$(NC)"

# Watch for changes and rebuild
watch:
	@echo "$(CYAN)👁️  Watching for changes...$(NC)"
	cargo watch -x "build --target wasm32-unknown-unknown"

# Format code
fmt:
	@echo "$(CYAN)🎨 Formatting code$(NC)"
	cargo fmt --all
	@echo "$(GREEN)✅ Code formatted$(NC)"

# Lint code
lint:
	@echo "$(CYAN)🔍 Linting code$(NC)"
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✅ Linting complete$(NC)"

# Run benchmarks
bench:
	@echo "$(CYAN)📊 Running benchmarks$(NC)"
	cargo bench

# Create a new component
new-component:
	@read -p "Component name: " name; \
	node scripts/ultrathink/create-component.js $$name

# Git commit with AI-generated message
commit-ai:
	@echo "$(CYAN)🤖 Generating commit message...$(NC)"
	@git add -A && \
	MSG=$$(node scripts/ultrathink/generate-commit.js) && \
	git commit -m "$$MSG" && \
	echo "$(GREEN)✅ Committed with AI-generated message$(NC)"

# Install all dependencies
install:
	@echo "$(CYAN)📦 Installing dependencies$(NC)"
	npm install
	cargo fetch
	@echo "$(GREEN)✅ Dependencies installed$(NC)"

# Full CI pipeline
ci: clean install lint test validate
	@echo "$(GREEN)✅ CI pipeline complete$(NC)"

# Deploy to production (placeholder)
deploy:
	@echo "$(RED)❌ Production deployment not yet implemented$(NC)"
	@echo "$(YELLOW)See ROADMAP_TO_PRODUCTION.md for details$(NC)"

# Performance analysis
perf:
	@echo "$(CYAN)⚡ Analyzing performance$(NC)"
	@node scripts/ultrathink/performance-analysis.js

# Security audit
audit:
	@echo "$(CYAN)🔒 Running security audit$(NC)"
	cargo audit
	npm audit

# Documentation generation
docs:
	@echo "$(CYAN)📚 Generating documentation$(NC)"
	cargo doc --no-deps --open

# Version bump
version:
	@echo "$(CYAN)📌 Current version:$(NC)"
	@grep version Cargo.toml | head -1
	@read -p "New version: " version; \
	sed -i '' "s/version = \".*\"/version = \"$$version\"/" Cargo.toml && \
	echo "$(GREEN)✅ Version updated to $$version$(NC)"