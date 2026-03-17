//! Ollama LLM client for context-aware transcription polishing.
//!
//! This module sends Whisper-transcribed (and dictionary-corrected) text to a
//! local [Ollama](https://ollama.ai) instance for grammar cleanup, filler-word
//! removal, and punctuation. It sits at the end of the transcription pipeline:
//!
//! ```text
//! Whisper → DeveloperDictionary → OllamaClient::polish_text() → text injection
//! ```
//!
//! The prompt strategy is designed to be **model-agnostic** — it uses XML-style
//! delimiters and few-shot examples that work across model families (phi3, llama3,
//! gemma, mistral, etc.), combined with a hardened output sanitiser that strips
//! common LLM artifacts regardless of source.

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ─── Request / Response Types ────────────────────────────────────────────────

/// Body for Ollama `/api/generate`.
#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

/// Parsed response from Ollama `/api/generate`.
#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

/// Parsed response from Ollama `/api/tags`.
#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

/// A single model entry from `/api/tags`.
#[derive(Deserialize)]
struct ModelInfo {
    name: String,
}

// ─── Client ──────────────────────────────────────────────────────────────────

/// HTTP client for the local Ollama API.
///
/// Wraps a blocking `reqwest::Client` with a configurable base URL
/// (defaults to `http://localhost:11434`).
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaClient {
    /// Create a new client targeting `http://localhost:11434`.
    ///
    /// The underlying HTTP client has a 30-second default timeout.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client for OllamaClient");

        Self {
            client,
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Check if Ollama is running and reachable (3-second timeout).
    pub fn check_health(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// List all locally available model names from Ollama.
    pub fn list_models(&self) -> Result<Vec<String>, String> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        let tags: TagsResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Polish raw transcription text using the specified Ollama model.
    ///
    /// Uses XML-delimited few-shot examples that work across model families,
    /// combined with a context-aware system prompt. Falls back to the original
    /// text if the LLM fails or produces unusable output.
    ///
    /// # Arguments
    ///
    /// * `raw_text` — the dictionary-corrected transcription
    /// * `context` — detected window context (`"code"`, `"chat"`, `"email"`, etc.)
    /// * `model_name` — the Ollama model to use (e.g. `"phi3:mini"`)
    pub fn polish_text(&self, raw_text: &str, context: &str, model_name: &str) -> String {
        if model_name.is_empty() {
            return raw_text.to_string();
        }

        let system_prompt = build_system_prompt(context);

        // Few-shot examples teach the cleaning behaviour without exposing realistic
        // developer phrases that small models might parrot back verbatim.
        // The 3-digit IDs make each example clearly artificial.
        // The "NOW:" separator hard-anchors the start of the real task.
        let user_prompt = format!(
            r#"<example id="1">
<input>uh so like example one two three filler words here</input>
<output>Example one two three.</output>
</example>

<example id="2">
<input>hey um can you pass me the uh the config file please thanks</input>
<output>Can you pass me the config file please?</output>
</example>

<example id="3">
<input>what is the uh best way to handle errors in rust</input>
<output>What is the best way to handle errors in Rust?</output>
</example>

<example id="4">
<input>use the at-param decorator for validation</input>
<output>Use the @param decorator for validation.</output>
</example>

<example id="5">
<input>I think we need to see LLM also like an addition for it to be able to do a bit of grammar and spelling check so that when the final words come out it comes out really close</input>
<output>I think we need the LLM to also do grammar and spelling checks, so the final output comes out really close to what was intended.</output>
</example>

<example id="6">
<input>you know we start with pick up anything and then think some things to mean words</input>
<output>We start by picking up everything, and then some things end up as the wrong words.</output>
</example>

<example id="7">
<input>it does not get the context wrong wherever the words that have been spoken is being used it will not be published</input>
<output>It should not get the context wrong. Wherever the spoken words are used, they should be accurate before being inserted.</output>
</example>

NOW: Clean up the following input text. Fix any misheard words using context.
<input>{}</input>
<output>"#,
            raw_text
        );

        let request = GenerateRequest {
            model: model_name.to_string(),
            prompt: user_prompt,
            system: system_prompt,
            stream: false,
        };

        let result = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .timeout(Duration::from_secs(15))
            .json(&request)
            .send();

        match result {
            Ok(resp) => {
                if let Ok(gen_resp) = resp.json::<GenerateResponse>() {
                    let polished = sanitise_llm_output(&gen_resp.response, raw_text);
                    if polished.is_empty() {
                        raw_text.to_string()
                    } else {
                        println!("LLM polished: {}", polished);
                        polished
                    }
                } else {
                    eprintln!("LLM: Failed to parse response, using raw text");
                    raw_text.to_string()
                }
            }
            Err(e) => {
                eprintln!("LLM: Request failed ({}), using raw text", e);
                raw_text.to_string()
            }
        }
    }

    /// Pull (download) a model via Ollama's streaming `/api/pull` endpoint.
    ///
    /// Emits `"ollama-download-progress"` events (0.0–100.0) to the frontend.
    ///
    /// # Errors
    ///
    /// Returns an error if the pull request fails, the stream ends without a
    /// `"success"` status, or reading from the stream fails.
    pub fn pull_model(
        &self,
        model_name: &str,
        app_handle: &tauri::AppHandle,
        cancel_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), String> {
        #[derive(Serialize)]
        struct PullRequest {
            name: String,
            stream: bool,
        }

        #[derive(Deserialize, Debug)]
        struct PullResponse {
            status: String,
            #[allow(dead_code)]
            digest: Option<String>,
            total: Option<u64>,
            completed: Option<u64>,
        }

        let request = PullRequest {
            name: model_name.to_string(),
            stream: true, // Stream to prevent massive blocking/timeouts
        };

        let mut response = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .timeout(Duration::from_secs(1800)) // 30 min max for large models
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to initiate pull: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned status {}", response.status()));
        }

        use std::io::{BufRead, BufReader};
        use tauri::Emitter;

        let reader = BufReader::new(&mut response);

        for line in reader.lines() {
            // Check for cancellation on every NDJSON line
            if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = app_handle.emit("ollama-download-progress", -1.0_f64);
                println!("[Ollama] Pull cancelled by user.");
                return Err("cancelled".to_string());
            }

            let line = line.map_err(|e| format!("Error reading stream: {}", e))?;
            if line.is_empty() {
                continue;
            }

            if let Ok(json_resp) = serde_json::from_str::<PullResponse>(&line) {
                if let (Some(total), Some(completed)) = (json_resp.total, json_resp.completed) {
                    if total > 0 {
                        let progress = (completed as f64 / total as f64) * 100.0;
                        let _ = app_handle.emit("ollama-download-progress", progress);
                    }
                }

                if json_resp.status == "success" {
                    println!("Successfully pulled model: {}", model_name);
                    return Ok(());
                }
            }
        }

        Err(format!(
            "Pull stream ended without success status for model: {}",
            model_name
        ))
    }
}

// ─── Prompt Building ─────────────────────────────────────────────────────────

/// Build a context-aware system prompt for the LLM.
///
/// Kept intentionally **short and directive** — small models like phi3:mini
/// parrot long instructions back verbatim. The few-shot examples in
/// [`OllamaClient::polish_text`] do the real teaching.
fn build_system_prompt(context: &str) -> String {
    let context_hint = match context {
        "code" => "Preserve technical terms, function names, and code symbols exactly.",
        "comment" => "Write clean professional English suitable for code comments.",
        "chat" => "Keep the tone casual and concise.",
        "email" => "Use professional tone and grammar.",
        "terminal" => "Keep output concise and command-like.",
        _ => "Use correct grammar and natural phrasing.",
    };

    format!(
        "You are a transcription corrector. The input is speech-to-text output that may contain: \
         (1) filler words (uh, um, like, so, you know) — remove them. \
         (2) Misheard or wrong words from speech recognition — fix them using surrounding context \
         (e.g. \"think\" should be \"thing\", \"their\" should be \"there\"). \
         (3) Grammar and punctuation errors — correct them. \
         (4) Awkward phrasing from spoken language — make it read naturally. \
         Preserve all code symbols (@, #, =>, ===) and technical terms exactly as they appear. \
         Do NOT replace symbols with English words. Do NOT change the meaning or add new ideas. \
         Output ONLY the cleaned text inside <output></output> tags. \
         No explanations. No commentary. {}",
        context_hint
    )
}

// ─── Output Sanitisation ─────────────────────────────────────────────────────

/// Sanitise raw LLM output into clean, usable text.
///
/// This function is designed to be **model-agnostic** — it handles quirks from
/// small models (echoed prefixes, commentary) and large models (markdown fences,
/// verbose preambles) alike.
///
/// Falls back to `original` if the sanitised output is empty or has a
/// suspicious length ratio (hallucination / over-summarisation guard).
fn sanitise_llm_output(raw: &str, original: &str) -> String {
    let mut text = raw.trim().to_string();

    // 1. Extract from <output> tags if present
    if let Some(start) = text.find("<output>") {
        let content_start = start + "<output>".len();
        if let Some(end) = text[content_start..].find("</output>") {
            text = text[content_start..content_start + end].trim().to_string();
        } else {
            // Opening tag but no closing tag — take everything after it
            text = text[content_start..].trim().to_string();
        }
    }

    // 2. Strip common echoed prefixes
    for prefix in &[
        "OUTPUT:", "Output:", "output:", "Here is", "Here's", "Sure,",
    ] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.trim().to_string();
        }
    }

    // 3. Strip markdown code fences
    if text.starts_with("```") {
        text = text
            .lines()
            .skip(1) // skip opening ```
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
    }

    // 4. Strip surrounding quotes (safe, no byte-index panic)
    if let Some(unquoted) = text.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        text = unquoted.to_string();
    }

    // 5. Take only the first line (commentary guard)
    if text.contains('\n') {
        if let Some(first_line) = text.lines().next() {
            let trimmed = first_line.trim().to_string();
            if !trimmed.is_empty() {
                text = trimmed;
            }
        }
    }

    // 6. Length guard — reject hallucinated or over-summarised output
    if !original.is_empty() {
        let ratio = text.len() as f64 / original.len() as f64;
        if ratio > 3.0 || ratio < 0.3 {
            eprintln!(
                "LLM output length ratio {:.1}x — discarding (original: {} chars, output: {} chars)",
                ratio,
                original.len(),
                text.len()
            );
            return original.to_string();
        }
    }

    text
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── sanitise_llm_output ──────────────────────────────────────────────

    #[test]
    fn sanitise_clean_output() {
        assert_eq!(
            sanitise_llm_output("Hello world.", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_extracts_from_output_tags() {
        assert_eq!(
            sanitise_llm_output("<output>Hello world.</output>", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_extracts_unclosed_output_tag() {
        assert_eq!(
            sanitise_llm_output("<output>Hello world.", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_strips_echoed_prefix() {
        assert_eq!(
            sanitise_llm_output("OUTPUT: Hello world.", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_strips_quotes() {
        assert_eq!(
            sanitise_llm_output("\"Hello world.\"", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_strips_commentary() {
        assert_eq!(
            sanitise_llm_output(
                "Hello world.\nI cleaned up your text by removing filler words.",
                "hello world"
            ),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_strips_code_fence() {
        assert_eq!(
            sanitise_llm_output("```\nHello world.\n```", "hello world"),
            "Hello world."
        );
    }

    #[test]
    fn sanitise_rejects_hallucination() {
        let original = "hi";
        let hallucinated = "Hello! I'm here to help you with your coding tasks today. Let me know what you need assistance with and I'll do my best.";
        assert_eq!(sanitise_llm_output(hallucinated, original), original);
    }

    #[test]
    fn sanitise_rejects_over_summarisation() {
        let original = "I was thinking that maybe we should refactor the entire database module to use the new ORM";
        let too_short = "ok";
        assert_eq!(sanitise_llm_output(too_short, original), original);
    }

    #[test]
    fn sanitise_empty_llm_output_falls_back() {
        // Empty output triggers the length guard → falls back to original
        assert_eq!(sanitise_llm_output("", "hello world"), "hello world");
    }

    // ── build_system_prompt ──────────────────────────────────────────────

    #[test]
    fn system_prompt_contains_context_hint() {
        let prompt = build_system_prompt("code");
        assert!(prompt.contains("Preserve technical terms"));
    }

    #[test]
    fn system_prompt_uses_xml_instruction() {
        let prompt = build_system_prompt("code");
        assert!(prompt.contains("<output></output>"));
    }

    #[test]
    fn system_prompt_fallback_context() {
        let prompt = build_system_prompt("unknown_context");
        assert!(prompt.contains("correct grammar"));
    }

    // ── OllamaClient construction ────────────────────────────────────────

    #[test]
    fn default_matches_new() {
        let a = OllamaClient::new();
        let b = OllamaClient::default();
        assert_eq!(a.base_url, b.base_url);
    }
}
