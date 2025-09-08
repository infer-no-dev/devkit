#!/usr/bin/env bash

# devkit installation script
# Downloads and installs the latest devkit binary

set -euo pipefail

# Configuration
GITHUB_REPO="infer-no-dev/devkit"
BINARY_NAME="devkit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect system architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case $arch in
        x86_64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) log_error "Unsupported architecture: $arch"; exit 1 ;;
    esac
}

# Detect operating system
detect_os() {
    local os
    os=$(uname -s)
    case $os in
        Linux) echo "unknown-linux-gnu" ;;
        Darwin) echo "apple-darwin" ;;
        *) log_error "Unsupported operating system: $os"; exit 1 ;;
    esac
}

# Get latest release info from GitHub
get_latest_release() {
    local repo="$1"
    local release_info
    
    log_info "Fetching latest release information..."
    
    if command -v curl >/dev/null 2>&1; then
        release_info=$(curl -s "https://api.github.com/repos/${repo}/releases/latest")
    elif command -v wget >/dev/null 2>&1; then
        release_info=$(wget -qO- "https://api.github.com/repos/${repo}/releases/latest")
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    # Extract tag name using basic text processing (no jq dependency)
    echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
}

# Download and install binary
install_binary() {
    local version="$1"
    local arch="$2"
    local os="$3"
    
    local binary_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/${BINARY_NAME}-${arch}-${os}"
    local temp_file="/tmp/${BINARY_NAME}"
    
    log_info "Downloading ${BINARY_NAME} ${version} for ${arch}-${os}..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -sL "$binary_url" -o "$temp_file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$binary_url" -O "$temp_file"
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    if [[ ! -f "$temp_file" ]]; then
        log_error "Failed to download binary from $binary_url"
        exit 1
    fi
    
    # Make sure install directory exists
    mkdir -p "$INSTALL_DIR"
    
    # Make binary executable and move to install directory
    chmod +x "$temp_file"
    mv "$temp_file" "${INSTALL_DIR}/${BINARY_NAME}"
    
    log_success "${BINARY_NAME} installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Check if binary is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "To use ${BINARY_NAME} from anywhere, add this line to your shell profile:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "For bash: ~/.bashrc or ~/.bash_profile"
        echo "For zsh: ~/.zshrc"
        echo "For fish: ~/.config/fish/config.fish"
        echo ""
        echo "Or run: echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
    fi
}

# Main installation function
main() {
    echo "devkit Installation Script"
    echo "=========================="
    echo ""
    
    # Check if we're updating an existing installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local current_version
        current_version=$("$BINARY_NAME" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        log_info "Current version: $current_version"
    fi
    
    # Detect system
    local arch os
    arch=$(detect_arch)
    os=$(detect_os)
    
    log_info "Detected system: ${arch}-${os}"
    
    # Get latest version
    local latest_version
    latest_version=$(get_latest_release "$GITHUB_REPO")
    
    if [[ -z "$latest_version" ]]; then
        log_error "Failed to get latest release information"
        exit 1
    fi
    
    log_info "Latest version: $latest_version"
    
    # Install binary
    install_binary "$latest_version" "$arch" "$os"
    
    # Check PATH
    check_path
    
    # Verify installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        log_success "Installation complete! Installed version: $installed_version"
        echo ""
        echo "Try running: $BINARY_NAME --help"
    else
        log_error "Installation verification failed. The binary may not be in your PATH."
        echo "Binary location: ${INSTALL_DIR}/${BINARY_NAME}"
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "devkit installation script"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --uninstall    Remove devkit"
        echo ""
        echo "Environment variables:"
        echo "  INSTALL_DIR    Installation directory (default: ~/.local/bin)"
        exit 0
        ;;
    --uninstall)
        if [[ -f "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
            rm "${INSTALL_DIR}/${BINARY_NAME}"
            log_success "devkit uninstalled from ${INSTALL_DIR}/${BINARY_NAME}"
        else
            log_warn "devkit not found in ${INSTALL_DIR}"
        fi
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
