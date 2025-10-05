#!/bin/bash

echo "🧪 Phase 1: Basic Functionality Tests"
echo "===================================="
echo ""

# Ensure we're in the devkit directory
cd /home/rga/devkit

echo "✅ Test 1: Application startup and help"
echo "Running: ./target/release/devkit interactive --help"
echo "Expected: Should show help for interactive command"
./target/release/devkit interactive --help
echo ""

echo "✅ Test 2: Basic startup and immediate quit"
echo "Running: echo 'q' | ./target/release/devkit interactive"
echo "Expected: Should start and quit cleanly"
echo 'q' | ./target/release/devkit interactive
echo ""

echo "✅ Test 3: Help command"
echo "Running: /help command"
echo "Expected: Should show comprehensive help text"
# This creates a more complex test that sends /help and then quits
(echo 'i'; echo '/help'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "✅ Test 4: Status command"
echo "Running: /status command"
echo "Expected: Should show system status"
(echo 'i'; echo '/status'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "✅ Test 5: PWD command"
echo "Running: /pwd command"
echo "Expected: Should show current directory"
(echo 'i'; echo '/pwd'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "✅ Test 6: List directory"
echo "Running: /ls command"
echo "Expected: Should list current directory contents"
(echo 'i'; echo '/ls'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "✅ Test 7: Clear command"
echo "Running: /clear command" 
echo "Expected: Should clear the screen"
(echo 'i'; echo '/clear'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "✅ Test 8: Invalid command handling"
echo "Running: /invalid-command"
echo "Expected: Should show error message with help suggestion"
(echo 'i'; echo '/invalid-command'; sleep 1; echo -e '\033'; echo 'q') | timeout 10 ./target/release/devkit interactive 2>/dev/null
echo ""

echo "🎯 Phase 1 Tests Complete!"
echo ""
echo "Manual tests still needed:"
echo "- Persistent input mode (enter 'i', type multiple commands)"
echo "- Status bar message verification"
echo "- Theme switching (/theme dark, /theme light, etc.)"
echo "- Visual UI rendering"
echo ""
echo "Run './target/release/devkit interactive' manually to test UI features"