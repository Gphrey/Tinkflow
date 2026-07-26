# Testing And Verification

Tinkflow has both normal build checks and OS-dependent manual checks.

## Automated Checks

From `tinkflow/`:

```powershell
npm run build
```

From `tinkflow/src-tauri/`:

```powershell
cargo check
cargo test
```

## Manual Checks

Use `npm run tauri dev` from `tinkflow/`.

Recording pipeline:

- Press and hold the configured hotkey.
- Confirm the overlay appears and shows `Listening...`.
- Release the hotkey.
- Confirm states progress through processing/transcribing/polishing/done or return to idle on short audio.
- Confirm text is inserted into the active app.

Settings:

- Change the hotkey and confirm the old hotkey no longer starts recording.
- Change the audio device and confirm missing devices fail with a useful message.
- Toggle launch at startup and confirm the OS autostart plugin state matches persisted settings.

Models:

- Download a Whisper model and confirm progress updates.
- Cancel a download and confirm the UI resets through the `-1.0` sentinel.
- If Ollama is installed, pull a model and confirm `llm_model` is selected.

Overlay:

- Open Monitor and compare its state log against the overlay.
- If Monitor updates but overlay sticks, inspect event delivery and the `useRecording` polling fallback.

## Version 1.5 Checks

Additional manual checks for v1.5:

- Change insertion mode between `auto`, `direct`, and `clipboard`; confirm text is inserted or saved to history if insertion fails.
- Use Copy Last from Dashboard and from the tray menu after a transcription.
- Add a personal correction and confirm it applies after Whisper transcription.
- Disable dictation in Settings or from the tray menu and confirm the global hotkey no longer starts capture.
- Edit a context profile tone and confirm the local LLM polishing prompt still preserves the intended context behavior.
- Open Dashboard Model Center and confirm Whisper/Ollama health reflects local model state.

Known Windows environment notes:

- Rust verification requires `clang.dll`/`libclang.dll` for `whisper-rs-sys` bindgen. In PowerShell, set `LIBCLANG_PATH` to the directory containing `libclang.dll`.
- Use a short Cargo target directory on Windows to avoid MSBuild path-length failures in native Whisper/CMake output: `$env:CARGO_TARGET_DIR='C:\tmp\t'`.
- Windows defaults to the CPU Whisper build for reliable agent verification. Vulkan acceleration is opt-in with `cargo check --features whisper-vulkan` and requires `VULKAN_SDK`.

Recommended Windows check:

```powershell
$env:LIBCLANG_PATH='C:\Program Files\LLVM\bin'
$env:CARGO_TARGET_DIR='C:\tmp\t'
cargo check
cargo test --lib corrections
```
