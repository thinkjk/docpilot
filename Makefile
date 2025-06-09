# DocPilot Makefile
# Provides convenient commands for building, testing, and running DocPilot

.PHONY: help build test test-unit test-integration test-e2e clean install run-e2e-tests

# Default target
help:
	@echo "DocPilot - Intelligent Terminal Documentation Tool"
	@echo "=================================================="
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build the project in release mode"
	@echo "  build-debug     - Build the project in debug mode"
	@echo "  test            - Run all tests (unit + integration)"
	@echo "  test-unit       - Run unit tests only"
	@echo "  test-integration- Run integration tests only"
	@echo "  test-e2e        - Run end-to-end usability tests"
	@echo "  clean           - Clean build artifacts"
	@echo "  install         - Install DocPilot to system"
	@echo "  run-example     - Run a quick example workflow"
	@echo "  check           - Run cargo check and clippy"
	@echo "  format          - Format code with rustfmt"
	@echo ""
	@echo "E2E Testing:"
	@echo "  The end-to-end tests automatically test all DocPilot functionality"
	@echo "  without requiring manual typing. They cover:"
	@echo "  • Complete user workflows"
	@echo "  • Configuration management"
	@echo "  • Session state management"
	@echo "  • Documentation generation"
	@echo "  • Error handling"
	@echo "  • Performance testing"

# Build targets
build:
	@echo "🔨 Building DocPilot (release mode)..."
	cargo build --release

build-debug:
	@echo "🔨 Building DocPilot (debug mode)..."
	cargo build

# Test targets
test: test-unit test-integration
	@echo "✅ All tests completed!"

test-unit:
	@echo "🧪 Running unit tests..."
	cargo test --lib

test-integration:
	@echo "🧪 Running integration tests..."
	cargo test --test '*'

test-e2e: build
	@echo "🚀 Running end-to-end usability tests..."
	@echo "This will test all DocPilot functionality automatically!"
	./scripts/run_e2e_tests.sh

# Alternative target for running E2E tests with Rust
run-e2e-tests: build
	@echo "🚀 Running comprehensive E2E tests..."
	cargo test --test e2e_usability_test

# Development targets
check:
	@echo "🔍 Running cargo check..."
	cargo check
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings

format:
	@echo "🎨 Formatting code..."
	cargo fmt

# Installation
install: build
	@echo "📦 Installing DocPilot..."
	cargo install --path .

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "🧹 Removing test artifacts..."
	rm -rf /tmp/docpilot-test-* 2>/dev/null || true

# Example workflow
run-example: build
	@echo "🎯 Running example DocPilot workflow..."
	@echo "This demonstrates basic usage without E2E testing"
	@echo ""
	@echo "1. Starting a test session..."
	./target/release/docpilot start "Example workflow demonstration" --output example-output.md || true
	@sleep 1
	@echo ""
	@echo "2. Adding some annotations..."
	./target/release/docpilot note "This is an example note" || true
	./target/release/docpilot warn "This is an example warning" || true
	./target/release/docpilot milestone "Example milestone reached" || true
	@echo ""
	@echo "3. Checking session status..."
	./target/release/docpilot status || true
	@echo ""
	@echo "4. Stopping the session..."
	./target/release/docpilot stop || true
	@echo ""
	@echo "5. Generating documentation..."
	./target/release/docpilot generate --output example-final.md --template standard || true
	@echo ""
	@echo "✅ Example completed! Check example-final.md for output."

# Quick test to verify installation
verify: build
	@echo "🔍 Verifying DocPilot installation..."
	./target/release/docpilot --version
	./target/release/docpilot --help | head -10
	@echo "✅ DocPilot is working correctly!"

# Development workflow
dev: format check test
	@echo "🎉 Development workflow completed!"

# CI/CD targets
ci: check test test-e2e
	@echo "🎉 CI pipeline completed successfully!"

# Documentation
docs:
	@echo "📚 Generating documentation..."
	cargo doc --open

# Performance testing
perf-test: build
	@echo "⚡ Running performance tests..."
	./scripts/run_e2e_tests.sh | grep -E "(Performance|annotations rapidly|rapid status)"

# Security audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Update dependencies
update:
	@echo "📦 Updating dependencies..."
	cargo update

# Show project statistics
stats:
	@echo "📊 Project Statistics:"
	@echo "======================"
	@echo "Lines of code:"
	@find src -name "*.rs" -exec wc -l {} + | tail -1
	@echo ""
	@echo "Test files:"
	@find . -name "*test*.rs" -o -name "tests" -type d | wc -l
	@echo ""
	@echo "Dependencies:"
	@grep -c "^[a-zA-Z]" Cargo.toml || echo "0"

# Help for E2E testing specifically
help-e2e:
	@echo "End-to-End Usability Testing Guide"
	@echo "=================================="
	@echo ""
	@echo "The E2E tests automatically verify all DocPilot functionality:"
	@echo ""
	@echo "🧪 Test Suites:"
	@echo "  1. Complete Basic Workflow    - Full user journey from start to finish"
	@echo "  2. Configuration Management   - LLM provider setup and configuration"
	@echo "  3. Session State Management   - Session lifecycle and state transitions"
	@echo "  4. Documentation Templates    - All available output templates"
	@echo "  5. Error Handling             - Edge cases and error conditions"
	@echo "  6. Concurrent Operations      - Multi-user and concurrent scenarios"
	@echo "  7. Performance Testing        - Stress testing and performance validation"
	@echo "  8. Filesystem Integration     - File operations and permissions"
	@echo "  9. Help Documentation         - All help commands and documentation"
	@echo "  10. Shell Integration         - Integration with shell commands"
	@echo ""
	@echo "🚀 Running E2E Tests:"
	@echo "  make test-e2e                 - Run all E2E tests"
	@echo "  ./scripts/run_e2e_tests.sh    - Run tests directly"
	@echo "  ./scripts/run_e2e_tests.sh --no-cleanup  - Keep test artifacts"
	@echo ""
	@echo "✅ Benefits:"
	@echo "  • No manual typing required"
	@echo "  • Tests all functionality automatically"
	@echo "  • Validates complete user workflows"
	@echo "  • Catches regressions early"
	@echo "  • Provides confidence in releases"