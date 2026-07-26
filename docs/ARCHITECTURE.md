# Architecture

Tinkflow is a local-first desktop dictation app built with React, TypeScript, Rust, and Tauri 2.

## Runtime Flow

```text
Global hotkey press
  -> AudioCapturer::start_recording()
  -> recording-state: listening

Global hotkey release
  -> AudioCapturer::stop_recording()
  -> recording-state: processing
  -> WhisperTranscriber::load_model() if needed
  -> recording-state: transcribing
  -> WhisperTranscriber::transcribe()
  -> DeveloperDictionary::apply()
  -> ContextDetector::detect_current_context()
  -> recording-state: polishing
  -> OllamaClient::polish_text() when Ollama and a model are available
  -> TextInjector::inject()
  -> HistoryManager::add()
  -> dictation-result: final text, record id, insertion outcome
  -> recording-state: done
  -> recording-state: idle
```

If audio is too short, the pipeline returns to `idle` without transcription.

## Frontend

The frontend lives in `tinkflow/src`.

- `App.tsx` decides whether the current Tauri window is the main app or overlay. Overlay detection happens synchronously from `getCurrentWindow().label`.
- `Dashboard.tsx` shows Whisper and Ollama status, handles local Ollama model selection and downloads.
- `SettingsView.tsx` manages app settings, audio input selection, startup behavior, hotkey choice, and Whisper model selection.
- `Onboarding.tsx` owns the full-window three-step first-run journey and model readiness. See `docs/ONBOARDING_UX.md` for the design contract.
- `Monitor.tsx` is a diagnostics page for recording-state events.
- `StatusIndicator.tsx` renders the live recording status pill used by the overlay and monitor.
- `DictationResultPanel.tsx` shows persisted final text, insertion status, and recovery copy controls in the overlay.
- `useRecording.ts` listens to `recording-state` and polls `get_recording_state` as a fallback.

Frontend talks to Rust through `invoke(...)` commands and Tauri events. Keep interfaces in sync manually; there is no generated IPC schema.

## Backend

The backend lives in `tinkflow/src-tauri/src`.

- `main.rs` only calls `tinkflow_lib::run()`.
- `lib.rs` wires modules, managed state, plugins, windows, and Tauri commands.
- `hotkey.rs` owns the dictation pipeline orchestration.
- `audio.rs` captures microphone audio with CPAL, converts to f32 mono, applies RMS VAD, and resamples to 16 kHz for Whisper.
- `whisper.rs` manages local GGML Whisper models and speech-to-text inference through `whisper-rs`.
- `llm.rs` talks to local Ollama for optional cleanup and has a sanitizer to reject unusable model output.
- `dictionary.rs` applies deterministic developer term corrections before LLM polishing.
- `context.rs` detects the foreground app/window title and maps it to context hints.
- `injector.rs` uses Enigo to insert final text into the active app.
- `settings.rs` persists `AppSettings` to the app data directory.

## IPC Commands

Registered in `lib.rs`:

- `greet`
- `check_whisper_model`
- `list_installed_whisper_models`
- `download_whisper_model`
- `load_whisper_model`
- `check_ollama_status`
- `list_ollama_models`
- `pull_ollama_model`
- `cancel_download`
- `get_app_settings`
- `update_app_settings`
- `get_audio_devices`
- `get_recording_state`
- `get_active_dictation_result`
- `set_overlay_interactive`
- `dismiss_overlay`

Events emitted by Rust:

- `recording-state`: one of `idle`, `listening`, `processing`, `loading-model`, `transcribing`, `polishing`, `done`, `error`.
- `dictation-result`: a persisted transcription result containing `record_id`, final `text`, `insertion_succeeded`, `injection_mode`, and `injection_error`.
- `model-download-progress`: Whisper model progress, with `-1.0` meaning cancelled.
- `ollama-download-progress`: Ollama model progress, with `-1.0` meaning cancelled.

## Important Couplings

- `SettingsManager` is shared by audio, hotkey updates, app startup, and frontend settings.
- `WhisperTranscriber` is shared between app startup, settings changes, and the hotkey pipeline.
- `OllamaClient` is shared between dashboard status/model management and polishing.
- `recording-state` must remain compatible with `useRecording`, `StatusIndicator`, and `Monitor`.
- The overlay relies on both event delivery and polling because Windows transparent always-on-top windows can miss events.
- `dictation-result` is emitted only after history persistence, so overlay recovery controls can use `record_id` with `copy_transcription`.
- The result panel polls `get_active_dictation_result` as an event-delivery fallback for transparent Windows overlays.

## Graphify Snapshot

Graphify code-only graph as of 2026-07-26 after the v1.5 installer cleanup refresh:

- 474 nodes
- 714 edges
- 55 communities
- 0 inferred edges
- 0 token cost

Most connected abstractions in the graph:

- `AudioCapturer`
- `OllamaClient`
- `WhisperTranscriber`
- `SettingsManager`
- `update_app_settings()`
- `download_whisper_model()`
- `DeveloperDictionary`

See `graphify-out/GRAPH_REPORT.md` for the full generated report.

## Version 1.5 Implementation Notes

Version 1.5 adds the reliability and personalization layer around the original dictation pipeline.

New backend modules:

- `history.rs`: JSON-backed transcription history in the app data directory, keeping the last 100 records.
- `corrections.rs`: JSON-backed personal correction entries, applied after the static `DeveloperDictionary`.

New/expanded runtime behavior:

- The hotkey pipeline respects `dictation_enabled` before starting a recording.
- Transcription output is saved to history before insertion success is considered final.
- `TextInjector` supports `auto`, `direct`, and `clipboard` modes. `auto` tries direct Enigo typing first, then clipboard paste fallback.
- Failed insertion copies the final text to the clipboard when possible, so spoken output is recoverable.
- Context profiles are stored in `AppSettings` and passed into the Ollama prompt as profile hints.
- A tray menu exposes Open Tinkflow, Toggle Dictation, Copy Last Transcription, and Quit.
- `get_model_health` reports Whisper install state, Ollama reachability/version/models, injection mode, and dictation enabled state.

New IPC commands:

- `list_transcription_history`
- `clear_transcription_history`
- `copy_last_transcription`
- `list_corrections`
- `add_correction`
- `remove_correction`
- `set_correction_enabled`
- `get_model_health`

Frontend additions:

- Dashboard includes a Model Center summary and recent dictation recovery list.
- Settings includes dictation enable/disable, insertion mode, personal corrections, and context profile controls.
- Onboarding uses a full-window three-step journey, detects the persisted Whisper model, and keeps Ollama optional.
