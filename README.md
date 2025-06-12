# DocPilot 🚁

**Intelligent Terminal Documentation Tool**

DocPilot automatically captures your terminal commands and generates comprehensive, AI-enhanced documentation of your workflows. Perfect for creating tutorials, documenting processes, and sharing knowledge with your team.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-258%20passing-green.svg)](#testing)

## ✨ Features

### 🎯 **Smart Command Capture**

- **Reliable shell integration** - DocPilot automatically configures shell hooks with proper session management
- **Multi-shell support** (Bash, Zsh, Fish) with dynamic session detection
- **Session-isolated tracking** - only commands from active DocPilot sessions, no shell history pollution
- **Background monitoring by default** - continue using your terminal normally
- **Minimal setup** - one command to activate current session, future sessions auto-configured
- **Automatic success/failure detection** with exit code capture
- **Recent fixes** - Command capture now works reliably with proper session ID matching

### 🤖 **AI-Powered Analysis** (Default when configured)

- **AI-enhanced documentation by default** - Standard template automatically uses AI when LLM is configured
- **Real-time progress indicators** - Clear feedback during AI processing to show generation status
- **Multiple LLM providers** (Claude, ChatGPT, Gemini, Ollama)
- **Intelligent command explanation** with purpose, prerequisites, and troubleshooting
- **Workflow pattern analysis** and command relationship detection
- **Issue identification** and alternative suggestions with confidence scores
- **Security recommendations** and best practices generation
- **Smart command filtering** - Removes problematic commands before AI analysis

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
# DocPilot shows you exactly what to run next
```

2. **Activate command capture for current session** (one simple command):

```bash
# DocPilot provides the exact command to copy and paste:
source ~/.docpilot/zsh_hooks.zsh
# ✅ Current session: Commands now captured automatically
# ✅ Future sessions: Already configured to auto-capture
```

3. **Run your commands normally** - DocPilot captures them automatically:

```bash
npx create-react-app my-app
cd my-app
npm install axios
npm start
```

4. **Add manual annotations** for non-terminal activities:

```bash
docpilot annotate "Opened browser and navigated to localhost:3000"
docpilot annotate "Verified the React app is running correctly"
```

5. **Stop the session and generate documentation** (AI-enhanced by default):

```bash
docpilot stop
docpilot generate --output my-react-setup.md
# 🤖 Using AI-enhanced documentation for standard template (LLM configured)
# 🚀 Generating comprehensive AI-enhanced documentation...
# 🔍 Analyzing command: npx create-react-app my-app
# ✅ Analysis complete (confidence: 95.2%)
# 🎉 Comprehensive AI documentation complete!
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

#### Documentation Templates

DocPilot offers 8 different templates with AI enhancement automatically enabled when LLM is configured:

```bash
# Standard template (AI-enhanced by default when LLM configured)
docpilot generate --template standard --output guide.md

# Explicit AI-enhanced template (requires LLM setup)
docpilot generate --template ai-enhanced --output guide.md

# Other templates (also AI-enhanced when LLM available)
docpilot generate --template comprehensive   # Detailed with full metadata
docpilot generate --template minimal        # Compact format
docpilot generate --template hierarchical   # Organized by workflow phases
docpilot generate --template professional   # Business-ready format
docpilot generate --template technical      # Technical analysis focus
docpilot generate --template rich          # Enhanced with emojis
docpilot generate --template github        # GitHub-compatible format
```

#### Customization

```bash
# View current configuration
docpilot config

# Set LLM provider and API key for AI enhancement
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

# Activate command capture (DocPilot shows the exact command)
source ~/.docpilot/zsh_hooks.zsh

# Your commands - DocPilot captures them automatically
docker build -t myapp .
docker run -p 3000:3000 myapp
docker ps
docker logs container-id

docpilot annotate "Verified application is running in container"
docpilot stop
docpilot generate --output docker-setup.md --template comprehensive
# 🤖 Using AI-enhanced documentation for comprehensive template (LLM configured)
# 🔬 Generating comprehensive AI analysis...
# 📊 Analyzing workflow patterns and command relationships...
# 🎉 Comprehensive AI documentation complete!
```

### Example 2: Server Deployment

```bash
# Start session in background (default behavior)
docpilot start "Production server deployment"

# Activate command capture (DocPilot shows the exact command)
source ~/.docpilot/zsh_hooks.zsh

# Your deployment commands - monitored automatically
ssh user@server
git pull origin main
npm install --production
pm2 restart app
nginx -t && systemctl reload nginx

docpilot stop
docpilot generate --output deployment-guide.md
# 🤖 AI analysis enabled - generating enhanced documentation...
# 🔍 Analyzing command: ssh user@server
# 🔍 Analyzing command: git pull origin main
# ✅ Analysis complete (confidence: 88.5%)
# 🤖 Applying AI post-processing to improve documentation quality...
# ✅ Documentation generated successfully!
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

DocPilot is built with a modular architecture designed for extensibility and maintainability:

```
docpilot/
├── src/                           # Core application source
│   ├── main.rs                    # CLI entry point and command routing
│   ├── terminal/                  # Terminal monitoring and command capture
│   │   ├── mod.rs                 # Module exports and platform detection
│   │   ├── monitor.rs             # Real-time command monitoring
│   │   ├── monitor.test.rs        # Terminal monitoring tests
│   │   └── platform.rs            # Cross-platform shell integration
│   ├── llm/                       # AI integration and analysis
│   │   ├── mod.rs                 # LLM module exports
│   │   ├── client.rs              # Multi-provider LLM client
│   │   ├── config.rs              # LLM configuration management
│   │   ├── analyzer.rs            # AI-powered command analysis
│   │   ├── prompt.rs              # Prompt engineering and templates
│   │   ├── error_handler.rs       # LLM error handling and retry logic
│   │   └── integration_tests.rs   # LLM integration testing
│   ├── session/                   # Session lifecycle management
│   │   ├── mod.rs                 # Session module exports
│   │   ├── manager.rs             # Session state and persistence
│   │   └── manager.test.rs        # Session management tests
│   ├── filter/                    # Command filtering and validation
│   │   ├── mod.rs                 # Filter module exports
│   │   ├── command.rs             # Privacy filtering and validation
│   │   └── command.test.rs        # Command filtering tests
│   └── output/                    # Documentation generation
│       ├── mod.rs                 # Output module exports
│       ├── markdown.rs            # Markdown template engine
│       ├── markdown.test.rs       # Markdown generation tests
│       ├── codeblock.rs           # Code block formatting
│       └── markdown_formatting_demo.test.rs # Formatting demos
├── tests/                         # Integration and E2E tests
├── scripts/                       # Build and test automation
├── docs/                          # Project documentation
├── Makefile                       # Development workflow automation
├── Cargo.toml                     # Rust project configuration
└── README.md                      # Project overview and quick start
```

### Core Components

#### 🖥️ **Terminal Module** (`src/terminal/`)

- **Reliable shell integration** - Creates shell-specific hook files with dynamic session detection
- **Multi-shell support** (Bash, Zsh, Fish) with intelligent hook generation:
  - **Zsh**: Uses `preexec()` and `precmd()` functions for real-time capture
  - **Bash**: Uses `PROMPT_COMMAND` modification for command logging
  - **Fish**: Uses event-based functions for command capture
- **Dynamic session detection** - Hooks automatically find the most recent active session
- **Session-isolated capture** - Only commands from active DocPilot sessions
- **No shell history dependency** - Completely eliminates contamination from previous sessions
- **Smart log file management** - Background monitoring reads from correct hook log files
- **Automatic cleanup** - Hook files created in `~/.docpilot/` and cleaned up on session end

#### 🤖 **LLM Module** (`src/llm/`)

- **Multi-provider support** (Claude, ChatGPT, Gemini, Ollama)
- **Intelligent analysis** with context-aware prompts
- **Error handling** with retry logic and rate limiting
- **Configurable AI features** with provider-specific optimizations

#### 📊 **Session Module** (`src/session/`)

- **Persistent session storage** with JSON serialization
- **State management** (Active, Paused, Stopped, Error)
- **Annotation system** with multiple types (Note, Warning, Milestone, Explanation)
- **Session recovery** and backup functionality

#### 🔍 **Filter Module** (`src/filter/`)

- **Privacy protection** with sensitive data redaction
- **Command validation** and sequence dependency checking
- **Workflow optimization** suggestions and analysis
- **Typo detection** and suspicious command filtering

#### 📝 **Output Module** (`src/output/`)

- **8 documentation templates** (Standard, Minimal, Comprehensive, etc.)
- **Markdown generation** with syntax highlighting
- **Code block formatting** with language detection
- **Hierarchical organization** by workflow phases

### Testing Architecture

#### � **Comprehensive Test Coverage**

- **Unit Tests** (180+): Individual component validation
- **Integration Tests** (50+): Cross-component functionality
- **End-to-End Tests** (7 suites): Complete workflow validation
- **Performance Tests**: Stress testing with rapid operations

#### 🚀 **Automated E2E Testing**

- **No manual input required** - fully automated test execution
- **Complete functionality coverage** - tests all user-facing features
- **Cross-platform validation** - ensures compatibility across systems
- **Regression prevention** - catches breaking changes early

## 🧪 Testing

DocPilot has comprehensive test coverage with 258+ tests across multiple categories:

### Quick Testing with Makefile

DocPilot includes a comprehensive Makefile that simplifies common development tasks. For detailed documentation, see the [Makefile Guide](docs/makefile-guide.md).

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

**Made with ❤️ by Jason Kramer**

_Transform your terminal commands into beautiful documentation effortlessly._
