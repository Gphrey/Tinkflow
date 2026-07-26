# Agent Collaboration

This document lets different AI models work on Tinkflow without stepping on each other. `AGENTS.md` is the entry point; this file is the deeper operating protocol.

## Shared Context Contract

Every agent should leave the repo easier for the next agent to understand.

- Start by checking `git status --short`.
- Read `AGENTS.md`, this file, and `docs/ARCHITECTURE.md`.
- Use `graphify query` for broad architecture questions when `graphify-out/graph.json` exists.
- Keep edits scoped to the task. Do not reformat unrelated files.
- Add or update docs when you introduce a new module, command, setting, event, or pipeline state.
- In the final handoff, report changed files, checks run, and checks skipped.

## Model-Specific Entry Points

- Codex/OpenAI agents: `AGENTS.md`
- Claude agents: `CLAUDE.md`
- Gemini agents: `GEMINI.md`
- GitHub Copilot: `.github/copilot-instructions.md`

These files should stay thin and point back to `AGENTS.md`. Avoid maintaining multiple competing instruction sets.

## Collaboration Workflow

1. Orient
   - Run `git status --short`.
   - If the question is broad, run `graphify query "..."`.
   - Read the relevant source files directly before editing.

2. Plan
   - Identify the boundary: frontend UI, Tauri IPC, Rust pipeline, packaging, or docs.
   - Name any OS-dependent behavior that cannot be fully tested in a headless shell.

3. Change
   - Keep IPC changes paired: Rust command registration and frontend `invoke` calls must move together.
   - Keep settings changes paired: `AppSettings` in Rust, frontend interfaces, defaults, and any persisted migration behavior.
   - Keep recording state changes paired: Rust `emit_recording_state`, `RecordingState` union, `StatusIndicator`, and `Monitor`.

4. Verify
   - Run `npm run build` for frontend/type changes.
   - Run `cargo check` for Rust changes.
   - Run `cargo test` when touching modules with tests or logic that can be unit-tested.
   - For hotkey/audio/overlay/injection changes, add manual verification notes.

5. Handoff
   - Summarize intent and files changed.
   - List verification commands and results.
   - Note any graph refresh: `graphify . --code-only --no-viz`.

## Ownership Map

Use this map to route work to the right files.

| Concern | Primary files | Notes |
| --- | --- | --- |
| App startup and Tauri command registration | `tinkflow/src-tauri/src/lib.rs`, `main.rs` | Main managed state setup and command surface. |
| Recording orchestration | `tinkflow/src-tauri/src/hotkey.rs` | Starts/stops capture, calls Whisper, dictionary, Ollama, injector, and emits UI states. |
| Audio capture | `tinkflow/src-tauri/src/audio.rs` | CPAL input, mono conversion, VAD, resampling to 16 kHz. |
| Whisper models and transcription | `tinkflow/src-tauri/src/whisper.rs` | Model path, download, cancellation, loading, inference. |
| LLM polishing | `tinkflow/src-tauri/src/llm.rs` | Local Ollama only; prompt and output sanitizer are intentionally model-agnostic. |
| Developer speech corrections | `tinkflow/src-tauri/src/dictionary.rs` | Aho-Corasick replacements before LLM polishing. |
| Context detection | `tinkflow/src-tauri/src/context.rs` | Foreground window title to context label. |
| Text insertion | `tinkflow/src-tauri/src/injector.rs` | Enigo text injection into active app. |
| Recording state UI sync | `tinkflow/src/hooks/useRecording.ts`, `StatusIndicator.tsx`, `Monitor.tsx` | Event listener plus polling fallback for overlay reliability. |
| Settings UI | `SettingsView.tsx`, `Dashboard.tsx`, `Onboarding.tsx` | Frontend state mirrors Rust `AppSettings`. |

## Concurrency Notes

- Rust shared state is mostly `Arc<Mutex<...>>` managed by Tauri.
- `hotkey.rs` uses a `session_id` guard to prevent older transcription threads from writing stale output after a newer session starts.
- Overlay event delivery is not assumed reliable on Windows WebView2, so `useRecording` polls `get_recording_state` every 120 ms.
- Model downloads share `DownloadCancelFlag`; cancellation emits progress `-1.0` to the frontend.

## Review Checklist

- Did every new Tauri command appear in `generate_handler!`?
- Did every new frontend `invoke` name match the Rust command exactly?
- Did every new recording state update `RecordingState`, `VALID_STATES`, `StatusIndicator`, and `Monitor`?
- Did every new setting update Rust `AppSettings`, frontend interfaces, defaults, and persistence behavior?
- Did the change preserve local-only transcription/polishing unless explicitly requested otherwise?
- Did docs and Graphify notes stay current?
