#!/bin/bash

echo "🧪 Testing DevKit Interactive Mode UI"
echo ""

# Test basic startup
echo "1. Testing basic startup (10 second timeout)..."
timeout 10s ./target/release/devkit interactive &
UI_PID=$!

# Give it a few seconds to start
sleep 3

if ps -p $UI_PID > /dev/null; then
    echo "   ✅ UI started successfully"
    # Kill it gracefully
    kill $UI_PID 2>/dev/null
    sleep 1
    # Force kill if still running
    kill -9 $UI_PID 2>/dev/null
else
    echo "   ❌ UI failed to start or exited early"
fi

echo ""
echo "2. Testing with specific terminal settings..."
TERM=xterm-256color timeout 10s ./target/release/devkit interactive &
UI_PID2=$!

sleep 3

if ps -p $UI_PID2 > /dev/null; then
    echo "   ✅ UI started with TERM=xterm-256color"
    kill $UI_PID2 2>/dev/null
    sleep 1
    kill -9 $UI_PID2 2>/dev/null
else
    echo "   ❌ UI failed to start with TERM setting"
fi

echo ""
echo "💡 If both tests passed, the UI should work!"
echo "💡 To test scrolling manually:"
echo "   1. Run: ./target/release/devkit interactive"
echo "   2. Wait for UI to appear"
echo "   3. Press 'i' to enter input mode"
echo "   4. Type /help and press Enter (adds content)"
echo "   5. Press Escape to return to normal mode"  
echo "   6. Use arrow keys ↑↓ or j/k to scroll"
echo "   7. Press '?' for complete help"
echo "   8. Press 'q' to quit"