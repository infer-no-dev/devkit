#!/bin/bash
set -e

# DevKit Development Script
# Usage: ./scripts/dev.sh [command]

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_DIR="$( cd "$SCRIPT_DIR/.." &> /dev/null && pwd )"

cd "$PROJECT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Quick development setup
setup() {
    print_header "Setting up DevKit for development"
    
    # Install required tools
    if ! command -v cargo-audit &> /dev/null; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    if ! command -v cargo-deny &> /dev/null; then
        echo "Installing cargo-deny..."
        cargo install cargo-deny
    fi
    
    if ! command -v cargo-watch &> /dev/null; then
        echo "Installing cargo-watch..."
        cargo install cargo-watch
    fi
    
    print_success "Development environment ready"
}

# Run full test suite
test_all() {
    print_header "Running comprehensive test suite"
    
    echo "1. Running unit tests..."
    cargo test --lib
    
    echo -e "\n2. Running integration tests..."
    cargo test --test '*'
    
    echo -e "\n3. Running benchmarks..."
    cargo bench
    
    echo -e "\n4. Running property-based tests..."
    cargo test property_tests
    
    print_success "All tests completed"
}

# Quality checks
check_quality() {
    print_header "Running code quality checks"
    
    echo "1. Checking compilation..."
    cargo check
    
    echo -e "\n2. Running Clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo -e "\n3. Checking formatting..."
    cargo fmt -- --check
    
    echo -e "\n4. Security audit..."
    if command -v cargo-audit &> /dev/null; then
        cargo audit
    else
        print_warning "cargo-audit not installed, skipping security audit"
    fi
    
    echo -e "\n5. Dependency analysis..."
    if command -v cargo-deny &> /dev/null; then
        cargo deny check
    else
        print_warning "cargo-deny not installed, skipping dependency analysis"
    fi
    
    print_success "Quality checks completed"
}

# Development watch mode
watch() {
    print_header "Starting development watch mode"
    
    if command -v cargo-watch &> /dev/null; then
        cargo watch -x 'check' -x 'test --lib' -x 'clippy -- -D warnings'
    else
        print_error "cargo-watch not installed. Run: cargo install cargo-watch"
        exit 1
    fi
}

# Performance profiling
profile() {
    print_header "Running performance profiling"
    
    echo "Building release binary..."
    cargo build --release
    
    echo "Running benchmarks with profiling..."
    cargo bench -- --profile-time=5
    
    if command -v perf &> /dev/null; then
        echo "Running perf analysis..."
        perf record --call-graph=dwarf -o devkit.perf ./target/release/devkit --version
        perf report -i devkit.perf
    else
        print_warning "perf not available, skipping detailed profiling"
    fi
    
    print_success "Profiling completed"
}

# Clean everything
clean() {
    print_header "Cleaning build artifacts"
    
    cargo clean
    rm -rf ./target/criterion
    rm -f ./*.perf
    
    print_success "Clean completed"
}

# Release preparation
prepare_release() {
    local version=$1
    if [ -z "$version" ]; then
        print_error "Version required. Usage: ./dev.sh release [version]"
        exit 1
    fi
    
    print_header "Preparing release v$version"
    
    echo "1. Running full quality checks..."
    check_quality
    
    echo -e "\n2. Running comprehensive tests..."
    test_all
    
    echo -e "\n3. Building release binary..."
    cargo build --release
    
    echo -e "\n4. Running final integration tests..."
    ./target/release/devkit --version
    ./target/release/devkit --help
    
    print_success "Release v$version is ready!"
    echo "Next steps:"
    echo "  1. Update CHANGELOG.md"
    echo "  2. git tag v$version"
    echo "  3. git push origin v$version"
}

# Show help
show_help() {
    echo "DevKit Development Script"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  setup           Set up development environment"
    echo "  test            Run all tests"
    echo "  check           Run quality checks (lint, format, audit)"
    echo "  watch           Start development watch mode"
    echo "  profile         Run performance profiling"
    echo "  clean           Clean build artifacts"
    echo "  release [ver]   Prepare release with version"
    echo "  help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 setup"
    echo "  $0 test"
    echo "  $0 watch"
    echo "  $0 release 0.1.1"
}

# Main command dispatcher
case "${1:-help}" in
    "setup")
        setup
        ;;
    "test")
        test_all
        ;;
    "check")
        check_quality
        ;;
    "watch")
        watch
        ;;
    "profile")
        profile
        ;;
    "clean")
        clean
        ;;
    "release")
        prepare_release "$2"
        ;;
    "help"|*)
        show_help
        ;;
esac