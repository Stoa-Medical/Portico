# Desktop Shell (Tauri)

The desktop application wraps the SvelteKit UI in a Tauri window, giving us:
* Single distributable binary for macOS / Windows / Linux.
* Access to OS APIs (file-dialogs, system browser, future auto-updates).
* Optional **offline/demo** execution by bundling the Engine binary locally.

## Folder Layout

```txt
app/
  src-tauri/          # Rust side of the desktop app
    src/
      lib.rs          # Tauri commands & plugin setup
      main.rs         # Entrypoint – calls `portico_app_lib::run()`
```

## Important pieces

* **`greet` command** in `lib.rs` shows the canonical pattern for exposing Rust code to the Svelte front-end.
* **`tauri_plugin_opener`** is bundled to ensure links opened in the UI launch the OS browser instead of navigating the app window.
* The Tauri context is configured by `tauri.conf.json`; here you enable permissions, icons, and build targets.

## Permissions & Security

Tauri 2 introduces a granular permission system. We currently require:

| Permission | Why |
| ---------- | --- |
| `http`     | Call Supabase REST endpoints & optional local Engine |
| `shell`    | Open external URLs via system browser |

If we embed a local Engine for demo mode we will additionally request `process` to spawn the binary.

## Packaging & Release

1. `pnpm tauri build` – creates platform-native bundles.
2. Sign the artefacts (macOS notarisation, Windows code-signing).
3. Attach to a GitHub Release; auto-update can be configured later via `tauri://updater`.
