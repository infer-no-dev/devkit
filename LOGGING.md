# Logging Configuration

DevKit uses structured logging with different levels to provide appropriate information without overwhelming users.

## Default Behavior

By default, DevKit runs with **INFO** level logging, showing:
- ℹ️ Informational messages (startup, completion, etc.)
- ⚠️ Warnings
- ❌ Errors
- ✅ Success messages

## Customizing Log Level

You can control the logging level using the `RUST_LOG` environment variable:

### For Regular Use (Recommended)
```bash
# Default: INFO level (recommended for normal use)
devkit interactive

# Or explicitly set:
RUST_LOG=info devkit interactive
```

### For Troubleshooting
```bash
# DEBUG level - shows detailed internal operations
RUST_LOG=debug devkit interactive

# TRACE level - shows all internal details (very verbose)
RUST_LOG=trace devkit interactive
```

### For Quiet Operation
```bash
# WARN level - only shows warnings and errors
RUST_LOG=warn devkit interactive

# ERROR level - only shows errors
RUST_LOG=error devkit interactive
```

## Module-Specific Logging

You can also control logging for specific modules:

```bash
# Debug only for the UI module
RUST_LOG=devkit::ui=debug,info devkit interactive

# Debug for interactive commands, info for everything else
RUST_LOG=devkit::cli::commands::interactive=debug,info devkit interactive
```

## Interactive Mode

Interactive mode has been optimized to reduce log spam while still providing useful feedback. The following debug information is now properly controlled by log levels:

- Command processing (DEBUG level)
- UI event handling (DEBUG level)  
- Agent system operations (DEBUG level)

## CLI Verbose Mode

You can also use the CLI's built-in verbose flag:

```bash
# Verbose output (shows additional context)
devkit --verbose interactive

# Quiet mode (minimal output)
devkit --quiet interactive
```

## Examples

```bash
# Normal usage - clean, informative output
devkit interactive

# Troubleshooting plugin issues
RUST_LOG=devkit::plugins=debug devkit plugin install some-plugin

# Full debug mode for development
RUST_LOG=debug devkit interactive

# Quiet background operation
RUST_LOG=warn devkit --quiet analyze /path/to/code
```