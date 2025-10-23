# DevKit Interactive Mode Scrolling Test

## How to Test Scrolling

1. **Start DevKit Interactive Mode:**
   ```bash
   ./target/release/devkit interactive
   ```

2. **Wait for UI to load** (you should see panels with agent status, notifications, etc.)

3. **Generate some content to scroll through:**
   - Press `i` to enter input mode
   - Type `/help` and press Enter (this adds content)
   - Type `/status` and press Enter (more content)
   - Type `/ls` and press Enter (even more content)
   - Press `Esc` to return to Normal mode

4. **Test scrolling (make sure you're in Normal mode):**
   - **↑/↓ Arrow Keys** - Should scroll up/down line by line
   - **k/j** - Should scroll up/down line by line (vim-style)
   - **PageUp/PageDown** - Should scroll by page
   - **Home** or **g** - Should jump to top
   - **End** or **G** - Should jump to bottom
   - **a** - Toggle auto-scroll on/off

5. **Check Visual Feedback:**
   - Look for scroll position in output panel title: `Output (3) (25/150)`
   - Look for scroll mode indicator: `[AUTO]` or `[MANUAL]`
   - Scrollbar should appear on right side when there's more content

6. **Get Help:**
   - Press **?** or **F1** to see complete key binding help

## If TUI Hangs on Startup

If the TUI doesn't start properly, it might be a terminal compatibility issue. Try:

```bash
# Check if it's a raw mode issue
TERM=xterm-256color ./target/release/devkit interactive

# Or try with different terminal
# Sometimes Manjaro/Arch terminals need specific settings
```

## Expected Behavior

- **Default**: Auto-scroll enabled (new content appears at bottom)
- **Manual scrolling**: Temporarily disables auto-scroll
- **Return to bottom**: Re-enables auto-scroll
- **All scrolling**: Should work smoothly with immediate visual feedback

## Troubleshooting

If scrolling doesn't work:
1. Make sure you're in **Normal mode** (not Input mode)
2. Make sure there's enough content to scroll through
3. Try the vim-style keys: **j** (down), **k** (up)
4. Press **?** to see if help overlay works