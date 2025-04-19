# Cursor MDC Rules

This directory contains custom Cursor MDC (Metalinter Domain Configuration) rules for the Portico codebase. These rules help enforce coding standards and best practices across different parts of the project.

## Rule Files

- **app.mdc**: Rules for the Tauri 2.0 app with Svelte 5 and SvelteKit in the `/app` directory
- **python_bridge.mdc**: Rules for the Python code in the `/server/bridge` directory
- **rust_engine.mdc**: Rules for the Rust code in the `/server/engine` directory
- **shared_rust.mdc**: Rules for the shared Rust code in the `/shared` directory
- **general.mdc**: General rules applicable to the entire codebase

## Severity Levels

- **error**: Critical issues that should be fixed immediately
- **warning**: Potential problems that should be addressed
- **info**: Suggestions for code improvements

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
