# Tinkflow 1.5 Research And Suggestions

Date: 2026-07-20

This document proposes a practical v1.5 direction for Tinkflow based on the current codebase, the Graphify map, and ecosystem research around Tauri, Whisper/whisper.cpp, and Ollama.

## Current Baseline

Tinkflow is already a local-first desktop dictation app:

- Global hotkey starts and stops recording.
- CPAL captures microphone audio and resamples it to 16 kHz mono.
- Whisper handles local transcription.
- DeveloperDictionary applies deterministic corrections.
- Ollama optionally polishes text locally.
- Enigo injects the final text into the active app.
- React/Tauri frontend shows onboarding, dashboard, settings, monitor, and overlay state.

Graphify highlights the main hubs as `AudioCapturer`, `WhisperTranscriber`, `OllamaClient`, `SettingsManager`, `hotkey.rs`, and `lib.rs`. That means v1.5 should strengthen the pipeline, not scatter effort into unrelated surfaces.

## Recommended 1.5 Theme

**Tinkflow 1.5: Reliable local dictation with context-aware correction and a professional desktop shell.**

The release should make Tinkflow feel less like a promising pipeline and more like a daily tool: faster feedback, fewer failed insertions, better correction memory, clearer model setup, and stronger diagnostics.

## Release Pillars

### 1. Streaming-Like Dictation Feedback

Current behavior records first, then transcribes after release. For 1.5, keep hold-to-talk, but add partial progress and better silence handling.

Suggested work:

- Replace the simple RMS-only VAD decision with a small state machine: silence, speech-started, speech-active, speech-ended.
- Track audio duration, active speech duration, and silence duration in `AudioCapturer`.
- Emit richer states such as `listening`, `speech-detected`, `processing`, `transcribing`.
- Add optional automatic stop after a configurable silence timeout.
- Show elapsed recording time and activity level in the overlay.

Why it matters:

- Makes the app feel alive while recording.
- Reduces empty/too-short captures.
- Prepares the codebase for real streaming later.

Implementation area:

- `tinkflow/src-tauri/src/audio.rs`
- `tinkflow/src-tauri/src/hotkey.rs`
- `tinkflow/src/hooks/useRecording.ts`
- `StatusIndicator.tsx`, `Monitor.tsx`

Research note: whisper.cpp includes streaming examples with sliding-window/VAD modes, which suggests the ecosystem has already moved toward speech-activity-aware transcription.

### 2. Personal Correction Memory

The current dictionary is static. Version 1.5 should learn user corrections locally.

Suggested work:

- Add a local `corrections` store using the existing Tauri SQL plugin.
- Let users add phrases like `spoken -> replacement` from Settings.
- Track accepted LLM corrections and offer them as suggested dictionary entries.
- Apply user corrections before the static `DeveloperDictionary`, or merge both into one correction pass with deterministic precedence.
- Add import/export for correction profiles.

Why it matters:

- Developer dictation depends heavily on personal vocabulary: project names, package names, APIs, client names, acronyms.
- This gives Tinkflow compounding value without requiring cloud accounts.

Implementation area:

- New Rust module: `src-tauri/src/corrections.rs`
- Existing `dictionary.rs`
- `SettingsView.tsx`
- Tauri SQL plugin already present in `lib.rs`

### 3. Context Profiles

`ContextDetector` already maps active windows to broad contexts. Version 1.5 can turn that into a visible feature.

Suggested work:

- Add per-context polishing behavior: code, chat, email, terminal, general.
- Expose context profiles in Settings with toggles for punctuation, filler removal, technical-symbol preservation, and tone.
- Add app-specific overrides by window title match.
- Show detected context in Monitor and optionally in the overlay.

Why it matters:

- The same sentence should be polished differently in Slack, VS Code, an email client, and a terminal.
- This builds directly on code that already exists.

Implementation area:

- `context.rs`
- `llm.rs` prompt builder
- `settings.rs` AppSettings
- `SettingsView.tsx`, `Monitor.tsx`

### 4. Injection Reliability Layer

Current injection uses Enigo text insertion directly. Version 1.5 should add fallbacks and observability.

Suggested work:

- Add configurable injection modes: direct typing, clipboard paste, clipboard paste with restore.
- Detect injection failures and show a recoverable notification/state.
- Add a “copy last result” action in tray/dashboard.
- Store the last N transcriptions locally for recovery.

Why it matters:

- Text injection varies across apps, permission levels, and operating systems.
- A dictation tool must never lose the user’s spoken output.

Implementation area:

- `injector.rs`
- new history store
- Dashboard/Monitor UI
- possible Tauri clipboard plugin

### 5. Model And Setup Center

Model setup is split between onboarding, dashboard, and settings. Version 1.5 should unify it.

Suggested work:

- Create one Model Center view for Whisper and Ollama.
- Show installed models, active model, disk usage, download progress, and cancellation.
- Add model health checks: Whisper file exists/loadable, Ollama API reachable, selected LLM available.
- Add a tiny local benchmark: sample transcription latency and LLM polish latency.
- Use Ollama `/api/version` and `/api/tags` for clearer diagnostics.

Why it matters:

- Users need to understand what is local, what is installed, and why something is offline.
- This reduces first-run friction.

Implementation area:

- `Dashboard.tsx`
- `SettingsView.tsx`
- `ModelManager.tsx`
- `whisper.rs`
- `llm.rs`

### 6. Desktop Shell Polish

The app should behave like a resident utility, not only a window.

Suggested work:

- Add a system tray menu: Open Tinkflow, Toggle enabled, Copy last transcription, Settings, Quit.
- Add update support for packaged releases.
- Add clearer startup/autostart state handling.
- Add native notifications for model download failures and pipeline errors.

Why it matters:

- Dictation tools live in the background.
- Tauri already has official APIs/plugins for tray, shortcuts, updater, autostart, clipboard, and notifications.

Implementation area:

- `lib.rs` setup
- Tauri config/capabilities
- frontend shell components

## Best 1.5 Feature Set

If scope needs to stay tight, ship these five:

1. Personal Correction Memory
2. Injection Reliability Layer with clipboard fallback and last-result recovery
3. Model Center with health checks
4. Context Profiles
5. Tray Menu with enabled/disabled state and copy-last-result

These are high leverage because they reuse the existing architecture and make the app meaningfully better for daily use.

## Stretch Ideas

- Real-time partial transcription preview.
- Voice commands such as “scratch that”, “new line”, “send”, “copy that”.
- Per-application dictionaries.
- Local semantic memory using Ollama embeddings for project vocabulary.
- Prompt/polish A/B testing in Monitor.
- Import glossary from repo files or Graphify nodes.
- Accessibility permissions checklist per OS.
- Portable diagnostics bundle for bug reports.

## Suggested Milestones

### Milestone 1: Reliability Foundation

- Add transcription history store.
- Add copy-last-result and injection fallback.
- Add tray menu.
- Add errors that preserve final text instead of losing it.

### Milestone 2: Personalization

- Add user correction memory.
- Add context profiles.
- Update LLM prompt builder to use profile settings.
- Add Settings UI for corrections and profiles.

### Milestone 3: Model Center

- Merge model setup surfaces.
- Add health checks and model diagnostics.
- Add latency benchmark.
- Improve download cancellation and recovery UI.

### Milestone 4: Release Polish

- Add updater path.
- Refresh documentation.
- Run cross-platform smoke tests.
- Bump app version to `1.5.0` when release scope is implemented.

## Versioning Note

Do not bump package versions to `1.5.0` until the 1.5 feature scope is implemented or intentionally accepted as a planning-only version. Current package metadata is `0.2.0`.

## Research References

- Tauri updater plugin: https://v2.tauri.app/reference/javascript/updater/
- Tauri global shortcut plugin: https://tauri.app/reference/javascript/global-shortcut/
- Tauri tray API: https://v2.tauri.app/reference/javascript/api/namespacetray/
- Tauri feature/plugin catalog: https://v2.tauri.app/plugin/
- Ollama API introduction: https://docs.ollama.com/api/introduction
- Ollama embeddings: https://docs.ollama.com/capabilities/embeddings
- Ollama API structured outputs/tools reference: https://github.com/ollama/ollama/blob/main/docs/api.md
- whisper.cpp streaming example: https://github.com/ggml-org/whisper.cpp/blob/master/examples/stream/README.md
- whisper.cpp releases: https://github.com/ggml-org/whisper.cpp/releases
