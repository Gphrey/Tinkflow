# Tinkflow Agent Guide

This file is the canonical starting point for AI coding agents working in this repository. Model-specific files such as `CLAUDE.md` and `GEMINI.md` should link here rather than duplicate instructions.

## First Moves

1. Read this file, then read [docs/AGENT_COLLABORATION.md](docs/AGENT_COLLABORATION.md).
2. Check [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the current module map and data flow.
3. Use Graphify before broad codebase questions:
   - Existing graph: `graphify query "your question"`
   - Refresh after code changes: `graphify . --code-only --no-viz`
   - Report: [graphify-out/GRAPH_REPORT.md](graphify-out/GRAPH_REPORT.md)
4. Run `git status --short` before editing and do not overwrite uncommitted user work.

## Project Shape

Tinkflow is a local-first desktop dictation app.

- Frontend: React 19 + TypeScript + Vite in `tinkflow/src`.
- Desktop shell: Tauri 2 in `tinkflow/src-tauri`.
- Backend: Rust modules for global hotkeys, audio capture, Whisper transcription, optional Ollama polishing, and text injection.
- Main app command surface: `tinkflow/src-tauri/src/lib.rs`.

## High-Risk Areas

- `tinkflow/src-tauri/src/hotkey.rs`: orchestrates the recording pipeline and emits UI state.
- `tinkflow/src-tauri/src/audio.rs`: captures and resamples microphone audio.
- `tinkflow/src-tauri/src/whisper.rs`: downloads, loads, and runs local Whisper models.
- `tinkflow/src-tauri/src/llm.rs`: talks to local Ollama and sanitizes model output.
- `tinkflow/src/hooks/useRecording.ts`: keeps main and overlay windows synchronized.

Treat changes in these files as behavior changes, not simple refactors.

## Build And Check Commands

Run from `tinkflow/` unless noted.

```powershell
npm install
npm run build
npm run tauri dev
```

Rust checks:

```powershell
cd tinkflow\\src-tauri
$env:LIBCLANG_PATH='C:\\Program Files\\LLVM\\bin'
$env:CARGO_TARGET_DIR='C:\\tmp\\t'
cargo check
cargo test --lib corrections
```

Windows Vulkan Whisper builds are opt-in: cargo check --features whisper-vulkan after setting VULKAN_SDK.

Graphify commands from the repository root:

```powershell
graphify . --code-only --no-viz
graphify cluster-only D:\MIEBI\CodeDocs\Tinkflow --no-viz
graphify query "How does the dictation pipeline work?"
```

## Collaboration Rules

- Prefer small, inspectable changes with a short handoff note.
- Document any new Tauri command in both Rust and the frontend call site.
- Keep the Rust/frontend IPC names stable unless the change intentionally migrates every caller.
- Preserve local-first behavior. Whisper and Ollama are local runtime dependencies; do not add cloud model calls to the app without an explicit product decision.
- If a change touches audio, hotkeys, model downloads, or injection, include manual verification steps because these depend on OS state and hardware.


<claude-mem-context>
# Memory Context

# [Tinkflow] recent context, 2026-07-26 5:54pm GMT+1

Legend: 🎯session 🔴bugfix 🟣feature 🔄refactor ✅change 🔵discovery ⚖️decision 🚨security_alert 🔐security_note
Format: ID TIME TYPE TITLE
Fetch details: get_observations([IDs]) | Search: mem-search skill

Stats: 50 obs (16,767t read) | 485,767t work | 97% savings

### Jul 26, 2026
1738 4:50p ✅ Hotkey Overlay Fix Compiles Successfully Without Errors
1739 4:54p ✅ Overlay Architecture Refactored: Result Panel, History, and Tray Integration Added
1740 " 🔵 Settings Storage Location and Configuration Structure Identified
1741 4:55p 🔵 Root Cause Found: dictation_enabled Set to False in Settings
1742 " 🔴 Dictation Re-enabled in Persisted Settings
1743 5:03p 🔴 Restored Listening flow by fixing dictation_enabled flag in Tinkflow settings
1744 " ⚖️ Scope Copy feature fix tightly to result panel interaction only
1745 5:04p 🔵 Tauri cursor-event APIs located in local cargo registry
1746 5:05p 🔵 Tauri set_ignore_cursor_events API chain identified in 2.11.5
1747 " 🔴 Overlay cursor events now routed through Tauri main UI thread
1748 " 🔵 Compilation error in overlay cursor-event fix: borrow checker conflict
1749 " 🔴 Applied compiler-suggested clone fix for borrow checker error
1750 " 🔵 Compilation check passes for overlay cursor-event threading fix
1751 5:06p ✅ Code graph updated to reflect overlay cursor-event threading fix
1752 " 🔵 Tauri dev process check failed; unclear if watcher reloaded with fix
1753 " ✅ Tinkflow Tauri dev server launched with overlay cursor-event threading fix
1755 5:10p 🔴 Fixed hotkey worker transcription_quality scope error
1756 " 🔴 Fixed Whisper API method call set_suppress_non_speech_tokens → set_suppress_nst
1757 " 🟣 Recognition quality modes: Balanced and Accurate settings added
1758 " 🟣 Whisper accuracy enhancements: vocabulary, suppression, deterministic decoding
1759 " 🟣 Per-transcription copy and history management features
1760 " 🔵 Ollama model request failure despite health check passing
1761 " 🔵 UI redesign identified as blocking concern before additional feature work
1762 " ⚖️ UI redesign prioritized before bottom-center result popup feature
1754 " 🔵 Graphify mapped clipboard write path through copy_transcription → copy_text_to_clipboard → set_clipboard_text
1763 5:11p 🔵 Windows clipboard write uses PowerShell subprocess; vulnerable to silent failures in Tauri context
1764 " 🔵 Tinkflow does not use direct Windows clipboard APIs; confirms PowerShell subprocess is only clipboard method
1765 " 🔵 Win32 clipboard APIs are available in winapi crate but unused in Tinkflow implementation
1766 " 🔴 Replaced PowerShell subprocess clipboard with native Win32 API implementation
1767 5:12p ✅ Windows clipboard implementation replaced with native Win32 API (Cargo.toml and injector.rs updated)
1768 " 🔵 Native Win32 clipboard implementation compiles successfully without errors
1769 5:13p 🔵 All 37 unit tests pass with native Win32 clipboard implementation
1770 5:14p 🔵 Tinkflow process launched successfully with native Win32 clipboard implementation compiled
1771 " 🔵 Graphify codebase map updated to reflect native Win32 clipboard implementation
1772 5:15p 🔵 Graphify report finalized with 55 clipboard-aware communities after native Win32 implementation
S813 Fix Copy button in Tinkflow dictation result panel—shows "Copied" but clipboard remains empty, preventing paste (Jul 26, 5:15 PM)
1773 " ✅ Architecture documentation updated with native clipboard implementation metrics
S812 Verify git integrity and clipboard fix commit status (Jul 26, 5:15 PM)
S814 Reset Tinkflow to first-run state and validate v1.5.0 onboarding experience as new user. User completing v1.5.0 development cycle with UI/UX improvements, transcription result panel, copy button, and bug fixes. (Jul 26, 5:15 PM)
1774 5:22p 🔵 Windows sandbox ACLs block read access to .codex skills directory
1776 " 🔵 Rust Compilation Errors in Tinkflow Audio/Transcription Pipeline
1777 " 🟣 Transcription Result Popup with Copy and Status Feedback
1778 " 🔵 UI Behavior Issues with Listening State and Transcription Display
1779 " ⚖️ Testing Plan: Full Onboarding Flow Validation for v1.5.0
1775 " 🔵 Windows sandbox ACLs block all file/code operations on project directory
S815 Reset Tinkflow app to first-run state and launch for fresh onboarding experience testing. Execute app state reset by stopping process and restoring onboarding flag. (Jul 26, 5:23 PM)
1780 5:23p 🔵 Tinkflow onboarding state stored in AppSettings with boolean flag
1781 " 🔵 Tinkflow project has unstaged modifications across frontend and backend
1782 " ✅ Tinkflow app reset to first-run state by resetting onboarding_completed flag
S816 Reset Tinkflow app to first-time onboarding state so user can experience the onboarding flow as a new user (Jul 26, 5:23 PM)
S817 Setup agentic collaboration documentation and upgrade Tinkflow application to v1.5.0 with improved UX, including onboarding redesign and text detection popup feature. (Jul 26, 5:23 PM)
1783 5:32p 🔵 Rust Compilation Errors in Hotkey and Whisper Modules
1784 " 🔴 Feature Regression: Listening UI Overlay and Copy Button Malfunction
1785 " ⚖️ Onboarding UI Redesign Decision: Use Graphify for Architecture Planning
1786 " 🔵 Onboarding component architecture mapped to Tauri window configuration
S818 Setup agentic collaboration documentation and upgrade Tinkflow to v1.5.0 with improved UX, featuring systematic onboarding redesign using Graphify architecture tool and text detection popup functionality. (Jul 26, 5:32 PM)
1787 " 🔵 Desktop window constraint and onboarding UI layout analysis
S819 Upgrade Tinkflow to v1.5.0 with systematic onboarding UI/UX redesign using Graphify methodology, addressing current cramped card-based design to achieve premium application appearance. (Jul 26, 5:33 PM)
S820 Upgrade Tinkflow to v1.5.0 with systematic onboarding UI/UX redesign using Graphify methodology, transforming current cramped card-based design into a premium, welcoming first-time user experience. (Jul 26, 5:33 PM)
S821 Upgrade Tinkflow to v1.5.0 with systematic onboarding UI/UX redesign using Graphify methodology to transform card-based design into premium, welcoming first-time user experience. (Jul 26, 5:34 PM)
**Investigated**: Rust compilation errors in hotkey.rs and whisper.rs, listening UI state management and event flow, text popup feature integration points, copy button clipboard behavior, Graphify architecture methodology, local onboarding component implementation, broader UI-related source code architecture and state management patterns, and current design constraints affecting visual hierarchy and user journey.

**Learned**: Current onboarding uses cramped card-based design lacking visual polish. Graphify provides systematic redesign methodology. Rust codebase has API mismatches with cpal library requiring fixes. UI layer has tight state management coupling that caused previous feature regressions. Source inspection revealed architectural patterns that must be respected during redesign. Frontend icon library needed to support premium visual design improvements for new onboarding.

**Completed**: Identified all v1.5.0 blockers and regression root causes. Approved Graphify-based redesign methodology. Completed comprehensive source code analysis of onboarding and UI architecture. Identified and approved installation of frontend icon dependency to support redesigned UI.

**Next Steps**: Install frontend icon dependency. Design new onboarding UI leveraging icon library and premium visual patterns researched. Create improved user journey flow through onboarding. Implement redesigned onboarding component respecting identified state management patterns. Verify redesign integrates properly with existing listening UI, text popup feature, and copy button functionality. Then address remaining v1.5.0 items: Rust compilation errors and text popup feature implementation.


Access 486k tokens of past work via get_observations([IDs]) or mem-search skill.
</claude-mem-context>
