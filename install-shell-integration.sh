#!/bin/bash

# Agentic Development Environment - Shell Integration Installer
# This script installs shell integration for the agentic-dev-env tool

set -euo pipefail

# Configuration
PROJECT_DIR="/home/rga/projects/agentic-dev-env"
BINARY_NAME="agentic-dev-env"
SHELL_NAME=$(basename "$SHELL")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect current shell
detect_shell() {
    case "$SHELL_NAME" in
        bash)
            CONFIG_FILE="$HOME/.bashrc"
            COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
            ;;
        zsh)
            CONFIG_FILE="$HOME/.zshrc"
            COMPLETION_DIR="$HOME/.local/share/zsh/site-functions"
            ;;
        fish)
            CONFIG_FILE="$HOME/.config/fish/config.fish"
            COMPLETION_DIR="$HOME/.config/fish/completions"
            ;;
        *)
            print_error "Unsupported shell: $SHELL_NAME"
            exit 1
            ;;
    esac
    
    print_info "Detected shell: $SHELL_NAME"
    print_info "Config file: $CONFIG_FILE"
    print_info "Completion directory: $COMPLETION_DIR"
}

# Create necessary directories
create_directories() {
    print_info "Creating directories..."
    mkdir -p "$(dirname "$CONFIG_FILE")" 2>/dev/null || true
    mkdir -p "$COMPLETION_DIR" 2>/dev/null || true
}

# Build the project if not already built
build_project() {
    print_info "Checking if project needs to be built..."
    
    if [[ ! -f "$PROJECT_DIR/target/release/$BINARY_NAME" && ! -f "$PROJECT_DIR/target/debug/$BINARY_NAME" ]]; then
        print_info "Binary not found. Attempting to build project..."
        
        cd "$PROJECT_DIR"
        
        # Try release build first, fall back to debug if it fails
        if cargo build --release 2>/dev/null; then
            BINARY_PATH="$PROJECT_DIR/target/release/$BINARY_NAME"
            print_success "Release build completed successfully!"
        elif cargo build 2>/dev/null; then
            BINARY_PATH="$PROJECT_DIR/target/debug/$BINARY_NAME"
            print_warning "Release build failed, using debug build."
        else
            print_warning "Build failed. Creating stub for future use."
            BINARY_PATH=""
        fi
    else
        # Find existing binary
        if [[ -f "$PROJECT_DIR/target/release/$BINARY_NAME" ]]; then
            BINARY_PATH="$PROJECT_DIR/target/release/$BINARY_NAME"
        else
            BINARY_PATH="$PROJECT_DIR/target/debug/$BINARY_NAME"
        fi
        print_info "Found existing binary: $BINARY_PATH"
    fi
}

# Install shell alias/function
install_alias() {
    print_info "Installing shell alias..."
    
    local alias_content=""
    
    case "$SHELL_NAME" in
        bash|zsh)
            if [[ -n "$BINARY_PATH" && -f "$BINARY_PATH" ]]; then
                alias_content="
# Agentic Development Environment
export AGENTIC_DEV_ENV_HOME=\"$PROJECT_DIR\"
alias ade=\"$BINARY_PATH\"
alias agentic-dev-env=\"$BINARY_PATH\"

# Helper functions
ade-start() {
    \"$BINARY_PATH\" start --project \"\${1:-\$PWD}\"
}

ade-generate() {
    \"$BINARY_PATH\" generate \"\$*\"
}

ade-analyze() {
    \"$BINARY_PATH\" analyze \"\${1:-\$PWD}\"
}
"
            else
                alias_content="
# Agentic Development Environment (placeholder - build not ready)
export AGENTIC_DEV_ENV_HOME=\"$PROJECT_DIR\"
alias ade=\"echo 'Build the project first with: cd $PROJECT_DIR && cargo build --release'\"
alias agentic-dev-env=\"echo 'Build the project first with: cd $PROJECT_DIR && cargo build --release'\"
"
            fi
            ;;
        fish)
            if [[ -n "$BINARY_PATH" && -f "$BINARY_PATH" ]]; then
                alias_content="
# Agentic Development Environment
set -gx AGENTIC_DEV_ENV_HOME \"$PROJECT_DIR\"
alias ade \"$BINARY_PATH\"
alias agentic-dev-env \"$BINARY_PATH\"

# Helper functions
function ade-start
    \"$BINARY_PATH\" start --project (if test (count \$argv) -gt 0; echo \$argv[1]; else; echo \$PWD; end)
end

function ade-generate
    \"$BINARY_PATH\" generate \$argv
end

function ade-analyze
    \"$BINARY_PATH\" analyze (if test (count \$argv) -gt 0; echo \$argv[1]; else; echo \$PWD; end)
end
"
            else
                alias_content="
# Agentic Development Environment (placeholder - build not ready)
set -gx AGENTIC_DEV_ENV_HOME \"$PROJECT_DIR\"
alias ade \"echo 'Build the project first with: cd $PROJECT_DIR && cargo build --release'\"
alias agentic-dev-env \"echo 'Build the project first with: cd $PROJECT_DIR && cargo build --release'\"
"
            fi
            ;;
    esac
    
    # Check if aliases are already installed
    if ! grep -q "Agentic Development Environment" "$CONFIG_FILE" 2>/dev/null; then
        echo "$alias_content" >> "$CONFIG_FILE"
        print_success "Aliases installed to $CONFIG_FILE"
    else
        print_info "Aliases already installed, skipping..."
    fi
}

# Generate completion scripts
generate_completions() {
    print_info "Generating completion scripts..."
    
    case "$SHELL_NAME" in
        bash)
            cat > "$COMPLETION_DIR/agentic-dev-env" << 'EOF'
#!/bin/bash

_agentic_dev_env_completions() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    case ${COMP_CWORD} in
        1)
            COMPREPLY=($(compgen -W "start init config analyze generate version help" -- ${cur}))
            ;;
        2)
            case ${prev} in
                start)
                    COMPREPLY=($(compgen -W "--project" -- ${cur}))
                    ;;
                config)
                    COMPREPLY=($(compgen -W "--show --reset --export --import" -- ${cur}))
                    ;;
                analyze)
                    COMPREPLY=($(compgen -W "--output --dependencies" -- ${cur}))
                    ;;
                generate)
                    COMPREPLY=($(compgen -W "--file --language" -- ${cur}))
                    ;;
            esac
            ;;
        *)
            case ${prev} in
                --project|--file|--output|--export|--import)
                    COMPREPLY=($(compgen -f -- ${cur}))
                    ;;
                --language)
                    COMPREPLY=($(compgen -W "rust python javascript typescript go java c cpp" -- ${cur}))
                    ;;
            esac
            ;;
    esac
}

complete -F _agentic_dev_env_completions agentic-dev-env
complete -F _agentic_dev_env_completions ade
EOF
            print_success "Bash completion installed"
            ;;
        zsh)
            cat > "$COMPLETION_DIR/_agentic-dev-env" << 'EOF'
#compdef agentic-dev-env ade

_agentic_dev_env() {
    local state line
    typeset -A opt_args
    
    _arguments -C \
        '1: :->command' \
        '*: :->args' \
        && return 0
    
    case $state in
        command)
            _values 'command' \
                'start[Start the interactive development environment]' \
                'init[Initialize configuration for a new project]' \
                'config[Configure the development environment]' \
                'analyze[Analyze a codebase and generate context]' \
                'generate[Generate code from a natural language prompt]' \
                'version[Show version information]' \
                'help[Show help information]'
            ;;
        args)
            case $line[1] in
                start)
                    _arguments \
                        '--project[Project path to analyze]:path:_files -/'
                    ;;
                config)
                    _arguments \
                        '--show[Show current configuration]' \
                        '--reset[Reset to default configuration]' \
                        '--export[Export configuration to JSON]:path:_files' \
                        '--import[Import configuration from JSON]:path:_files'
                    ;;
                analyze)
                    _arguments \
                        '--output[Output file for analysis results]:path:_files' \
                        '--dependencies[Include dependency analysis]' \
                        '1:path:_files -/'
                    ;;
                generate)
                    _arguments \
                        '--file[Target file path]:path:_files' \
                        '--language[Target programming language]:language:(rust python javascript typescript go java c cpp)' \
                        '1:prompt:'
                    ;;
            esac
            ;;
    esac
}

_agentic_dev_env "$@"
EOF
            print_success "Zsh completion installed"
            ;;
        fish)
            cat > "$COMPLETION_DIR/agentic-dev-env.fish" << 'EOF'
# Completions for agentic-dev-env

complete -c agentic-dev-env -f
complete -c ade -f

# Subcommands
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "start" -d "Start the interactive development environment"
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "init" -d "Initialize configuration for a new project"
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "config" -d "Configure the development environment"
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "analyze" -d "Analyze a codebase and generate context"
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "generate" -d "Generate code from a natural language prompt"
complete -c agentic-dev-env -n "__fish_use_subcommand" -a "version" -d "Show version information"

# Options for start subcommand
complete -c agentic-dev-env -n "__fish_seen_subcommand_from start" -l project -d "Project path to analyze" -r

# Options for config subcommand
complete -c agentic-dev-env -n "__fish_seen_subcommand_from config" -l show -d "Show current configuration"
complete -c agentic-dev-env -n "__fish_seen_subcommand_from config" -l reset -d "Reset to default configuration"
complete -c agentic-dev-env -n "__fish_seen_subcommand_from config" -l export -d "Export configuration to JSON" -r
complete -c agentic-dev-env -n "__fish_seen_subcommand_from config" -l import -d "Import configuration from JSON" -r

# Options for analyze subcommand
complete -c agentic-dev-env -n "__fish_seen_subcommand_from analyze" -l output -d "Output file for analysis results" -r
complete -c agentic-dev-env -n "__fish_seen_subcommand_from analyze" -l dependencies -d "Include dependency analysis"

# Options for generate subcommand
complete -c agentic-dev-env -n "__fish_seen_subcommand_from generate" -l file -d "Target file path" -r
complete -c agentic-dev-env -n "__fish_seen_subcommand_from generate" -l language -d "Target programming language" -xa "rust python javascript typescript go java c cpp"

# Copy completions for ade alias
complete -c ade -w agentic-dev-env
EOF
            print_success "Fish completion installed"
            ;;
    esac
}

# Create desktop entry (optional)
create_desktop_entry() {
    print_info "Creating desktop entry..."
    
    local desktop_dir="$HOME/.local/share/applications"
    mkdir -p "$desktop_dir"
    
    if [[ -n "$BINARY_PATH" && -f "$BINARY_PATH" ]]; then
        cat > "$desktop_dir/agentic-dev-env.desktop" << EOF
[Desktop Entry]
Version=1.0
Name=Agentic Development Environment
Comment=AI-assisted development environment
Exec=$BINARY_PATH
Icon=utilities-terminal
Terminal=true
Type=Application
Categories=Development;TextEditor;
Keywords=AI;Development;Code;Assistant;
EOF
        print_success "Desktop entry created"
    else
        print_info "Skipping desktop entry (binary not available)"
    fi
}

# Main installation function
main() {
    echo "=============================================="
    echo "Agentic Development Environment"
    echo "Shell Integration Installer"
    echo "=============================================="
    echo
    
    detect_shell
    create_directories
    build_project
    install_alias
    generate_completions
    create_desktop_entry
    
    echo
    echo "=============================================="
    print_success "Shell integration installation complete!"
    echo "=============================================="
    echo
    echo "To use the new aliases and completions, either:"
    echo "  1. Restart your terminal"
    echo "  2. Or run: source $CONFIG_FILE"
    echo
    echo "Available commands:"
    echo "  • ade                 - Short alias for agentic-dev-env"
    echo "  • agentic-dev-env     - Full command name"
    echo "  • ade-start [path]    - Start interactive mode"
    echo "  • ade-generate <text> - Generate code from description"
    echo "  • ade-analyze [path]  - Analyze codebase"
    echo
    if [[ -z "$BINARY_PATH" || ! -f "$BINARY_PATH" ]]; then
        print_warning "Note: The project needs to be built before you can use these commands."
        echo "To build: cd $PROJECT_DIR && cargo build --release"
    fi
}

# Run main function
main "$@"
