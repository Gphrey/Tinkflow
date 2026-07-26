# Onboarding UX Direction

## Product Intent

The first-run experience should feel like entering Tinkflow, not opening a setup dialog. It uses the full desktop window, demonstrates the product in motion, and gets the user to a successful dictation with only one required decision: preparing a local Whisper model.

## Research Principles

- Keep first run focused on what is required to become productive. Whisper is required; Ollama is clearly optional.
- Teach through a product demonstration rather than paragraphs of feature explanation.
- Use warm, concise, action-led writing and safe defaults.
- Let the desktop window breathe and adapt fluidly instead of compressing a wide interface into a modal card.
- Keep motion purposeful: the waveform, caret, and result state explain the voice-to-text sequence.

Primary references:

- Apple Human Interface Guidelines, Onboarding: https://developer.apple.com/design/human-interface-guidelines/onboarding
- Microsoft First Experience guidance: https://learn.microsoft.com/en-us/windows/win32/uxguide/exper-first-exper
- Microsoft Windows typography: https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography
- Microsoft layout, alignment, margin, and padding: https://learn.microsoft.com/en-us/windows/apps/design/layout/alignment-margin-padding
- Microsoft writing style: https://learn.microsoft.com/en-us/windows/apps/design/style/writing-style

## User Journey

1. Welcome
   - Lead with the outcome: "Speak. Tinkflow writes."
   - Show Tinkflow working inside an editor, including listening, insertion, history, and copy recovery.
   - Establish local privacy without making the user read a technical explanation.

2. Voice setup
   - Detect the Whisper model selected in persisted settings and recognize it when already installed.
   - Offer one clear download action only when the selected model is missing.
   - Detect Ollama alongside Whisper setup, but never block progress when Ollama is unavailable.

3. Ready
   - Make the global shortcut the visual focus.
   - Preview the listen, transcribe, polish, and insert sequence.
   - Explain one recovery promise: finished text remains in History when insertion is unavailable.

## Master Design Prompt

Design a premium first-run experience for Tinkflow, a local-first desktop voice-to-text application for developers and knowledge workers. The onboarding must fill the application window edge to edge and must not appear inside a modal, floating card, or centered setup box. Use a dark neutral workspace with restrained teal and warm amber accents, strong left-aligned hierarchy, generous breathing room, crisp system-like typography, and subtle borders. The first screen should say "Speak. Tinkflow writes." and pair concise value copy with a large, realistic product visualization: a desktop editor receiving dictated text, a bottom listening overlay with a moving waveform, and a small inserted-and-saved result state with copy recovery. The journey should have only three steps: Welcome, Voice setup, Ready. Voice setup should show required local Whisper preparation and optional Ollama polishing in one open, divided workspace. The final screen should center the Ctrl + Space shortcut and show the sequence Listen, Transcribe, Polish, Insert. Motion should communicate state and continuity, not decorate. Keep every line readable, avoid nested cards, avoid gradients and decorative blobs, use familiar icons, and make the experience feel calm, private, capable, and alive.

## Layout Contract

- Default desktop window: 1080 by 720.
- Minimum desktop window: 900 by 620.
- Full-width top bar and footer frame the journey.
- Primary content uses a two-column layout above 760 pixels and a single-column layout below it.
- No onboarding content is wrapped in `.onboarding-card` or any equivalent outer card.
- The product preview is a functional illustration of the app, not stock imagery.