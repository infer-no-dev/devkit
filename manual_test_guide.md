# Manual Test Guide for Interactive Mode

## 🎯 Important: These tests verify our persistent input mode works correctly

### Test 1: Basic Persistent Input Mode ✨
1. Run: `./target/release/devkit interactive`
2. Press `i` to enter input mode
3. **Verify status bar says**: "INPUT MODE: Type commands and press Enter to submit | ESC: Exit to normal mode | Multiple commands allowed"
4. Type: `/help` and press Enter
5. **Expected**: Should show help text AND stay in input mode
6. Type: `/status` and press Enter  
7. **Expected**: Should show status AND stay in input mode
8. Type: `/pwd` and press Enter
9. **Expected**: Should show current directory AND stay in input mode
10. Press ESC to exit input mode
11. **Expected**: Should return to normal mode
12. Press `q` to quit

### Test 2: Command Mode Persistence
1. Run: `./target/release/devkit interactive`
2. Press `:` to enter command mode
3. **Verify status bar says**: "COMMAND MODE: Type /commands and press Enter to submit | ESC: Exit to normal mode | Multiple commands allowed"
4. Type: `help` (without /) and press Enter
5. Type: `ls` and press Enter
6. Type: `pwd` and press Enter
7. **Expected**: All commands work and mode persists
8. Press ESC to return to normal mode
9. Press `q` to quit

### Test 3: File System Commands
**In input mode (`i` first):**
- `/ls` - Should list files in current directory
- `/ls src` - Should list files in src directory  
- `/pwd` - Should show current path
- `/cd src` - Should change to src directory
- `/pwd` - Should show new path
- `/cd ..` - Should go back up
- `/pwd` - Should show original path

### Test 4: System Commands  
**In input mode (`i` first):**
- `/status` - Should show agent system status
- `/agents` - Should list active agents
- `/help` - Should show comprehensive help
- `/clear` - Should clear the output panel

### Test 5: Interface Commands
**In input mode (`i` first):**
- `/theme dark` - Should switch to dark theme
- `/theme light` - Should switch to light theme  
- `/theme blue` - Should switch to blue theme
- `/theme` - Should show available themes

### Test 6: Natural Language (Fallback Mode)
**In input mode (`i` first):**
- `generate a function to sort numbers` - Should show fallback response
- `explain what this code does` - Should show fallback response
- `help me debug this error` - Should show fallback response

### Test 7: Session Commands
**In input mode (`i` first):**
- `/save test_session` - Should save current session
- `/sessions` - Should list sessions
- `/history` - Should show conversation history

### Test 8: Error Handling
**In input mode (`i` first):**
- `/invalid-command` - Should show error with suggestion
- `/cd /nonexistent` - Should show directory not found error
- `/ls /nonexistent` - Should show path error

### Test 9: Exit Commands
**In input mode (`i` first):**
- `/quit` - Should exit application
- `/exit` - Should exit application

## 🔍 What to Look For

### ✅ Success Indicators:
- Status bar shows correct persistent mode messages
- Commands execute and mode stays active
- ESC properly exits to normal mode
- All file system operations work
- Themes switch correctly
- Error messages are helpful and clear
- Application doesn't crash or hang

### ❌ Failure Indicators:
- Mode exits after each command (old behavior)
- Status bar shows wrong messages
- Commands don't execute properly
- Application crashes or hangs
- Escape doesn't work to exit modes
- File operations fail unexpectedly

## 📝 Test Results Template

```
✅ Persistent Input Mode: PASS/FAIL
✅ Persistent Command Mode: PASS/FAIL  
✅ File System Commands: PASS/FAIL
✅ System Commands: PASS/FAIL
✅ Interface Commands: PASS/FAIL
✅ Natural Language: PASS/FAIL
✅ Session Commands: PASS/FAIL
✅ Error Handling: PASS/FAIL
✅ Exit Commands: PASS/FAIL
```