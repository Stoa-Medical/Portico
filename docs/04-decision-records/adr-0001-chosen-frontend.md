# ADR-0001: Initial MVP Design Decisions

Status: **Accepted**
Date: **2025-04-30**

---

## Context

Portico aims to be an *agentic integration engine* that lets developers connect data sources, run deterministic code or LLM steps, and collect results. The first milestone is an MVP that can be demo-ed and iterated on quickly.

## Decisions

1. **Supabase as Source of Truth**
   * Provides Postgres, Auth, Storage, and Realtime out-of-the-box.
   * Eliminates the need for a custom auth microservice.
2. **Rust for Runtime (`engine`)**
   * Strong type safety and performance.
   * Easy to ship a single static binary.
   * gRPC (`tonic`) selected for language-agnostic transport.
3. **Python Bridge for Supabase Realtime**
   * Mature async SDK while Rust ecosystem stabilises.
   * Keeps Supabase-specific code isolated so we can rewrite later.
4. **Agent / Step / Signal / RuntimeSession Data Model**
   * Event-driven core – Signals trigger work.
   * Agents own Steps; Steps are deterministic (code) or nondeterministic (LLM).
5. **Thread-Pool Concurrency in Engine**
   * Each Agent has its own FIFO queue feeding a shared pool; good CPU utilisation, predictable ordering.
6. **Tauri Desktop App with SvelteKit UI**
   * Allows offline demos and on-prem installs.
   * Reuses the same SvelteKit codebase for both web and desktop builds.
7. **No Horizontal Scaling for MVP**
   * Simplifies networking, configuration, and the DB schema.
   * Focus remains on delivering core value quickly.

## Consequences

* Quick path to a functioning prototype for stakeholder demos.
* Some duplication between Rust and Python models (managed through shared protobuf definitions).
* The Bridge is a potential bottleneck but can be rewritten in Rust once SDKs mature.
* Supabase lock-in is acceptable for MVP; abstraction layers can be added later if needed.

## Alternatives Considered

1. **Pure Python Backend**
   * Simpler development but concerns about performance and type safety
   * Would require PyO3 for desktop app integration anyway

2. **GraphQL instead of gRPC**
   * More frontend-friendly but adds complexity for streaming
   * gRPC chosen for efficiency and strong typing

3. **Electron instead of Tauri**
   * Larger bundle size (100MB+ vs 10MB)
   * Tauri provides better OS integration and security

4. **Custom Auth instead of Supabase**
   * Would delay MVP by weeks
   * Supabase auth is battle-tested and feature-complete
