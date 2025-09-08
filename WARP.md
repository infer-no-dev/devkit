# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is an intelligent, multi-agent development environment built in Rust, designed for AI-assisted code generation on large existing codebases. The system leverages multiple concurrent AI agents, advanced code analysis using tree-sitter, and cross-shell compatibility to provide natural language programming assistance.

## Development Commands

### Build and Run
```bash
# Build the project (release mode for performance)
cargo build --release

# Build for development
cargo build

# Run the application
cargo run

# Run with specific subcommands
cargo run -- start --project ./path/to/project
cargo run -- init ./new-project
cargo run -- analyze ./codebase --output analysis.json
cargo run -- generate "create a function to parse JSON" --language rust
```

### Code Quality
```bash
# Check compilation without building
cargo check

# Run clippy for linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test agents::
```

### Development Utilities
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Check for unused dependencies
cargo machete

# Run with debug logging
RUST_LOG=debug cargo run

# Run with trace logging
RUST_LOG=trace cargo run
```

## Architecture Overview

The codebase is organized into six core modules that work together to provide the agentic development environment:

### Agent System (`src/agents/`)
- **Multi-agent coordination**: Manages concurrent AI agents with task prioritization
- **Agent types**: Different specialized agents for code generation, analysis, and debugging
- **Communication**: Inter-agent messaging and coordination protocols
- **Task management**: Assignment, tracking, and result aggregation

### Code Generation (`src/codegen/`)
- **Natural language processing**: Converts prompts to code using context analysis
- **Language detection**: Automatically identifies target programming languages
- **Style preferences**: Configurable code formatting and naming conventions
- **Template system**: Reusable code templates for common patterns
- **Code analysis**: Quality assessment and improvement suggestions

### Context Management (`src/context/`)
- **Codebase analysis**: Deep analysis of file structures and relationships
- **Symbol indexing**: Comprehensive symbol table for cross-references
- **Dependency tracking**: Automatic detection of project dependencies
- **Repository integration**: Git integration for version control awareness
- **Relationship mapping**: Understanding of imports, inheritance, and references

### Shell Integration (`src/shell/`)
- **Cross-platform support**: Native integration with bash, zsh, fish, and PowerShell
- **Command execution**: Safe command execution with timeout and error handling
- **Environment management**: Shell environment variable handling
- **Shell detection**: Automatic detection of current shell type

### User Interface (`src/ui/`)
- **Terminal-based UI**: Built with ratatui for rich terminal interfaces
- **Block-based I/O**: Organized input/output grouping
- **Real-time monitoring**: Live agent status and progress tracking
- **Customizable layouts**: Configurable panel arrangements

### Configuration (`src/config/`)
- **Hierarchical config**: Project-level and global configuration support
- **Runtime settings**: Dynamic configuration updates
- **Validation**: Configuration validation and error reporting
- **Import/export**: JSON-based configuration portability

## Key Development Patterns

### Agent Development
When creating new agents:
- Implement the `Agent` trait with `process_task()` method
- Handle `AgentTask` with proper error handling using `AgentError`
- Support task prioritization via `TaskPriority` enum
- Use `AgentStatus` to communicate current state
- Produce `AgentResult` with artifacts and next actions

### Context-Aware Code Generation
The system uses comprehensive codebase context:
- `CodebaseContext` provides full project understanding
- `FileContext` contains file-level symbols and relationships
- `SymbolIndex` enables cross-reference lookups
- Always consider existing code patterns and naming conventions

### Shell Command Integration
For cross-platform compatibility:
- Use `ShellManager` for all command execution
- Handle different shell syntaxes via `CommandOperation`
- Set appropriate timeouts for long-running operations
- Use `ShellConfig` for environment customization

### Configuration Management
- Use `ConfigManager` for all settings access
- Validate changes with `ConfigValidator`
- Support both TOML and JSON formats
- Implement graceful fallbacks to defaults

## Important Implementation Details

### Error Handling
- Each module defines specific error types using `thiserror`
- Use `anyhow` for application-level error handling
- Propagate errors appropriately through the async call chain
- Provide meaningful error messages for user-facing operations

### Async Architecture
- Built on `tokio` runtime for concurrent operations
- Agent operations are naturally async for parallelism
- File I/O operations use async patterns
- Command execution supports timeouts and cancellation

### Memory Management
- Use `Arc<RwLock<>>` for shared mutable state
- Clone contexts efficiently for parallel processing
- Cache analysis results when appropriate
- Clean up temporary files and resources

### Dependencies and Build Issues
- **Known Issue**: `Cargo.toml` contains typo - should be `reqwest` not `requwest`
- Tree-sitter integration requires proper grammar dependencies
- Cross-platform shell support requires platform-specific testing
- UI components depend on terminal capability detection

## Testing Strategy

The project structure suggests these testing approaches:
- Unit tests for individual agent behaviors
- Integration tests for agent coordination
- End-to-end tests for complete workflows
- Shell integration tests across platforms
- UI component tests with mock terminals

## Configuration Files

### Project Configuration
- `.agentic-config.toml` - Project-specific settings
- `config.toml` - User configuration (gitignored)

### Supported Languages
The context analyzer and code generator support detection and generation for multiple languages based on file extensions and content analysis. Language-specific formatting and style rules are configurable per project.

## Performance Considerations

- Analysis caching prevents repeated file parsing
- Async operations allow concurrent agent processing  
- Symbol indexing optimizes cross-reference lookups
- Tree-sitter provides efficient syntax analysis
- Configuration validation happens at load time

## Security Notes

- Shell command execution includes timeout protection
- File system operations respect permission boundaries
- No sensitive data should be logged at info level
- Configuration files may contain API keys - handle appropriately
