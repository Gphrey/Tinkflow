# Tinkflow

Tinkflow is a lightning-fast, keyboard-driven voice assistant application built for desktop environments, utilizing native system APIs via Rust and Tauri, combined with a fast React frontend.

## Features Currently Implemented (Phase 1 & 2)
- **Tauri Application:** Native desktop shell with hot-reloading React/TypeScript frontend.
- **Global Hotkeys:** `Ctrl+Shift+Space` serves as the primary mechanism to start and stop transcription using `global-hotkey` crate.
- **Audio Capture Pipeline:** Connects to the system's default microphone using `cpal`, dynamically resamples audio to 16kHz f32 mono utilizing `rubato`, and buffers the data across threads using `crossbeam-channel`.
- **UI State Synchronization:** Hotkey presses emit Tauri IPC events directly to the React frontend to update the recording status (Idle -> Listening -> Processing).

## Prerequisites (Windows)
- Node.js 22+
- Rust 1.93+
- Build Tools for Visual Studio 2022 (C++ workload)
- CMake
- LLVM/Clang

## Running the App
1. Install dependencies:
   ```bash
   npm install
   ```
2. Run the development server:
   ```bash
   npm run tauri dev
   ```

## Usage
- Open the Tinkflow app.
- Press and hold `Ctrl+Space` anywhere on your system to start recording. The app UI should show "Listening...".
- Release the hotkey to stop recording. The app UI should show "Processing..." and the Rust backend will output the number of captured samples in the terminal.

## Architecture
- **Frontend:** React + TypeScript + Vite
- **Backend:** Rust (Tauri 2.x)
- **Audio:** `cpal`, `rubato`
- **Hotkeys:** `global-hotkey`
- **Planned Models:** whisper-rs (local transcription)
