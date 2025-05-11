Portico Desktop App.

Portico is a desktop application built with SvelteKit (v2.9.0) + Tauri 2.0 using Svelte 5.

It has two modes: 1) connect to a remote server for management, or 2) run standalone with an embedded SQLite database.

Stack:
- Tauri 2.0
    - SvelteKit frontend (Svelte 5)
    - Rust backend
- Vite + TypeScript
- Tailwind CSS + Flowbite-Svelte
- CodeMirror Python editor
- Supabase JS for remote data

Requirements:
1. Node.js 18+ and pnpm
2. Rust stable toolchain
3. Python 3.10+ for embedded interpreter
4. SQLite CLI for standalone mode
5. SUPABASE_URL and SUPABASE_ANON_KEY in .env

Setup:
1. pnpm install
2. cp .env-example .env
3. edit .env with your Supabase credentials

Development:
1. pnpm dev
2. pnpm tauri dev

Build:
1. pnpm build
2. pnpm tauri build

Testing:
pnpm test

Configuration:
- See src/lib/stores/configStore.ts for settings management
- See src/lib/components/AdminSettings.svelte for admin UI
