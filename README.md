# DocPilot 🚁

**Intelligent Terminal Documentation Tool**

DocPilot automatically captures your terminal commands and generates comprehensive, AI-enhanced documentation of your workflows. Perfect for creating tutorials, documenting processes, and sharing knowledge with your team.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-258%20passing-green.svg)](#testing)

## ✨ Features

### 🎯 **Smart Command Capture**

- **Cross-platform terminal monitoring** (Linux, macOS)
- **Multi-shell support** (Bash, Zsh, Fish)
- **Background monitoring by default** - continue using your terminal normally
- **Real-time command tracking** with timestamps and context
- **Automatic success/failure detection**

### 🤖 **AI-Powered Analysis**

- **Multiple LLM providers** (Claude, ChatGPT, Gemini, Ollama)
- **Intelligent command explanation** and context analysis
- **Issue identification** and alternative suggestions
- **Security recommendations** and best practices

### 🔒 **Privacy & Security**

- **Smart privacy filtering** with configurable sensitivity levels
- **Automatic redaction** of passwords, API keys, and sensitive data
- **Secure API key storage** with encryption
- **Customizable sensitive pattern detection**

### 📝 **Professional Documentation**

- **Beautiful Markdown output** with syntax highlighting
- **Hierarchical organization** by workflow phases
- **Customizable templates** and formatting options
- **Code block generation** with intelligent language detection

### 🛡️ **Advanced Filtering**

- **Command validation** and sequence dependency checking
- **Typo detection** and suspicious command filtering
- **Workflow optimization** suggestions
- **Command deduplication** and cleanup

## 🚀 Quick Start

### For Developers (Using Makefile)

If you're contributing to DocPilot or want to build from source:

```bash
# Clone and build
git clone https://github.com/yourusername/docpilot.git
cd docpilot
make build          # Build optimized release version
make install        # Install to system

# Run comprehensive tests
make test-e2e       # Automated end-to-end testing (recommended)
make ci             # Full CI pipeline

# Quick development cycle
make dev            # Format + check + test
```

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository

### Installation

#### Option 1: Install from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/docpilot.git
cd docpilot

# Build and install
cargo build --release
cargo install --path .
```

#### Option 2: Install from Crates.io (Coming Soon)

```bash
cargo install docpilot
```

### Initial Setup

1. **Configure your LLM provider** (optional but recommended):

```bash
# Set up Claude (recommended)
docpilot config --provider claude --api-key your-claude-api-key

# Or use OpenAI
docpilot config --provider openai --api-key your-openai-api-key

# Or use local Ollama (no API key required)
docpilot config --provider ollama --base-url http://localhost:11434

# You can also set them separately
docpilot config --provider claude
docpilot config --api-key your-claude-api-key
docpilot config --base-url http://localhost:11434  # For current provider
```

2. **Test the installation**:

```bash
docpilot --version
docpilot --help
```

## 📖 Usage

### Basic Workflow

1. **Start a documentation session** (runs in background by default):

```bash
docpilot start "Setting up a new React project"
```

2. **Run your commands normally** - DocPilot monitors in the background:

```bash
npx create-react-app my-app
cd my-app
npm install axios
npm start
```

3. **Add manual annotations** for non-terminal activities:

```bash
docpilot annotate "Opened browser and navigated to localhost:3000"
docpilot annotate "Verified the React app is running correctly"
```

4. **Stop the session and generate documentation**:

```bash
docpilot stop
docpilot generate --output my-react-setup.md
```

### Advanced Usage

#### Session Management

```bash
# List all sessions
docpilot list

# Resume a paused session
docpilot resume session-id

# Export session data
docpilot export session-id --format json

# Import session data
docpilot import backup.json
```

#### Customization

```bash
# View current configuration
docpilot config

# Set LLM provider and API key
docpilot config --provider claude --api-key your-api-key

# Note: Advanced configuration options like privacy filtering,
# validation, and deduplication will be available in future versions
```

#### Filtering and Validation

```bash
# Generate with specific filtering
docpilot generate --exclude-failed --only-successful

# Include workflow optimizations
docpilot generate --include-optimizations

# Note: Advanced filtering options will be available in future versions
```

## 🔧 Configuration

DocPilot stores configuration in `~/.docpilot/config.json`. You can edit this file directly or use the CLI:

```bash
# View current configuration
docpilot config

# Set LLM provider and API key
docpilot config --provider claude --api-key your-api-key

# Set provider only
docpilot config --provider claude

# Set API key for current provider
docpilot config --api-key your-api-key

# Set base URL for current provider (useful for Ollama)
docpilot config --base-url http://localhost:11434

# Set Ollama with custom URL (no API key needed)
docpilot config --provider ollama --base-url http://localhost:11434
```

### Configuration Options

| Option         | Description                                   | Default |
| -------------- | --------------------------------------------- | ------- |
| `llm_provider` | AI provider (claude, chatgpt, gemini, ollama) | None    |
| `api_keys`     | API keys for configured providers             | None    |
| `base_urls`    | Base URLs for providers (e.g., Ollama)        | None    |

## 📚 Examples

### Example 1: Docker Setup Documentation

```bash
# Start session (runs in background by default)
docpilot start "Docker containerization setup"

# Your commands - DocPilot captures them automatically
docker build -t myapp .
docker run -p 3000:3000 myapp
docker ps
docker logs container-id

docpilot annotate "Verified application is running in container"
docpilot stop
docpilot generate --output docker-setup.md --template comprehensive
```

### Example 2: Server Deployment

```bash
# Start session in background (default behavior)
docpilot start "Production server deployment"

# Your deployment commands - monitored automatically
ssh user@server
git pull origin main
npm install --production
pm2 restart app
nginx -t && systemctl reload nginx

docpilot stop
docpilot generate --output deployment-guide.md --include-optimizations
```

### Example 3: Debugging Session (Foreground Mode)

```bash
# Run in foreground for debugging purposes
docpilot start "Debugging network issues" --foreground

# Commands are captured with immediate feedback
ping google.com
traceroute google.com
netstat -an | grep :80

# Press Ctrl+C to stop foreground session
```

## 🏗️ Architecture

DocPilot is built with a modular architecture:

```
src/
├── main.rs              # CLI entry point
├── terminal/            # Terminal monitoring
│   ├── monitor.rs       # Command capture
│   └── platform.rs      # Cross-platform support
├── llm/                 # AI integration
│   ├── client.rs        # LLM client abstraction
│   ├── analyzer.rs      # AI analysis engine
│   └── prompt.rs        # Prompt engineering
├── session/             # Session management
│   └── manager.rs       # Session lifecycle
├── filter/              # Command filtering
│   └── command.rs       # Filtering and validation
└── output/              # Documentation generation
    ├── markdown.rs      # Markdown templates
    └── codeblock.rs     # Code formatting
```

## 🧪 Testing

DocPilot has comprehensive test coverage with 258+ tests across multiple categories:

### Quick Testing with Makefile

```bash
# Run all tests (recommended)
make test           # Unit + integration tests

# Run comprehensive end-to-end tests
make test-e2e       # 7 automated test suites, no manual input required

# Run specific test categories
make test-unit      # Unit tests only
make test-integration # Integration tests only

# Performance and stress testing
make perf-test      # Performance validation with stress testing

# Complete CI pipeline
make ci             # check + test + test-e2e (full validation)
```

### Advanced Testing with Cargo

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test filter::command
cargo test llm::integration
cargo test session::manager

# Run with coverage
cargo test --all-features

# Run E2E tests directly
cargo test --test e2e_usability_test
```

### End-to-End Test Suites

The E2E tests (`make test-e2e`) automatically validate all functionality:

1. **Complete Basic Workflow** - Full user journey from start to finish
2. **Configuration Management** - LLM provider setup and configuration
3. **Session State Management** - Session lifecycle and state transitions
4. **Documentation Templates** - All 8 available output templates
5. **Error Handling** - Edge cases and error conditions
6. **Help Documentation** - All help commands and documentation
7. **Performance Testing** - Stress testing with 20+ rapid annotations

### Test Categories

- **Unit Tests** (180+): Individual component testing
- **Integration Tests** (50+): Cross-component functionality
- **End-to-End Tests** (7 suites): Complete workflow validation
- **Privacy Tests** (15+): Sensitive data filtering
- **Validation Tests** (13+): Command sequence validation

### Testing Benefits

✅ **No Manual Input Required** - All tests run automatically
✅ **Complete Coverage** - Tests all user-facing functionality
✅ **Cross-Platform** - Validates Linux and macOS compatibility
✅ **Performance Validated** - Stress testing with rapid operations
✅ **Regression Prevention** - Catches breaking changes early

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone and setup
git clone https://github.com/yourusername/docpilot.git
cd docpilot

# Install development dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- start "test session"
```

### Using the Makefile

DocPilot includes a comprehensive Makefile that simplifies common development tasks. For detailed documentation, see the [Makefile Guide](docs/makefile-guide.md).

Here are the most commonly used targets:

#### 🔨 **Build Commands**

```bash
make build          # Build in release mode (optimized)
make build-debug    # Build in debug mode (faster compilation)
make install        # Build and install DocPilot to your system
make verify         # Quick verification that DocPilot is working
```

#### 🧪 **Testing Commands**

```bash
make test           # Run all tests (unit + integration)
make test-unit      # Run unit tests only
make test-integration # Run integration tests only
make test-e2e       # Run comprehensive end-to-end tests
make run-e2e-tests  # Alternative E2E test runner using Rust
```

#### 🚀 **End-to-End Testing**

The E2E tests are particularly powerful - they automatically test all DocPilot functionality without requiring manual input:

```bash
make test-e2e       # Runs 7 comprehensive test suites:
                    # • Complete Basic Workflow
                    # • Configuration Management
                    # • Session State Management
                    # • Documentation Templates (all 8 templates)
                    # • Error Handling & Edge Cases
                    # • Help Documentation
                    # • Performance Testing
```

#### 🛠️ **Development Commands**

```bash
make check          # Run cargo check and clippy linting
make format         # Format code with rustfmt
make dev            # Complete development workflow (format + check + test)
make clean          # Clean build artifacts and test files
```

#### 📚 **Documentation & Analysis**

```bash
make docs           # Generate and open Rust documentation
make stats          # Show project statistics (lines of code, tests, dependencies)
make help           # Show all available Makefile targets
make help-e2e       # Detailed guide for E2E testing
```

#### ⚡ **Performance & Quality**

```bash
make perf-test      # Run performance-focused tests
make audit          # Security audit of dependencies
make update         # Update all dependencies
make ci             # Complete CI pipeline (check + test + e2e)
```

#### 🎯 **Quick Examples**

```bash
make run-example    # Run a complete example workflow demonstration
                    # Shows: start → annotate → status → stop → generate
```

#### **Common Development Workflows**

**Quick Development Cycle:**

```bash
make dev            # Format, check, and test everything
```

**Before Committing:**

```bash
make ci             # Run the full CI pipeline
```

**Testing New Features:**

```bash
make test-e2e       # Comprehensive end-to-end validation
```

**Performance Validation:**

```bash
make perf-test      # Stress test with multiple annotations
```

### Code Style

- Follow Rust standard formatting: `cargo fmt` or `make format`
- Ensure clippy compliance: `cargo clippy` or `make check`
- Add tests for new features
- Update documentation for API changes
- Run `make ci` before submitting pull requests

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Rust Community** for excellent tooling and libraries
- **LLM Providers** for AI capabilities
- **Contributors** who help improve DocPilot

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/docpilot/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/docpilot/discussions)
- **Documentation**: [Wiki](https://github.com/yourusername/docpilot/wiki)

## 🗺️ Roadmap

- [ ] **Web Interface** - Browser-based session management
- [ ] **Team Collaboration** - Shared sessions and templates
- [ ] **Plugin System** - Custom analyzers and formatters
- [ ] **Cloud Sync** - Cross-device session synchronization
- [ ] **IDE Integration** - VS Code and other editor plugins

---

**Made with ❤️ by the DocPilot team**

_Transform your terminal commands into beautiful documentation effortlessly._
