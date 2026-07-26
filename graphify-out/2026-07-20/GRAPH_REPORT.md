# Graph Report - D:\MIEBI\CodeDocs\Tinkflow  (2026-07-20)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 425 nodes · 668 edges · 39 communities (23 shown, 16 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c95b10b9`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- lib.rs
- llm.rs
- package.json
- dictionary.rs
- App.tsx
- bundle
- WhisperTranscriber
- AudioCapturer
- compilerOptions
- settings.rs
- CorrectionManager
- HistoryManager
- String
- .init_on_main_thread
- permissions
- compilerOptions
- get_foreground_window_title
- CorrectionEntry
- Arc
- Mutex
- Result
- SettingsManager
- Default
- AppSettings
- AtomicU32
- RecordingStateStore
- Default
- DownloadCancelFlag
- Mutex
- Option
- PathBuf
- State
- TranscriptionRecord

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `OllamaClient` - 16 edges
3. `CorrectionManager` - 14 edges
4. `AudioCapturer` - 14 edges
5. `HistoryManager` - 13 edges
6. `WhisperTranscriber` - 13 edges
7. `update_app_settings()` - 11 edges
8. `AppSettings` - 10 edges
9. `download_whisper_model()` - 10 edges
10. `bundle` - 9 edges

## Surprising Connections (you probably didn't know these)
- `check_ollama_status()` --references--> `OllamaClient`  [EXTRACTED]
  tinkflow/src-tauri/src/lib.rs → tinkflow/src-tauri/src/llm.rs
- `list_ollama_models()` --references--> `OllamaClient`  [EXTRACTED]
  tinkflow/src-tauri/src/lib.rs → tinkflow/src-tauri/src/llm.rs
- `update_app_settings()` --references--> `WhisperTranscriber`  [EXTRACTED]
  tinkflow/src-tauri/src/lib.rs → tinkflow/src-tauri/src/whisper.rs
- `get_model_health()` --references--> `OllamaClient`  [EXTRACTED]
  tinkflow/src-tauri/src/lib.rs → tinkflow/src-tauri/src/llm.rs
- `Monitor()` --calls--> `useRecording()`  [EXTRACTED]
  tinkflow/src/components/Monitor.tsx → tinkflow/src/hooks/useRecording.ts

## Import Cycles
- None detected.

## Communities (39 total, 16 thin omitted)

### Community 0 - "lib.rs"
Cohesion: 0.15
Nodes (37): App, AppSettings, AtomicU32, CorrectionManager, HistoryManager, RecordingStateStore, ActiveHotkeyId, add_correction() (+29 more)

### Community 1 - "llm.rs"
Cohesion: 0.09
Nodes (20): AtomicBool, Client, build_system_prompt(), default_matches_new(), GenerateRequest, GenerateResponse, ModelInfo, OllamaClient (+12 more)

### Community 2 - "package.json"
Cohesion: 0.06
Nodes (33): react, react-dom, @tauri-apps/api, @tauri-apps/cli, @tauri-apps/plugin-autostart, @tauri-apps/plugin-opener, dependencies, react (+25 more)

### Community 3 - "dictionary.rs"
Cohesion: 0.07
Nodes (8): AhoCorasick, Default, default_is_same_as_new(), DeveloperDictionary, dict(), Self, String, Vec

### Community 4 - "App.tsx"
Cohesion: 0.09
Nodes (22): AppSettings, AppSettings, Dashboard(), ModelHealth, RECOMMENDED_MODELS, TranscriptionRecord, ModelManager(), EventLogEntry (+14 more)

### Community 5 - "bundle"
Cohesion: 0.07
Nodes (28): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.ico, app, security, windows, build (+20 more)

### Community 6 - "WhisperTranscriber"
Cohesion: 0.19
Nodes (19): DownloadCancelFlag, Option, PathBuf, State, canonical_model_name(), check_whisper_model(), download_whisper_model(), get_model_path() (+11 more)

### Community 7 - "AudioCapturer"
Cohesion: 0.14
Nodes (17): Arc, Box, Mutex, Result, Send, Sender, SettingsManager, Stream (+9 more)

### Community 8 - "compilerOptions"
Cohesion: 0.09
Nodes (22): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+14 more)

### Community 9 - "settings.rs"
Cohesion: 0.16
Nodes (16): AppSettings, ContextProfile, default_context_profiles(), default_hotkey(), default_injection_mode(), profile(), AppHandle, Default (+8 more)

### Community 10 - "CorrectionManager"
Cohesion: 0.22
Nodes (11): CorrectionEntry, CorrectionManager, now_ms(), replace_ascii_case_insensitive(), AppHandle, Mutex, PathBuf, Result (+3 more)

### Community 11 - "HistoryManager"
Cohesion: 0.23
Nodes (11): HistoryManager, now_ms(), AppHandle, Mutex, Option, PathBuf, Result, Self (+3 more)

### Community 12 - "String"
Cohesion: 0.32
Nodes (11): Enigo, copy_text_to_clipboard(), inject_via_clipboard(), InjectionReport, paste_clipboard(), Option, Result, Self (+3 more)

### Community 13 - ".init_on_main_thread"
Cohesion: 0.21
Nodes (13): HotKey, context_with_profile(), emit_recording_state(), HotkeyListener, parse_hotkey(), AppHandle, AppSettings, Arc (+5 more)

### Community 14 - "permissions"
Cohesion: 0.17
Nodes (11): autostart:allow-disable, autostart:allow-enable, autostart:allow-is-enabled, core:default, main, opener:default, description, identifier (+3 more)

### Community 15 - "compilerOptions"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 16 - "get_foreground_window_title"
Cohesion: 0.36
Nodes (5): ContextDetector, get_foreground_window_title(), Option, Self, String

## Knowledge Gaps
- **87 isolated node(s):** `$schema`, `identifier`, `description`, `main`, `core:default` (+82 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `OllamaClient` connect `llm.rs` to `lib.rs`, `dictionary.rs`, `.init_on_main_thread`?**
  _High betweenness centrality (0.158) - this node is a cross-community bridge._
- **Why does `WhisperTranscriber` connect `WhisperTranscriber` to `lib.rs`, `.init_on_main_thread`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **What connects `$schema`, `identifier`, `description` to the rest of the system?**
  _87 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `llm.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08961593172119488 - nodes in this community are weakly interconnected._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.058823529411764705 - nodes in this community are weakly interconnected._
- **Should `dictionary.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07196969696969698 - nodes in this community are weakly interconnected._
- **Should `App.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09090909090909091 - nodes in this community are weakly interconnected._