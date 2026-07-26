# Collaboration

This file exists as a root-level collaboration signpost for agents and tools that look for a plain collaboration document.

For the canonical instructions, read these in order:

1. [AGENTS.md](AGENTS.md)
2. [docs/AGENT_COLLABORATION.md](docs/AGENT_COLLABORATION.md)
3. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
4. [docs/GRAPHIFY.md](docs/GRAPHIFY.md)
5. [docs/TESTING.md](docs/TESTING.md)

## Working Agreement

- Check `git status --short` before editing.
- Use Graphify for broad codebase questions when `graphify-out/graph.json` exists.
- Keep model-specific instruction files thin and pointed back to `AGENTS.md`.
- Preserve local-first behavior: Whisper and Ollama are local runtime dependencies.
- Pair frontend IPC changes with Rust command changes.
- Pair settings changes across Rust `AppSettings`, frontend interfaces, defaults, and persistence behavior.
- Pair recording-state changes across Rust emission, `useRecording`, `StatusIndicator`, and `Monitor`.
- Leave a short handoff note with changed files, verification commands, and skipped checks.

## Current Collaboration Docs

- OpenAI/Codex entrypoint: [AGENTS.md](AGENTS.md)
- Claude entrypoint: [CLAUDE.md](CLAUDE.md)
- Gemini entrypoint: [GEMINI.md](GEMINI.md)
- GitHub Copilot entrypoint: [.github/copilot-instructions.md](.github/copilot-instructions.md)

This file is intentionally root-level so agents and tools can discover the collaboration protocol quickly.

    