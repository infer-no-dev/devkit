#!/usr/bin/env bash

# DevKit Development Environment Setup Script
# This script sets up the development environment for contributing to devkit

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust if not present
install_rust() {
    if command_exists rustc; then
        local rust_version
        rust_version=$(rustc --version | cut -d' ' -f2)
        log_info "Rust is already installed (version: $rust_version)"
        return 0
    fi

    log_info "Installing Rust..."
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "${HOME}/.cargo/env"
        log_success "Rust installed successfully"
    else
        log_error "curl is required to install Rust. Please install curl first."
        exit 1
    fi
}

# Install required Rust components
install_rust_components() {
    log_info "Installing required Rust components..."
    
    local components=(
        "rustfmt"
        "clippy"
        "llvm-tools-preview"
    )
    
    for component in "${components[@]}"; do
        if ! rustup component list --installed | grep -q "^$component"; then
            log_info "Installing $component..."
            rustup component add "$component"
            log_success "$component installed"
        else
            log_info "$component already installed"
        fi
    done
}

# Install development tools
install_dev_tools() {
    log_info "Installing development tools..."
    
    local tools=(
        "cargo-audit:Security audit tool"
        "cargo-outdated:Check for outdated dependencies"
        "cargo-udeps:Find unused dependencies"
        "cargo-llvm-cov:Code coverage"
        "cargo-watch:Auto-rebuild on changes"
        "cargo-edit:Edit Cargo.toml from command line"
    )
    
    for tool_info in "${tools[@]}"; do
        local tool_name="${tool_info%%:*}"
        local tool_desc="${tool_info##*:}"
        
        if ! command_exists "$tool_name"; then
            log_info "Installing $tool_name ($tool_desc)..."
            if [[ "$tool_name" == "cargo-udeps" ]]; then
                # cargo-udeps requires nightly
                rustup install nightly
                cargo +nightly install cargo-udeps --locked
            else
                cargo install "$tool_name"
            fi
            log_success "$tool_name installed"
        else
            log_info "$tool_name already installed"
        fi
    done
}

# Install system dependencies based on OS
install_system_deps() {
    log_info "Checking system dependencies..."
    
    case "$(uname -s)" in
        Linux*)
            if command_exists apt-get; then
                log_info "Detected Debian/Ubuntu system"
                sudo apt-get update
                sudo apt-get install -y build-essential pkg-config libssl-dev git
            elif command_exists yum; then
                log_info "Detected Red Hat/CentOS system"
                sudo yum groupinstall -y "Development Tools"
                sudo yum install -y openssl-devel git
            elif command_exists pacman; then
                log_info "Detected Arch Linux system"
                sudo pacman -S --needed base-devel openssl git
            else
                log_warning "Unknown Linux distribution. Please ensure you have:"
                log_warning "- C compiler (gcc/clang)"
                log_warning "- pkg-config"
                log_warning "- OpenSSL development headers"
                log_warning "- git"
            fi
            ;;
        Darwin*)
            log_info "Detected macOS system"
            if command_exists xcode-select; then
                xcode-select --install 2>/dev/null || true
            fi
            if command_exists brew; then
                brew install openssl git
            else
                log_warning "Homebrew not found. Please install manually:"
                log_warning "- Xcode Command Line Tools"
                log_warning "- OpenSSL"
                log_warning "- git"
            fi
            ;;
        CYGWIN*|MINGW*|MSYS*)
            log_info "Detected Windows system"
            log_warning "Please ensure you have the following installed:"
            log_warning "- Visual Studio Build Tools or Visual Studio Community"
            log_warning "- git for Windows"
            ;;
        *)
            log_warning "Unknown operating system: $(uname -s)"
            log_warning "Please ensure you have a C compiler and git installed"
            ;;
    esac
}

# Set up git hooks
setup_git_hooks() {
    if [[ ! -d .git ]]; then
        log_warning "Not in a git repository. Skipping git hooks setup."
        return 0
    fi
    
    log_info "Setting up git hooks..."
    
    # Pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/usr/bin/env bash

set -euo pipefail

echo "Running pre-commit checks..."

# Check formatting
if ! cargo fmt --all -- --check; then
    echo "Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "Clippy found issues. Please fix them."
    exit 1
fi

# Run tests
if ! cargo test; then
    echo "Tests failed. Please fix them."
    exit 1
fi

echo "All pre-commit checks passed!"
EOF

    chmod +x .git/hooks/pre-commit
    log_success "Git hooks set up successfully"
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    # Build the project
    if cargo build; then
        log_success "Project builds successfully"
    else
        log_error "Project failed to build"
        return 1
    fi
    
    # Run tests
    if cargo test --lib; then
        log_success "Library tests pass"
    else
        log_error "Some tests failed"
        return 1
    fi
    
    # Check formatting
    if cargo fmt --all -- --check; then
        log_success "Code is properly formatted"
    else
        log_warning "Code formatting needs attention. Run 'cargo fmt' to fix."
    fi
    
    # Run clippy
    if cargo clippy --all-targets --all-features -- -D warnings; then
        log_success "No clippy warnings"
    else
        log_warning "Clippy found some issues. Please review them."
    fi
}

# Create example environment configuration
create_env_example() {
    if [[ ! -f .env.example ]]; then
        log_info "Creating example environment file..."
        cat > .env.example << 'EOF'
# Example environment configuration for devkit development

# Logging configuration
RUST_LOG=devkit=debug,info
RUST_BACKTRACE=1

# AI Provider Configuration (optional)
# OPENAI_API_KEY=your-openai-api-key
# ANTHROPIC_API_KEY=your-anthropic-api-key

# Ollama Configuration (if using local Ollama)
OLLAMA_ENDPOINT=http://localhost:11434

# Development flags
DEV_MODE=true
EOF
        log_success "Created .env.example file"
        log_info "Copy it to .env and customize as needed: cp .env.example .env"
    fi
}

# Main installation function
main() {
    echo "=================================================="
    echo "DevKit Development Environment Setup"
    echo "=================================================="
    
    install_system_deps
    install_rust
    install_rust_components
    install_dev_tools
    setup_git_hooks
    create_env_example
    verify_installation
    
    echo "=================================================="
    log_success "Development environment setup complete!"
    echo "=================================================="
    
    echo
    log_info "Next steps:"
    echo "  1. Run 'cargo build' to build the project"
    echo "  2. Run 'cargo test' to run tests"
    echo "  3. Run 'cargo clippy' for linting"
    echo "  4. Run 'cargo fmt' to format code"
    echo "  5. Start developing! 🎉"
    echo
    log_info "Useful development commands:"
    echo "  • cargo watch -x build          # Auto-rebuild on changes"
    echo "  • cargo watch -x test           # Auto-test on changes"
    echo "  • cargo audit                   # Security audit"
    echo "  • cargo outdated               # Check for outdated deps"
    echo "  • cargo llvm-cov --html        # Generate coverage report"
}

# Run main function
main "$@"