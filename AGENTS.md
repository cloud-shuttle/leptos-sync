# AGENTS.md - Leptos-Sync Development Guide

## Commands
- **Build**: `make build` / `cargo build --all` / `pnpm build`
- **Test**: `make test` (all Rust tests) / `make test-e2e` (Playwright) / `cargo test --all` / `pnpm test`
- **Single test**: `cargo test --bin <specific_test>` / `cargo test <test_name>` / `playwright test <test_file>`
- **WASM tests**: `make test-wasm` (Node) / `make test-browser` (Chrome headless)
- **Check/Lint**: `make check` (clippy + fmt check) / `cargo clippy --all -- -D warnings` / `cargo fmt --all -- --check`
- **Dev**: `cargo leptos watch` / `pnpm dev` (Leptos development server)
- **Database**: `pnpm db:up` (start containers) / `pnpm db:reset` (clean restart)

## Architecture
- **Workspace**: Multi-crate Rust workspace with Leptos frontend integration
- **Core crates**: `leptos-sync-core` (main library), `leptos-sync-components` (UI), `leptos-sync-macros` (proc-macros)
- **Examples**: Multiple demo apps (text editor, task manager, document editor, project manager)
- **Storage**: Hybrid storage (OPFS → IndexedDB → LocalStorage) with automatic fallback
- **Transports**: WebSocket (leptos-ws-pro), InMemory, Multi-transport with failover
- **CRDTs**: LWW-Register, GCounter, RGA, LSEQ, Yjs Tree, DAG structures

## Code Style
- **Edition**: Rust 2024, Leptos 0.8.x
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Imports**: Group std, external crates, local crates; use explicit paths for public APIs
- **Error handling**: `thiserror` for custom errors, `Result<T, E>` return types, propagate with `?`
- **Async**: `async-trait` for traits, `tokio` runtime, `wasm-bindgen-futures` for WASM
- **Components**: Use `#[component]` macro, PascalCase component names, props structs with `Props` suffix
