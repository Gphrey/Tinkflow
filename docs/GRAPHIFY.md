# Graphify

Graphify is set up for this repository in `graphify-out/`.

## Current State

The current graph was built code-only because the installed Graphify CLI requested an LLM API key for docs and images.

Current outputs:

- `graphify-out/graph.json`
- `graphify-out/GRAPH_REPORT.md`
- `graphify-out/.graphify_analysis.json`

The code-only graph is still useful for architecture questions, call relationships, and source navigation.

## Common Commands

Run from the repository root.

```powershell
graphify query "How does the dictation pipeline work?"
graphify query "What depends on SettingsManager?"
graphify query "How does the overlay stay synchronized?"
```

Refresh after code changes:

```powershell
graphify . --code-only --no-viz
graphify cluster-only D:\MIEBI\CodeDocs\Tinkflow --no-viz
```

Generate the browser visualization when needed:

```powershell
graphify export html
```

## Semantic Extraction

To include docs and images in the graph, configure one of the API keys supported by the installed Graphify CLI, then rerun without `--code-only`.

Preferred for this repo:

```powershell
$env:GEMINI_API_KEY="..."
graphify . --no-viz
graphify cluster-only D:\MIEBI\CodeDocs\Tinkflow --no-viz
```

Do not add model API keys to the repository.

## Agent Usage

Agents should use Graphify for broad questions before scanning many files by hand. This installed CLI works best when the query includes exact graph vocabulary such as `SettingsManager`, `WhisperTranscriber`, `OllamaClient`, `AudioCapturer`, `recording-state`, or a file name.

Good queries:

- "How does recording-state flow from Rust to React?"
- "What files change if I add a new recording state?"
- "What connects SettingsManager to audio and hotkeys?"
- "Where is LLM output sanitized?"

For precise implementation work, use Graphify for orientation and then read the source files directly.

