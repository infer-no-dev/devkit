# DevKit CLI Reference (Source of Truth)

This document is generated from the CLI's built-in help texts to ensure it always matches the implementation.

## devkit (top-level)

```text
🤖 Agentic Development Environment

An intelligent, multi-agent development environment built for AI-assisted 
code generation on large existing codebases. The system leverages multiple 
concurrent AI agents, advanced code analysis, and natural language programming.

Features:
• Multi-agent coordination for complex tasks
• Advanced code analysis with semantic understanding
• Context-aware code generation
• Cross-platform shell integration
• Rich terminal-based interface
• Comprehensive configuration management


Usage: devkit [OPTIONS] <COMMAND>

Commands:
  init         Initialize a new agentic development project
  interactive  Start the interactive development mode
  analyze      Analyze codebase and generate context
  generate     Generate code or scaffold a project from a prompt
  agent        Manage AI agents
  config       Configuration management
  inspect      Context and symbol inspection
  profile      Performance profiling and diagnostics
  template     Template management
  status       Project status and health check
  shell        Shell integration and completion
  demo         Run end-to-end demo workflow
  blueprint    System blueprint operations
  plugin       Plugin marketplace operations
  chat         AI-powered project manager agent
  session      Session management (list, create, switch, branch, analytics)
  visualize    Open coordination visualizer
  dashboard    View system dashboard
  analytics    Generate analytics reports
  monitor      Monitor agent performance and system metrics
  export       Export session data and reports
  behavior     Agent behavior customization
  diagnose     Run project diagnostics
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Enable verbose output

  -q, --quiet
          Quiet mode - minimal output

  -c, --config <CONFIG>
          Configuration file path

  -C, --directory <DIRECTORY>
          Working directory

      --format <FORMAT>
          Output format (text, json, yaml)
          
          [default: text]
          [possible values: text, json, yaml, table]

      --color <COLOR>
          Enable colored output
          
          [default: auto]

      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)

      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator)
          
          [possible values: true, false]

      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)

      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)

      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)

      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)

      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Subcommands

### devkit init
```text
Initialize a new agentic development project

Usage: devkit init [OPTIONS] <NAME>

Arguments:
  <NAME>  Project directory name

Options:
  -t, --template <TEMPLATE>    Project template to use
  -l, --language <LANGUAGE>    Programming language
      --no-interactive         Skip interactive prompts
      --force                  Force overwrite existing directory
      --git                    Initialize git repository
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit interactive
```text
Start the interactive development mode

Usage: devkit interactive [OPTIONS]

Options:
  -V, --view <VIEW>            Start with specific view (agents, context, config, logs)
      --auto-start             Auto-start agents
  -m, --monitor                Monitor mode (read-only)
  -w, --web                    Enable web dashboard
      --web-port <WEB_PORT>    Web dashboard port (default: 8080)
      --web-host <WEB_HOST>    Web dashboard host (default: 127.0.0.1)
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit analyze
```text
Analyze codebase and generate context

Usage: devkit analyze [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...  Target files or directories

Options:
  -d, --depth <DEPTH>
          Analysis depth (shallow, normal, deep) [default: normal]
      --include-tests
          Include test files
  -e, --export <EXPORT>
          Export results to file
      --analysis-types <ANALYSIS_TYPES>
          Specific analysis types (symbols, dependencies, architecture, quality)
  -p, --progress
          Show progress
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
  -h, --help
          Print help
```

### devkit generate
```text
Generate code or scaffold a multi-file project from a natural language prompt.

Examples:
  devkit generate "todo api" --language rust --stack rust-axum --root ./api
  devkit generate "marketing site" --language typescript --stack nextjs --root ./web --dry-run
  devkit generate --list-stacks
  devkit generate --apply-plan plan.json --force

Flags:
  --stack, --dry-run, --force, --no-scaffold, --single-file, --root, --export-plan, --apply-plan, --list-stacks

Usage: devkit generate [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>
          Natural language prompt

Options:
  -o, --output <OUTPUT>
          Target file or directory (if directory and scaffolding is enabled, a project is created here)

  -l, --language <LANGUAGE>
          Programming language

      --context <CONTEXT>
          Include context files

      --strategy <STRATEGY>
          Generation strategy (focused, comprehensive, iterative)
          
          [default: focused]

      --max-tokens <MAX_TOKENS>
          Maximum tokens to generate

      --temperature <TEMPERATURE>
          Temperature for generation (0.0-1.0)

  -p, --preview
          Preview mode (don't write files)

      --scaffold
          Enable automatic project scaffolding (multi-file). Disabled if --single-file or --no-scaffold is set

      --no-scaffold
          Disable scaffolding (alias for --scaffold=false)

      --single-file
          Force single-file output (disables scaffolding)

      --root <ROOT>
          Root directory to scaffold into (overrides detection from --output)

      --stack <STACK>
          Stack preset (e.g. rust-axum, rust-actix, rust-axum-sqlx, node-express, node-nest, nextjs, python-fastapi, python-fastapi-sqlalchemy)

      --dry-run
          Dry run scaffolding (print plan, do not write)

      --force
          Overwrite existing files/directories during scaffolding

      --list-stacks
          List available --stack presets and exit

      --export-plan <EXPORT_PLAN>
          Export planned file map to JSON (planning only)

  -v, --verbose
          Enable verbose output

      --apply-plan <APPLY_PLAN>
          Apply a previously exported plan JSON instead of generating

  -q, --quiet
          Quiet mode - minimal output

  -c, --config <CONFIG>
          Configuration file path

  -C, --directory <DIRECTORY>
          Working directory

      --format <FORMAT>
          Output format (text, json, yaml)
          
          [default: text]
          [possible values: text, json, yaml, table]

      --color <COLOR>
          Enable colored output
          
          [default: auto]

  -h, --help
          Print help (see a summary with '-h')
```

### devkit agent
```text
Manage AI agents

Usage: devkit agent [OPTIONS] <COMMAND>

Commands:
  list         List available agents
  status       Show agent status
  start        Start specific agents
  stop         Stop specific agents
  create       Create custom agent
  remove       Remove custom agent
  logs         Show agent logs
  cancel-task  Cancel a running or queued task by ID
  resume       Resume pending/running tasks from snapshots
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit config
```text
Configuration management

Usage: devkit config [OPTIONS] <COMMAND>

Commands:
  show          Show current configuration
  set           Set configuration value
  get           Get configuration value
  validate      Validate configuration
  environment   Switch environment
  environments  List available environments
  edit          Edit configuration interactively
  reset         Reset to defaults
  export        Export configuration
  import        Import configuration
  help          Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit inspect
```text
Context and symbol inspection

Usage: devkit inspect [OPTIONS] <COMMAND>

Commands:
  symbols        Inspect symbols in codebase
  file           Show file context information
  dependencies   Analyze dependencies
  relationships  Show code relationships
  quality        Code quality metrics
  help           Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit profile
```text
Performance profiling and diagnostics

Usage: devkit profile [OPTIONS] <COMMAND>

Commands:
  system       Profile system performance
  agents       Profile agent performance
  context      Profile context analysis
  diagnostics  Show system diagnostics
  memory       Memory usage analysis
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit template
```text
Template management

Usage: devkit template [OPTIONS] <COMMAND>

Commands:
  list    List available templates
  show    Show template details
  apply   Apply a template with variables
  create  Create new template
  remove  Remove template
  update  Update template
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit status
```text
Project status and health check

Usage: devkit status [OPTIONS]

Options:
  -d, --detailed
          Show detailed status
      --components <COMPONENTS>
          Check specific components
  -p, --performance
          Include performance metrics
      --external
          Check external dependencies
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit shell
```text
Shell integration and completion

Usage: devkit shell [OPTIONS] <COMMAND>

Commands:
  completion  Generate shell completion scripts
  install     Install shell integration
  status      Show shell integration status
  help        Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit demo
```text
Run end-to-end demo workflow

Usage: devkit demo [OPTIONS]

Options:
  -s, --step <STEP>            Run specific demo step (analyze, generate, interactive, all)
      --yes                    Skip confirmation prompts
      --cleanup                Clean up demo artifacts after completion
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit blueprint
```text
System blueprint operations

Usage: devkit blueprint [OPTIONS] <COMMAND>

Commands:
  extract    Extract system blueprint from codebase
  generate   Generate project from blueprint
  replicate  Replicate current system
  validate   Validate blueprint file
  info       Show blueprint information
  compare    Compare blueprints
  evolution  Blueprint evolution and versioning
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit plugin
```text
Plugin marketplace operations

Usage: devkit plugin [OPTIONS] <COMMAND>

Commands:
  search   Search for plugins in the marketplace
  info     Show detailed information about a plugin
  install  Install a plugin from the marketplace
  list     List installed plugins
  update   Update plugins
  status   Show plugin system status
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -c, --config <CONFIG>        Configuration file path
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit chat
```text
AI-powered project manager agent

Usage: devkit chat [OPTIONS]

Options:
  -p, --project <PROJECT>      Project root directory
  -c, --config <CONFIG>        Configuration file
      --debug                  Enable debug output
  -m, --message <MESSAGE>      Initial message or question to start the conversation
      --persist                Keep conversation history persistent across sessions
      --resume                 Continue from previous conversation session
      --onboarding             Show onboarding greeting (disable with --no-onboarding)
      --max-turns <MAX_TURNS>  Maximum number of conversation turns [default: 50]
  -v, --verbose                Enable verbose output
  -q, --quiet                  Quiet mode - minimal output
  -C, --directory <DIRECTORY>  Working directory
      --format <FORMAT>        Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>          Enable colored output [default: auto]
  -h, --help                   Print help
```

### devkit session
```text
Session management (list, create, switch, branch, analytics)

Usage: devkit session [OPTIONS] <COMMAND>

Commands:
  list       List all sessions
  create     Create a new session
  switch     Switch to a session
  branch     Create session branch for experimentation
  analytics  View session analytics
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit visualize
```text
Open coordination visualizer

Usage: devkit visualize [OPTIONS]

Options:
  -V, --view <VIEW>
          Visualization type (network, timeline, resource, overview) [default: network]
      --refresh <REFRESH>
          Auto-refresh interval in seconds [default: 5]
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit dashboard
```text
View system dashboard

Usage: devkit dashboard [OPTIONS]

Options:
  -p, --port <PORT>
          Dashboard port [default: 8080]
      --host <HOST>
          Dashboard host [default: localhost]
      --open
          Open browser automatically
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit analytics
```text
Generate analytics reports

Usage: devkit analytics [OPTIONS] <COMMAND>

Commands:
  report  Generate analytics report
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit monitor
```text
Monitor agent performance and system metrics

Usage: devkit monitor [OPTIONS]

Options:
  -t, --target <TARGET>
          Monitor target (agents, system, performance) [default: agents]
      --real-time
          Real-time monitoring
      --interval <INTERVAL>
          Refresh interval in seconds [default: 2]
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit export
```text
Export session data and reports

Usage: devkit export [OPTIONS]

Options:
  -s, --session <SESSION>
          Session name to export
      --format <FORMAT>
          Export format [default: json] [possible values: text, json, yaml, table]
  -o, --output <OUTPUT>
          Output file
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit behavior
```text
Agent behavior customization

Usage: devkit behavior [OPTIONS] <COMMAND>

Commands:
  edit    Open behavior editor
  load    Load behavior profile
  create  Create custom behavior profile
  list    List available profiles
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Enable verbose output
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```

### devkit diagnose
```text
Run project diagnostics

Usage: devkit diagnose [OPTIONS]

Options:
      --check <CHECK>
          Run specific diagnostic (config, agents, context, all) [default: all]
      --fix
          Fix issues automatically where possible
  -v, --verbose
          Show detailed diagnostic information
  -q, --quiet
          Quiet mode - minimal output
  -c, --config <CONFIG>
          Configuration file path
  -C, --directory <DIRECTORY>
          Working directory
      --format <FORMAT>
          Output format (text, json, yaml) [default: text] [possible values: text, json, yaml, table]
      --color <COLOR>
          Enable colored output [default: auto]
      --orchestrator-task-timeout-seconds <ORCHESTRATOR_TASK_TIMEOUT_SECONDS>
          Task timeout in seconds (orchestrator)
      --orchestrator-retry-failed-tasks <ORCHESTRATOR_RETRY_FAILED_TASKS>
          Enable retries (orchestrator) [possible values: true, false]
      --orchestrator-max-retry-attempts <ORCHESTRATOR_MAX_RETRY_ATTEMPTS>
          Max retry attempts (orchestrator)
      --orchestrator-backoff <ORCHESTRATOR_BACKOFF>
          Backoff strategy: fixed|exponential (orchestrator)
      --orchestrator-backoff-base-secs <ORCHESTRATOR_BACKOFF_BASE_SECS>
          Base/backoff seconds (orchestrator)
      --orchestrator-backoff-factor <ORCHESTRATOR_BACKOFF_FACTOR>
          Backoff factor (orchestrator, exponential only)
      --orchestrator-backoff-max-secs <ORCHESTRATOR_BACKOFF_MAX_SECS>
          Backoff max seconds (orchestrator, exponential only)
  -h, --help
          Print help
```
