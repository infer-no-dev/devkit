# DevKit Usage Examples 📚

Real-world examples and tutorials for getting the most out of DevKit.

## 🎯 Basic Examples

### Code Generation

#### Generate Simple Functions
```bash
# Create a utility function
devkit generate "create a function that converts temperature from celsius to fahrenheit"

# Create with specific language
devkit generate "create a function to validate email addresses" --language python

# Save to specific file
devkit generate "create a JWT token validation function" \
    --language rust --output src/auth.rs
```

#### Generate Complex Code
```bash
# Generate a REST API endpoint
devkit generate "create a REST API endpoint for user registration with validation" \
    --language rust

# Generate with context from existing files
devkit generate "add error handling to this database connection" \
    --context src/database.rs --language rust

# Generate tests
devkit generate "create unit tests for the user authentication module" \
    --context src/auth.rs --language rust
```

### Codebase Analysis

#### Basic Analysis
```bash
# Analyze current directory
devkit analyze .

# Analyze with JSON output
devkit analyze . --format json > analysis.json

# Analyze specific directory
devkit analyze ./src --include-tests --include-docs
```

#### Advanced Analysis
```bash
# Quiet analysis (minimal output)
devkit analyze ./src --quiet

# Detailed analysis with file breakdown
devkit analyze . --format yaml --output project_analysis.yaml

# Analyze just core source files
devkit analyze ./src --exclude "*.test.*" --exclude "*.spec.*"
```

## 🏗️ Project Workflows

### Starting a New Project

```bash
# Initialize new project
devkit init ./my-rust-api

# Change to project directory
cd my-rust-api

# Install shell integration (if not done globally)
devkit shell install

# Check project status
devkit status

# Generate initial API structure
devkit generate "create a basic REST API structure with health check endpoint" \
    --language rust --output src/main.rs
```

### Working with Existing Projects

```bash
# Clone a project
git clone https://github.com/user/project.git
cd project

# Analyze the codebase to understand it
devkit analyze . --format json > codebase_analysis.json

# Generate new features based on existing patterns
devkit generate "add user authentication similar to existing admin auth" \
    --context src/auth/ --language rust

# Check that everything is working
devkit status
```

## 🎨 Language-Specific Examples

### Rust Development

```bash
# Generate Rust web server
devkit generate "create a basic Actix-web server with CORS middleware" \
    --language rust --output src/server.rs

# Generate error handling
devkit generate "create comprehensive error handling with custom error types" \
    --language rust --context src/main.rs

# Generate database models
devkit generate "create User and Post models with Diesel ORM relationships" \
    --language rust --output src/models.rs
```

### Python Development

```bash
# Generate Flask API
devkit generate "create a Flask REST API for task management with SQLAlchemy" \
    --language python --output app.py

# Generate data processing
devkit generate "create a function to process CSV data and generate statistics" \
    --language python

# Generate async code
devkit generate "create async function to fetch data from multiple APIs" \
    --language python --output src/api_client.py
```

### JavaScript/TypeScript

```bash
# Generate React component
devkit generate "create a reusable React component for user profile display" \
    --language typescript --output src/components/UserProfile.tsx

# Generate Node.js API
devkit generate "create Express.js API with JWT authentication middleware" \
    --language javascript --output server.js

# Generate utility functions
devkit generate "create utility functions for form validation and data formatting" \
    --language typescript --output src/utils.ts
```

## 🧰 Advanced Use Cases

### Refactoring Existing Code

```bash
# Analyze code for improvement opportunities
devkit analyze src/legacy_module.py --format json

# Generate improved version
devkit generate "refactor this code for better error handling and type safety" \
    --context src/legacy_module.py --language python \
    --output src/improved_module.py
```

### Adding Features to Existing Systems

```bash
# Understand existing authentication system
devkit analyze src/auth/ --format text

# Add new feature that integrates well
devkit generate "add password reset functionality that integrates with existing auth system" \
    --context src/auth/ --language rust \
    --output src/auth/password_reset.rs
```

### Documentation Generation

```bash
# Generate API documentation
devkit generate "create comprehensive API documentation for these endpoints" \
    --context src/routes/ --output README_API.md

# Generate code comments
devkit generate "add detailed documentation comments to this module" \
    --context src/core.rs --language rust
```

## 🔄 Workflow Patterns

### Daily Development Workflow

```bash
# Morning: Check project status
dk status

# Understand what you're working on
dk-analyze ./feature-branch

# Generate new functionality
dk-generate "implement user permission system with role-based access"

# Verify everything looks good
dk status
```

### Code Review Workflow

```bash
# Analyze changes in current branch
devkit analyze . --format json > current_analysis.json

# Compare with main branch analysis
git checkout main
devkit analyze . --format json > main_analysis.json

# Use analysis to understand impact of changes
diff main_analysis.json current_analysis.json
```

### Testing Workflow

```bash
# Generate comprehensive tests
devkit generate "create unit tests covering all edge cases for user validation" \
    --context src/user.rs --language rust --output tests/user_tests.rs

# Generate integration tests
devkit generate "create integration tests for the complete user registration flow" \
    --context src/ --language rust --output tests/integration/registration.rs
```

## 🎛️ Configuration Examples

### Project-Specific Configuration

Create `.devkit/config.toml`:
```toml
[general]
log_level = "debug"
auto_save = true

[codegen]
[codegen.ai_model_settings]
default_provider = "ollama"
default_model = "codellama:7b"  # Better for code generation
temperature = 0.3               # Lower temperature for more consistent code

[codegen.default_style]
indentation = "spaces"
indent_size = 2                 # For this project's style
line_length = 80
naming_convention = "camelCase" # JavaScript project
include_comments = true
include_type_hints = true
```

### Multi-Language Project

```toml
[codegen.language_preferences.rust]
style = { indentation = "spaces", indent_size = 4, naming_convention = "snake_case" }

[codegen.language_preferences.typescript]
style = { indentation = "spaces", indent_size = 2, naming_convention = "camelCase" }

[codegen.language_preferences.python]
style = { indentation = "spaces", indent_size = 4, naming_convention = "snake_case" }
```

## 🚀 Tips and Tricks

### Efficient Prompting

**Good prompts:**
```bash
# Specific and clear
devkit generate "create a function that validates JWT tokens and returns user claims"

# Includes context about expected behavior  
devkit generate "create middleware that logs request duration and handles errors gracefully"

# Specifies patterns to follow
devkit generate "create a database model following the existing User model pattern"
```

**Less effective prompts:**
```bash
# Too vague
devkit generate "make authentication"

# Missing context
devkit generate "fix this code"  # without --context

# Overly complex in single request
devkit generate "create entire user management system with auth, permissions, and admin panel"
```

### Using Context Effectively

```bash
# Single file context
devkit generate "add caching to this service" --context src/user_service.rs

# Multiple files context
devkit generate "create integration between user and order systems" \
    --context src/user.rs --context src/order.rs

# Directory context for understanding patterns
devkit generate "create new API endpoint following existing patterns" \
    --context src/api/
```

### Shell Integration Power Features

After `devkit shell install`:

```bash
# Tab completion works for all commands
dk generate <TAB>          # Shows generate options
dk analyze --format <TAB>  # Shows: json, yaml, text

# Quick aliases
dk status                  # Check system health
dk-analyze ./src          # Quick analysis
dk-generate "prompt here" # Quick generation

# Combine with other tools
dk analyze . --format json | jq '.files[] | select(.language == "rust")'
```

## 🔧 Troubleshooting Examples

### Debug AI Generation Issues

```bash
# Check system status first
devkit status

# Test with simple prompt
devkit generate "create hello world function" --language rust

# Check logs for detailed error info
tail -f logs/devkit.log

# Test Ollama connection
curl http://localhost:11434/api/tags
```

### Debug Analysis Issues

```bash
# Test analysis on single file
devkit analyze src/main.rs --format text

# Check if tree-sitter parsers are working
devkit analyze --verbose ./src

# Test with minimal flags
devkit analyze . --quiet --format json
```

---

## 📝 Contributing Examples

Have a great use case or example? We'd love to include it!

1. Fork the repository
2. Add your example to this file
3. Test that it works with current DevKit version
4. Submit a PR with clear description

---

**More examples?** Check our [GitHub discussions](https://github.com/infer-no-dev/devkit/discussions) for community-shared workflows! 🎉