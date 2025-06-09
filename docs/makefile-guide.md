# DocPilot Makefile Guide

This guide provides comprehensive documentation for all Makefile targets available in DocPilot.

## Quick Reference

```bash
make help           # Show all available targets
make help-e2e       # Detailed E2E testing guide
make build          # Build optimized release version
make test-e2e       # Run comprehensive automated tests
make dev            # Complete development workflow
make ci             # Full CI pipeline
```

## Build Targets

### `make build`

Builds DocPilot in release mode with optimizations enabled.

```bash
make build
# Equivalent to: cargo build --release
```

**Output**: Optimized binary at `./target/release/docpilot`

### `make build-debug`

Builds DocPilot in debug mode for faster compilation during development.

```bash
make build-debug
# Equivalent to: cargo build
```

**Output**: Debug binary at `./target/debug/docpilot`

### `make install`

Builds and installs DocPilot to your system PATH.

```bash
make install
# Equivalent to: cargo build --release && cargo install --path .
```

**Requirements**: Automatically runs `make build` first

## Testing Targets

### `make test`

Runs all unit and integration tests.

```bash
make test
# Equivalent to: make test-unit && make test-integration
```

### `make test-unit`

Runs only unit tests (library tests).

```bash
make test-unit
# Equivalent to: cargo test --lib
```

### `make test-integration`

Runs only integration tests.

```bash
make test-integration
# Equivalent to: cargo test --test '*'
```

### `make test-e2e`

Runs comprehensive end-to-end usability tests that validate all DocPilot functionality automatically.

```bash
make test-e2e
# Equivalent to: ./scripts/run_e2e_tests.sh
```

**Features**:

- ✅ Tests all 7 major functionality areas
- ✅ No manual input required
- ✅ Validates complete user workflows
- ✅ Tests all 8 documentation templates
- ✅ Performance and stress testing
- ✅ Error handling and edge cases

**Test Suites**:

1. **Complete Basic Workflow** - Full user journey
2. **Configuration Management** - LLM provider setup
3. **Session State Management** - Session lifecycle
4. **Documentation Templates** - All output formats
5. **Error Handling** - Edge cases and validation
6. **Help Documentation** - CLI help system
7. **Performance Testing** - Stress testing with 20+ annotations

### `make run-e2e-tests`

Alternative E2E test runner using Rust test framework.

```bash
make run-e2e-tests
# Equivalent to: cargo test --test e2e_usability_test
```

## Development Targets

### `make check`

Runs code quality checks including linting.

```bash
make check
# Equivalent to: cargo check && cargo clippy -- -D warnings
```

### `make format`

Formats all Rust code using rustfmt.

```bash
make format
# Equivalent to: cargo fmt
```

### `make dev`

Complete development workflow - format, check, and test.

```bash
make dev
# Equivalent to: make format && make check && make test
```

**Recommended**: Use this before committing changes.

### `make clean`

Cleans build artifacts and test files.

```bash
make clean
# Removes: target/ directory and /tmp/docpilot-test-* files
```

## Documentation Targets

### `make docs`

Generates and opens Rust documentation.

```bash
make docs
# Equivalent to: cargo doc --open
```

### `make stats`

Shows project statistics including lines of code, test files, and dependencies.

```bash
make stats
```

**Example Output**:

```
📊 Project Statistics:
======================
Lines of code:
 16927 total

Test files:
7

Dependencies:
18
```

## Quality & Performance Targets

### `make audit`

Runs security audit on dependencies.

```bash
make audit
# Equivalent to: cargo audit
```

### `make update`

Updates all dependencies to latest versions.

```bash
make update
# Equivalent to: cargo update
```

### `make perf-test`

Runs performance-focused tests from the E2E suite.

```bash
make perf-test
# Filters E2E output for performance-related tests
```

## Example & Verification Targets

### `make run-example`

Runs a complete example workflow demonstration.

```bash
make run-example
```

**Demonstrates**:

1. Starting a documentation session
2. Adding various annotation types
3. Checking session status
4. Stopping the session
5. Generating documentation

### `make verify`

Quick verification that DocPilot is working correctly.

```bash
make verify
# Shows version and help information
```

## CI/CD Targets

### `make ci`

Complete CI pipeline for continuous integration.

```bash
make ci
# Equivalent to: make check && make test && make test-e2e
```

**Recommended**: Use this to validate changes before pushing.

## Help Targets

### `make help`

Shows all available Makefile targets with descriptions.

```bash
make help
```

### `make help-e2e`

Detailed guide specifically for end-to-end testing.

```bash
make help-e2e
```

## Common Workflows

### First-Time Setup

```bash
git clone https://github.com/yourusername/docpilot.git
cd docpilot
make build
make install
make verify
```

### Development Cycle

```bash
# Make your changes...
make dev            # Format, check, test
make test-e2e       # Comprehensive validation
```

### Before Committing

```bash
make ci             # Full CI pipeline
```

### Release Preparation

```bash
make clean
make ci
make stats          # Review project statistics
make audit          # Security check
```

### Performance Testing

```bash
make perf-test      # Performance validation
make test-e2e       # Full E2E including performance tests
```

## Troubleshooting

### Build Issues

```bash
make clean          # Clean artifacts
make build-debug    # Try debug build first
```

### Test Failures

```bash
make test-unit      # Run unit tests only
make test-integration # Run integration tests only
make test-e2e       # Run E2E tests for full validation
```

### Performance Issues

```bash
make perf-test      # Check performance tests
make stats          # Review project size
```

## Advanced Usage

### Custom Test Runs

```bash
# Run E2E tests with cleanup disabled
./scripts/run_e2e_tests.sh --no-cleanup

# Run specific test modules
cargo test filter::command
cargo test llm::integration
```

### Development with Logging

```bash
RUST_LOG=debug cargo run -- start "test session"
```

### Manual E2E Testing

```bash
# Build first
make build

# Run example workflow
make run-example

# Or run comprehensive E2E tests
make test-e2e
```

## Integration with IDEs

### VS Code

Add these tasks to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "DocPilot: Build",
      "type": "shell",
      "command": "make build",
      "group": "build"
    },
    {
      "label": "DocPilot: Test E2E",
      "type": "shell",
      "command": "make test-e2e",
      "group": "test"
    },
    {
      "label": "DocPilot: Dev Workflow",
      "type": "shell",
      "command": "make dev",
      "group": "build"
    }
  ]
}
```

## Best Practices

1. **Always run `make dev` before committing**
2. **Use `make test-e2e` for comprehensive validation**
3. **Run `make ci` before pushing to ensure CI will pass**
4. **Use `make clean` when switching between debug/release builds**
5. **Run `make audit` regularly to check for security issues**
6. **Use `make stats` to monitor project growth**

## Performance Notes

- `make build` creates optimized binaries (~2-3x faster than debug)
- `make test-e2e` runs 7 comprehensive test suites (~30-60 seconds)
- `make dev` is the fastest way to validate changes during development
- `make ci` provides the most thorough validation before releases

---

For more information, run `make help` or `make help-e2e` for detailed guidance.
