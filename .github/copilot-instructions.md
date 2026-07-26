# GitHub Copilot Instructions

Read [../AGENTS.md](../AGENTS.md) for the canonical project guidance.

Tinkflow is a Tauri 2 desktop dictation app with a React frontend and Rust backend. Keep IPC names stable between `tinkflow/src-tauri/src/lib.rs` and frontend `invoke(...)` calls. Preserve local-first behavior and avoid adding cloud services without an explicit product decision.
