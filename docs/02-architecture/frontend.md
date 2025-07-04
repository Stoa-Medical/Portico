# Frontend Architecture

Portico's UI is a conventional **SvelteKit 2.x** application located in `app/src/`.

## Tech choices

| Tool | Why |
| ---- | --- |
| SvelteKit + Vite | File-based routing, SSR, fast dev server |
| Tailwind CSS | Utility-first styling (matches team preference) |
| Vitest + Svelte Testing Library | Fast unit tests with good Svelte support |

## Directory guide

```txt
app/src/
  routes/
    +layout.svelte       # global shell (nav, theming)
    +layout.ts           # load common data (Supabase session)
    +page.svelte         # landing page
    agents/              # CRUD pages for Agents
    analytics/           # success & latency charts
    login/               # Auth flows
    register/            # Sign-up flows
  lib/
    components/          # Reusable UI widgets (PageHeader, Charts…)
    stores/              # Svelte stores (e.g. configStore.ts)
    supabase.ts          # Supabase client factory
    types.ts             # Shared TS types mirrored from Rust models
```

## Styling

Tailwind classes are authored inline with occasional `app.css` for global resets. No component libraries are introduced to keep bundle size small.

## State Management & Data Fetching

* Supabase client handles auth and CRUD—no GraphQL layer required.
* Lightweight Svelte stores expose current user and config.
* Pages use `load()` functions (`+page.ts`) for SSR-friendly data fetching.

## Testing

Unit tests live alongside components with the `.test.ts` suffix. Run all tests via:

```bash
pnpm test
```
