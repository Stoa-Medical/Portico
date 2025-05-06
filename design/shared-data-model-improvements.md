# Improving Shared Rust Data Models for Readability

This document captures recommendations to simplify and clarify the `shared` crate’s data models, prioritizing developer onboarding and reducing boilerplate.

## 1. Derive-based Enums
- Use `strum` and `sqlx::Type` derives to replace manual `as_str`, `FromStr`, and SQLx encode/decode.
- Example:
  ```rust
  #[derive(EnumString, AsRefStr, Display, Type, Serialize, Deserialize)]
  #[strum(serialize_all = "lowercase")]
  #[sqlx(type_name = "running_status", rename_all = "lowercase")]
  pub enum RunningStatus { Waiting, Running, Completed, Cancelled }
  ```

## 2. New-type IDs Instead of `IdFields<I>`
- Define `LocalId(i32)` and `GlobalId(Uuid)` structs with their own derives.
- Eliminates heavy trait bounds on generics and improves type safety.

## 3. Builders via `typed-builder`
- Replace multi-arg `new()` constructors with a fluent builder:
  ```rust
  #[derive(TypedBuilder)]
  pub struct Agent { /* fields */ }
  let agent = Agent::builder().identifiers(ids).description("foo").build();
  ```

## 4. Timestamp Macro
- Create or adopt a `#[derive(Timestamps)]` macro for `created`/`updated` fields, `Default`, and `update()`.

## 5. Reduce `pub use` Re-exports
- Prefer `models::signals::Signal` over flat re-exports to keep modules discoverable.

## 6. Structured Error Handling
- Use `thiserror` to define domain-specific error enums, returning `Result<T, MyError>` in pure libraries and converting to `anyhow::Error` at app boundaries.

## 7. SQLx Compile-time Query Validation
- Use `sqlx::query_file!` or `query!` macros to catch SQL errors at build time instead of `format!` strings.

## 8. Separate Models from Services
- Move side-effect code (PythonRuntime, LLM, webscrape) into a `services/` module, leaving `lib.rs` focused on pure data models.

## 9. Rustdoc Examples
- Add `/// ```no_run` examples for each public struct/enum to demonstrate common usage.

## 10. Cargo Features & Lints
- Define feature flags for derive-only crates (`strum`, `typed-builder`, `thiserror`).
- Add:
  ```toml
  [package.metadata.cargo-udeps.ignore]
  ```
- Enforce `#![deny(clippy::all, rust_2018_idioms)]` in the crate root.

## 11. (Optional) SQLx Enum Codegen
- Leverage `sqlx-cli` to autogenerate Rust enums from Postgres types.

## 12. Onboarding & README
- Turn `shared/README.txt` into `shared/README.md` with:
  - A diagram of model relationships
  - First-PR checklist (e.g. `cargo test`, `sqlx migrate run`)
  - Table mapping DB tables ↔ Rust structs
