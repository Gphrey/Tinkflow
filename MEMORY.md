# Memory

Durable project memory for AI agents working on Tinkflow. Keep this short, factual, and useful across sessions. Put detailed instructions in `AGENTS.md` or `docs/`, not here.

## Project Identity

Tinkflow is a local-first desktop dictation app for developers. It uses React + TypeScript + Vite for the frontend and Rust + Tauri 2 for the desktop/backend layer.

## Current Architecture Snapshot

- Main Rust wiring lives in `tinkflow/src-tauri/src/lib.rs`.
- The dictation pipeline is orchestrated in `tinkflow/src-tauri/src/hotkey.rs`.
- Audio capture is in `tinkflow/src-tauri/src/audio.rs` and produces 16 kHz f32 mono audio for Whisper.
- Local transcription is in `tinkflow/src-tauri/src/whisper.rs` via `whisper-rs` and GGML model files.
- Optional text polishing is local through Ollama in `tinkflow/src-tauri/src/llm.rs`.
- Personal corrections and the developer dictionary run before optional LLM polishing.
- Text insertion and native Windows clipboard recovery live in `tinkflow/src-tauri/src/injector.rs`.
- Frontend recording state sync uses event listening plus polling fallback in `tinkflow/src/hooks/useRecording.ts`.

## Graphify State

Graphify is installed globally and initialized for this repository.

- Outputs live in `graphify-out/`.
- The current code-only graph has 471 nodes, 711 edges, and 55 communities.
- Useful query terms include `SettingsManager`, `WhisperTranscriber`, `OllamaClient`, `AudioCapturer`, `Onboarding()`, and `hotkey.rs`.
- Refresh from the repository root with `graphify . --code-only --no-viz`.
- Rebuild the report with `graphify cluster-only D:\MIEBI\CodeDocs\Tinkflow --no-viz`.

## Version 1.5 State

Implemented v1.5 features include:

- Transcription history with copy recovery.
- Direct, clipboard, and automatic insertion modes.
- Personal corrections stored locally and applied in the hotkey pipeline.
- Context profiles included in local Ollama prompt context.
- Model health reporting and Dashboard model management.
- Tray actions for opening Tinkflow, toggling dictation, copying the last transcription, and quitting.
- A bottom-center dictation result panel with inserted/saved state and native clipboard copy.
- A full-window three-step onboarding journey: Welcome, Voice setup, Ready.

## Onboarding Decision

The first-run experience is not a modal or card. It fills the Tauri window, demonstrates the real voice-to-text workflow, requires only Whisper, and treats Ollama as optional. The configured Whisper model must be detected instead of assuming `tiny.en`. The design prompt, research references, journey, and layout contract are in `docs/ONBOARDING_UX.md`.

Default window size is 1080 by 720 with a 900 by 620 minimum. The onboarding has been visually checked at the default native size and near the minimum size.

## Verification State

- `npm run build` passes after the onboarding refresh.
- `cargo check` and `cargo test --lib` passed during the v1.5 result-panel and clipboard pass.
- Windows Whisper uses CPU by default for reliable builds; Vulkan remains opt-in and requires `VULKAN_SDK`.

## Agent Notes

- The canonical agent guide is `AGENTS.md`.
- Cross-model collaboration protocol is `docs/AGENT_COLLABORATION.md`.
- Architecture details are in `docs/ARCHITECTURE.md`.
- Verification notes are in `docs/TESTING.md`.
- Do not store secrets, API keys, model keys, or personal tokens here.