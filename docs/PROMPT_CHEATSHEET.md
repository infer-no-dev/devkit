# DevKit Prompt Cheatsheet

Use this one-pager to draft strong prompts quickly.

- Mini checklist
  - Goal/output: what should exist after generation?
  - Stack/language: e.g., Rust Axum, Node/Express, Next.js, Go Gin
  - Interfaces: routes/API, CLI flags, pages/components, I/O formats
  - Data model: entities, fields, relations, validations
  - Constraints: style, error handling, tests, perf, security
  - Deploy/env: Docker, .env keys, ports, prod/dev differences
  - Output path: where files go

- Seed patterns
  - CRUD API: "Build a {stack} REST API for {entity} with fields {fields}, CRUD endpoints, pagination, validation, auth {type}, DB {db}, tests, Docker."
  - Web app: "Create a {framework} app with pages {list}, global layout, state via {store}, UI {lib}, form validation, SSR/CSR, and basic tests."
  - CLI tool: "Generate a {lang} CLI '{name}' with subcommands {subs}, flags {flags}, rich help, logging, config file, and unit tests."
  - Worker/service: "Background worker in {lang} that consumes {queue}, retries, idempotency, metrics, structured logs, graceful shutdown."
  - Auth: "Add {auth} to {stack}: signup/login, session/JWT, password reset, role-based guards, CSRF, rate-limits, tests."
  - DB + migrations: "Set up {ORM} with entities {entities}, relations, migrations, seed scripts, and repository layer."
  - Testing: "Write unit/integration tests for {module}: cases {list}, fixtures, mocks/stubs, coverage target {pct}."
  - Refactor: "Refactor {path}: extract {functions}, improve error handling, add docs, simplify control flow, keep behavior identical."
  - Docs: "Generate README and docs for {project}: setup, run, test, deploy, config, architecture diagram, API docs."

- Strong prompt template
"""
Generate a {lang} {app_type}.
Requirements:
- Stack: {details}
- Features: {bullets}
- Data model: {entities}
- Interfaces: {routes/pages/CLI}
- Quality: tests, lint, error handling, logging
- Output: write to {path}; keep file structure idiomatic.
Return only code/files; no extraneous text.
"""

- DevKit commands
- Analyze context: `devkit analyze . --progress`
- Generate code: `devkit generate "…" --language {lang} --output {path} --preview`
- Templates:
  - List: `devkit template list --language rust`
  - Show: `devkit template show rust_function`
  - Apply (stdout): `devkit template apply rust_function -v name=do_work -v return_type=Result<()> -v parameters="path: &str" -v body="println!(\"{}\", path); Ok(())"`
  - Apply to file: `devkit template apply rust_function -v name=do_work -v body="…" --output src/utils.rs --force`
- Chat: `devkit chat --project .`

- Tips
- Be concrete: explicit routes, schema, filenames
- Include tests and error paths
- Add small example inputs/outputs
- Use `--preview` first; then save
- Iterate: follow up with focused refine prompts

## CLI quoting tips (bash)
- Safest: single-quoted heredoc so nothing needs escaping
  ```
  ./target/release/devkit generate "$(cat <<'EOF'
  Build a production-ready, responsive site for "Acme Home & IT" …
  (paste your full prompt here; quotes, $vars, & symbols are safe)
  EOF
  )" --language typescript --stack nextjs --root ./site --dry-run
  ```
- Put long specs in a file and pass as context
  ```
  echo "Full spec…" > spec.md
  ./target/release/devkit generate "Build the site per spec" \
    --language typescript --stack nextjs --root ./site --context spec.md
  ```
- Inline single quotes? Escape as: 'Acme'\''s Bakery'
- Avoid smart quotes; prefer ASCII ' and ".
- Single-quoted heredoc ('EOF') prevents $VAR expansion and backslash escapes.
