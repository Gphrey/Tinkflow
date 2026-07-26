# Graph Report - D:\MIEBI\CodeDocs\Tinkflow  (2026-07-26)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 474 nodes · 714 edges · 55 communities (26 shown, 29 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2af50853`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- lib.rs
- .init_on_main_thread
- llm.rs
- package.json
- dictionary.rs
- bundle
- download_whisper_model
- settings.rs
- compilerOptions
- CorrectionManager
- HistoryManager
- String
- Onboarding.tsx
- permissions
- Monitor.tsx
- compilerOptions
- App.tsx
- get_foreground_window_title
- Dashboard.tsx
- SettingsView.tsx
- Sidebar.tsx
- Default
- PathBuf
- Arc
- Mutex
- Result
- SettingsManager
- AppHandle
- AppSettings
- Arc
- AtomicU32
- CorrectionManager
- HistoryManager
- Mutex
- OllamaClient
- Option
- RecordingStateStore
- SettingsManager
- String
- TranscriptionRecord
- WhisperTranscriber
- Option
- Result
- Self
- String
- DownloadCancelFlag
- Result
- State
- Vec

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `AudioCapturer` - 13 edges
3. `CorrectionManager` - 12 edges
4. `HistoryManager` - 12 edges
5. `OllamaClient` - 11 edges
6. `update_app_settings()` - 11 edges
7. `AppSettings` - 10 edges
8. `WhisperTranscriber` - 10 edges
9. `download_whisper_model()` - 10 edges
10. `bundle` - 10 edges

## Surprising Connections (you probably didn't know these)
- `Monitor()` --calls--> `useRecording()`  [EXTRACTED]
  tinkflow/src/components/Monitor.tsx → tinkflow/src/hooks/useRecording.ts
- `StatusIndicator()` --calls--> `useRecording()`  [EXTRACTED]
  tinkflow/src/components/StatusIndicator.tsx → tinkflow/src/hooks/useRecording.ts

## Import Cycles
- None detected.

## Communities (55 total, 29 thin omitted)

### Community 0 - "lib.rs"
Cohesion: 0.12
Nodes (47): App, CorrectionEntry, DictationResult, DictationResultStore, DownloadCancelFlag, Result, State, ActiveHotkeyId (+39 more)

### Community 1 - ".init_on_main_thread"
Cohesion: 0.08
Nodes (35): AppHandle, AppSettings, Arc, AtomicU32, AudioCapturer, Box, CorrectionManager, HistoryManager (+27 more)

### Community 2 - "llm.rs"
Cohesion: 0.09
Nodes (21): AtomicBool, Client, build_system_prompt(), default_matches_new(), GenerateRequest, GenerateResponse, ModelInfo, OllamaClient (+13 more)

### Community 3 - "package.json"
Cohesion: 0.06
Nodes (35): lucide-react, react, react-dom, @tauri-apps/api, @tauri-apps/cli, @tauri-apps/plugin-autostart, @tauri-apps/plugin-opener, dependencies (+27 more)

### Community 4 - "dictionary.rs"
Cohesion: 0.07
Nodes (8): AhoCorasick, default_is_same_as_new(), DeveloperDictionary, dict(), Default, Self, String, Vec

### Community 5 - "bundle"
Cohesion: 0.06
Nodes (31): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.ico, app, security, windows, build (+23 more)

### Community 6 - "download_whisper_model"
Cohesion: 0.18
Nodes (20): canonical_model_name(), check_whisper_model(), download_whisper_model(), get_model_path(), list_installed_whisper_models(), load_whisper_model(), model_url(), AppHandle (+12 more)

### Community 7 - "settings.rs"
Cohesion: 0.16
Nodes (17): AppSettings, ContextProfile, default_context_profiles(), default_hotkey(), default_injection_mode(), default_transcription_quality(), profile(), AppHandle (+9 more)

### Community 8 - "compilerOptions"
Cohesion: 0.09
Nodes (22): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+14 more)

### Community 9 - "CorrectionManager"
Cohesion: 0.22
Nodes (11): CorrectionEntry, CorrectionManager, now_ms(), replace_ascii_case_insensitive(), AppHandle, Mutex, PathBuf, Result (+3 more)

### Community 10 - "HistoryManager"
Cohesion: 0.22
Nodes (11): HistoryManager, now_ms(), AppHandle, Mutex, Option, PathBuf, Result, Self (+3 more)

### Community 11 - "String"
Cohesion: 0.27
Nodes (11): Enigo, Option, Self, String, DictationResult, copy_text_to_clipboard(), inject_via_clipboard(), InjectionReport (+3 more)

### Community 12 - "Onboarding.tsx"
Cohesion: 0.17
Nodes (7): MODEL_SIZES, ModelManager(), ModelManagerProps, AppSettings, OnboardingProps, STEPS, WAVEFORM_BARS

### Community 13 - "permissions"
Cohesion: 0.17
Nodes (11): autostart:allow-disable, autostart:allow-enable, autostart:allow-is-enabled, core:default, main, opener:default, description, identifier (+3 more)

### Community 14 - "Monitor.tsx"
Cohesion: 0.36
Nodes (7): EventLogEntry, Monitor(), StatusIndicator(), StatusIndicatorProps, RecordingState, useRecording(), VALID_STATES

### Community 15 - "compilerOptions"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 16 - "App.tsx"
Cohesion: 0.29
Nodes (4): AppSettings, DictationResult, DictationResultPanel(), PanelPhase

### Community 17 - "get_foreground_window_title"
Cohesion: 0.36
Nodes (5): ContextDetector, get_foreground_window_title(), Option, Self, String

### Community 18 - "Dashboard.tsx"
Cohesion: 0.40
Nodes (5): AppSettings, Dashboard(), ModelHealth, RECOMMENDED_MODELS, TranscriptionRecord

### Community 19 - "SettingsView.tsx"
Cohesion: 0.33
Nodes (4): AppSettings, ContextProfile, CorrectionEntry, WHISPER_MODELS

## Knowledge Gaps
- **97 isolated node(s):** `$schema`, `identifier`, `description`, `main`, `core:default` (+92 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `context_with_profile()` connect `.init_on_main_thread` to `String`?**
  _High betweenness centrality (0.003) - this node is a cross-community bridge._
- **What connects `$schema`, `identifier`, `description` to the rest of the system?**
  _97 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11613475177304965 - nodes in this community are weakly interconnected._
- **Should `.init_on_main_thread` be split into smaller, more focused modules?**
  _Cohesion score 0.08350951374207188 - nodes in this community are weakly interconnected._
- **Should `llm.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08636977058029689 - nodes in this community are weakly interconnected._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.05555555555555555 - nodes in this community are weakly interconnected._
- **Should `dictionary.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07196969696969698 - nodes in this community are weakly interconnected._