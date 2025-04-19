# Cursor MDC Rules

This directory contains custom Cursor MDC (Metalinter Domain Configuration) rules for the Portico codebase. These rules help enforce coding standards and best practices across different parts of the project.

## Rule Files

### Language-Specific Rules (/languages)

- **svelte.mdc**: Svelte 5 specific language rules for UI components
- **rust.mdc**: Rust language rules for all Rust code in the project
- **python.mdc**: Python language rules for all Python code in the project

### Framework-Specific Rules (/frameworks)

- **tauri.mdc**: Tauri 2.0 specific rules for the desktop app components

### Repo-Specific Rules
- **python_bridge.mdc**: Rules for the Python bridge component in `/server/bridge`
- **rust_engine.mdc**: Rules for the Rust engine component in `/server/engine`
- **shared_rust.mdc**: Rules for shared Rust code in `/shared`

### General Rules

- **general.mdc**: Rules applicable to all files in the codebase

## Severity Levels

- **error**: Critical issues that should be fixed immediately
- **warning**: Potential problems that should be addressed
- **info**: Suggestions for code improvements

## Rule Configuration

Each rule file contains:
- **description**: What the ruleset covers
- **globs**: File patterns the rules apply to
- **alwaysApply**: Whether to always run these rules or only on matched files

## Adding New Rules

To add a new rule, follow this format:

```
rule "Rule name" {
  pattern = /regex_pattern/
  not_pattern = /exception_pattern/  // Optional
  message = "Message to display when rule is triggered"
  severity = "warning"  // Can be error, warning, or info
}
```

You can also use other rule properties like:
- `filesize_max`: Maximum file size in lines
- `line_length_max`: Maximum line length in characters
- `scope`: Where to apply the rule (e.g., "import")

## Documentation

For more information on Cursor MDC rules and their syntax, refer to the [Cursor documentation](https://cursor.sh/docs/metalinter).
