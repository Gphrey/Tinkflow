# Tinkflow Research — Table of Contents

> **Created:** February 15, 2026  
> **Purpose:** Voice-to-Text tool research and product design for Tinkflow

---

## Documents

| # | Document | Description |
|---|---|---|
| 01 | [WisprFlow Deep Dive](./01_wisperflow_deep_dive.md) | Complete analysis of WisprFlow — features, architecture, pricing, UX patterns, strengths/weaknesses |
| 02 | [Competitor Landscape](./02_competitor_landscape.md) | 15+ competitor profiles across categories — premium, local/privacy-first, open-source, OS built-ins — with comparison matrix |
| 03 | [Opportunities & Insights](./03_opportunities_and_insights.md) | Market gaps, architecture recommendations, feature prioritization, positioning options, revenue models, risks |
| 04 | [Tinkflow Design Document](./04_tinkflow_design.md) | ⭐ **Validated product design** — architecture, smart context detection, features, tech stack, project structure, UX, model strategy |
| 05 | [Skills Guide](./05_skills_guide.md) | 🧭 **Development guide** — maps 20 relevant skills to Tinkflow modules and dev phases |

---

## Key Decisions (from Brainstorming)

| Decision | Choice |
|---|---|
| **Target audience** | Developers (expandable later) |
| **Privacy model** | Local-first — all processing on-device |
| **Activation** | Hold-to-talk hotkey |
| **AI cleanup** | Local LLM post-processing (via Ollama) |
| **Priority workflows** | IDEs + Communication tools |
| **LLM integration** | Smart detect Ollama → fallback to bundled model |
| **Context handling** | Unified smart detection (not rigid modes) |
| **Tech stack** | Tauri 2.x (Rust + React/TypeScript) |

---

## 🧭 Skills Quick Reference

> **Full guide:** [05_skills_guide.md](./05_skills_guide.md)

| Working On | Invoke These Skills |
|---|---|
| Audio / Whisper / VAD | `voice-ai-development`, `voice-agents` |
| Ollama / LLM prompts | `llm-app-patterns`, `prompt-engineering` |
| React UI / TypeScript | `typescript-expert`, `react-best-practices`, `frontend-design` |
| Architecture decisions | `architecture`, `software-architecture` |
| SQLite / Dictionary | `database-design` |
| Bugs / Issues | `systematic-debugging` |
| Task planning | `plan-writing`, `brainstorming` |
