# Interactive Mode Commands Test Plan

## Overview
This document outlines comprehensive tests for all interactive mode commands and features to ensure everything works correctly after our persistent input mode changes.

## Test Categories

### 1. Basic UI/Input Functionality
- [x] Start interactive mode
- [x] Enter input mode with 'i' (stays persistent)
- [x] Enter command mode with ':' (stays persistent) 
- [x] Exit modes with ESC
- [x] Quit with 'q'
- [ ] Status bar shows correct messages
- [ ] Multiple commands in sequence work

### 2. File System Commands
- [ ] `/ls` - List current directory
- [ ] `/ls <path>` - List specific directory
- [ ] `/cd <path>` - Change directory
- [ ] `/pwd` - Show current directory

### 3. Session Management Commands
- [ ] `/save [file]` - Save session
- [ ] `/load <file>` - Load session
- [ ] `/sessions` - List available sessions
- [ ] `/session new [path]` - Create new session
- [ ] `/session switch <id>` - Switch sessions
- [ ] `/session delete <id>` - Delete session
- [ ] `/session clone <id>` - Clone session
- [ ] `/history` - Show conversation history
- [ ] `/artifacts` - Show code artifacts

### 4. Bookmark Commands
- [ ] `/bookmark create <name> <description>` - Create bookmark
- [ ] `/bookmark list` - List bookmarks
- [ ] `/bookmark goto <id>` - Go to bookmark
- [ ] `/bookmark delete <id>` - Delete bookmark

### 5. System/Agent Commands
- [ ] `/status` - Show system status
- [ ] `/agents` - List active agents
- [ ] `/tasks` - Show active tasks
- [ ] `/restart` - Restart agent system
- [ ] `/config [key] [value]` - Show/update config

### 6. Interface Commands
- [ ] `/clear` - Clear screen
- [ ] `/theme [name]` - Change theme (dark/light/blue/green)
- [ ] `/layout [type]` - Change layout (single/split/three/quad)
- [ ] `/help` - Show help message
- [ ] `/quit` or `/exit` - Exit interactive mode

### 7. Natural Language Processing
- [ ] General chat messages
- [ ] Generate commands
- [ ] Debug/fix commands
- [ ] Explain commands
- [ ] Optimize commands
- [ ] Refine commands

### 8. Error Handling
- [ ] Invalid commands show proper error messages
- [ ] Missing arguments show usage help
- [ ] File system errors handled gracefully
- [ ] Agent system errors handled gracefully

## Test Execution Plan

### Phase 1: Basic Functionality Tests
### Phase 2: File System Operations
### Phase 3: Session Management
### Phase 4: Advanced Features
### Phase 5: Error Conditions