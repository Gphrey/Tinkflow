//! Developer-focused text replacement dictionary for voice transcription.
//!
//! This module corrects common Whisper mistranscriptions of developer symbols,
//! annotations, operators, and technical terms. It runs **before** the LLM
//! polishing step in the transcription pipeline:
//!
//! ```text
//! Whisper → DeveloperDictionary::apply() → LLM polish → text injection
//! ```
//!
//! Internally, an [Aho-Corasick](https://docs.rs/aho-corasick) automaton is
//! compiled once at construction time, enabling all replacements to execute in
//! a single O(n) pass over the input text with ASCII-case-insensitive matching.

use aho_corasick::{AhoCorasick, MatchKind};

// ─── Pattern Tables ──────────────────────────────────────────────────────────
// Each table is a `&[(spoken, replacement)]` slice. Patterns within each
// category are ordered longest-first so that `LeftmostFirst` match semantics
// always prefer the most specific match.

/// Symbols that Whisper spells out because it cannot produce them directly.
const SYMBOLS: &[(&str, &str)] = &[
    ("at dot", "@."),
    ("at slash", "@/"),
    ("at sign", "@"),
    ("hashtag", "#"),
    ("hash tag", "#"),
    ("dollar sign", "$"),
    ("percent sign", "%"),
    ("ampersand", "&"),
    ("pipe symbol", "|"),
    ("pipe sign", "|"),
    ("backtick", "`"),
    ("back tick", "`"),
    ("tilde", "~"),
];

/// JSDoc / Javadoc-style `@` annotations.
const ANNOTATIONS: &[(&str, &str)] = &[
    ("at deprecated", "@deprecated"),
    ("at override", "@override"),
    ("at returns", "@returns"),
    ("at return", "@return"),
    ("at throws", "@throws"),
    ("at version", "@version"),
    ("at author", "@author"),
    ("at fixme", "@FIXME"),
    ("at param", "@param"),
    ("at since", "@since"),
    ("at agent", "@agent"),
    ("at link", "@link"),
    ("at type", "@type"),
    ("at todo", "@TODO"),
    ("at see", "@see"),
];

// NOTE: Operator replacements ("arrow function" → "=>", "equals equals" → "==")
// were intentionally removed. These phrases are too ambiguous — users often say
// them when *talking about* code, not dictating literal symbols. The LLM would
// then misinterpret the symbols and replace them with unrelated words.

/// Words and acronyms that Whisper commonly mistranscribes.
const MISTRANSCRIPTIONS: &[(&str, &str)] = &[
    ("typescript", "TypeScript"),
    ("javascript", "JavaScript"),
    ("react JS", "React.js"),
    ("next JS", "Next.js"),
    ("node JS", "Node.js"),
    ("view JS", "Vue.js"),
    ("no JS", "Node.js"),
    ("H.T.M.L.", "HTML"),
    ("A.P.I.", "API"),
    ("C.S.S.", "CSS"),
    ("S.Q.L.", "SQL"),
    ("H T T P", "HTTP"),
    ("A P I", "API"),
    ("sequel", "SQL"),
    ("Jason", "JSON"),
    ("jason", "JSON"),
];

/// Standalone "at" before a dot-prefixed path (e.g. "at .agent" → "@.agent").
const AT_DOT: &[(&str, &str)] = &[
    ("at .", "@."),
];

// ─── Public API ──────────────────────────────────────────────────────────────

/// A pre-compiled, single-pass text replacement engine for developer
/// transcription corrections.
///
/// The automaton is built once via [`DeveloperDictionary::new`] and can then be
/// applied to any number of input strings via [`DeveloperDictionary::apply`].
///
/// # Examples
///
/// ```
/// use tinkflow_lib::dictionary::DeveloperDictionary;
///
/// let dict = DeveloperDictionary::new();
/// assert_eq!(dict.apply("parse the Jason file"), "parse the JSON file");
/// assert_eq!(dict.apply("arrow function"), "arrow function"); // preserved as-is
/// ```
#[derive(Debug, Clone)]
pub struct DeveloperDictionary {
    /// Pre-compiled Aho-Corasick automaton over all patterns.
    automaton: AhoCorasick,
    /// Replacement strings, indexed in the same order as the patterns fed
    /// to the automaton builder.
    replacements: Vec<&'static str>,
}

impl Default for DeveloperDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl DeveloperDictionary {
    /// Build a new dictionary by compiling all pattern tables into a single
    /// Aho-Corasick automaton.
    ///
    /// The automaton uses:
    /// - **ASCII case-insensitive** matching (handles both `"Jason"` and `"jason"`).
    /// - **Leftmost-first** semantics so that longer patterns like
    ///   `"equals equals equals"` are preferred over the shorter `"equals equals"`.
    ///
    /// # Panics
    ///
    /// Panics if the Aho-Corasick automaton cannot be built (should never
    /// happen with valid static pattern tables).
    pub fn new() -> Self {
        // Collect all category tables into a single list.
        // Order matters: categories with longer / more specific patterns first.
        let all_tables: &[&[(&str, &str)]] = &[
            ANNOTATIONS,
            SYMBOLS,
            MISTRANSCRIPTIONS,
            AT_DOT,
        ];

        let mut patterns: Vec<&str> = Vec::new();
        let mut replacements: Vec<&'static str> = Vec::new();

        for table in all_tables {
            for &(spoken, replacement) in *table {
                patterns.push(spoken);
                replacements.push(replacement);
            }
        }

        let automaton = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::LeftmostFirst)
            .build(&patterns)
            .expect("failed to build DeveloperDictionary automaton");

        Self {
            automaton,
            replacements,
        }
    }

    /// Apply all dictionary replacements to `text` in a single pass.
    ///
    /// Returns a new `String` with every matching spoken pattern replaced by
    /// its developer-correct form.
    ///
    /// # Examples
    ///
    /// ```
    /// use tinkflow_lib::dictionary::DeveloperDictionary;
    ///
    /// let dict = DeveloperDictionary::new();
    ///
    /// // Symbol correction
    /// assert_eq!(dict.apply("use at sign here"), "use @ here");
    ///
    /// // Case-insensitive mistranscription
    /// assert_eq!(dict.apply("JASON data"), "JSON data");
    ///
    /// // Operators are preserved as natural language
    /// assert_eq!(dict.apply("use an arrow function"), "use an arrow function");
    /// ```
    pub fn apply(&self, text: &str) -> String {
        self.automaton.replace_all(text, &self.replacements)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build the dictionary once for the test module.
    fn dict() -> DeveloperDictionary {
        DeveloperDictionary::new()
    }

    // ── Symbol replacements ──────────────────────────────────────────────

    #[test]
    fn symbol_at_sign() {
        assert_eq!(dict().apply("use at sign here"), "use @ here");
    }

    #[test]
    fn symbol_hashtag() {
        assert_eq!(dict().apply("add a hashtag channel"), "add a # channel");
    }

    #[test]
    fn symbol_at_dot_path() {
        assert_eq!(dict().apply("import at dot components"), "import @. components");
    }

    #[test]
    fn symbol_at_slash() {
        assert_eq!(dict().apply("import at slash utils"), "import @/ utils");
    }

    // ── Annotation replacements ──────────────────────────────────────────

    #[test]
    fn annotation_param() {
        assert_eq!(dict().apply("at param name"), "@param name");
    }

    #[test]
    fn annotation_returns() {
        assert_eq!(dict().apply("at returns string"), "@returns string");
    }

    #[test]
    fn annotation_deprecated() {
        assert_eq!(dict().apply("mark it at deprecated"), "mark it @deprecated");
    }

    // ── Operators are NOT replaced (intentionally) ────────────────────────

    #[test]
    fn operators_preserved_arrow_function() {
        // "arrow function" should NOT become "=>" — it's natural English
        assert_eq!(dict().apply("use an arrow function"), "use an arrow function");
    }

    #[test]
    fn operators_preserved_equals() {
        assert_eq!(dict().apply("if x equals equals y"), "if x equals equals y");
    }

    // ── Mistranscription corrections ─────────────────────────────────────

    #[test]
    fn mistranscription_json() {
        assert_eq!(dict().apply("parse the Jason file"), "parse the JSON file");
    }

    #[test]
    fn mistranscription_json_lowercase() {
        assert_eq!(dict().apply("a jason object"), "a JSON object");
    }

    #[test]
    fn mistranscription_api() {
        assert_eq!(dict().apply("call the A.P.I."), "call the API");
    }

    #[test]
    fn mistranscription_sql_sequel() {
        assert_eq!(dict().apply("write a sequel query"), "write a SQL query");
    }

    #[test]
    fn mistranscription_node_js() {
        assert_eq!(dict().apply("use node JS server"), "use Node.js server");
    }

    #[test]
    fn mistranscription_typescript() {
        assert_eq!(dict().apply("write in typescript"), "write in TypeScript");
    }

    // ── Case insensitivity ───────────────────────────────────────────────

    #[test]
    fn case_insensitive_hashtag() {
        assert_eq!(dict().apply("HASHTAG trending"), "# trending");
    }

    #[test]
    fn case_insensitive_arrow_preserved() {
        assert_eq!(dict().apply("ARROW FUNCTION"), "ARROW FUNCTION");
    }

    // ── Edge cases ───────────────────────────────────────────────────────

    #[test]
    fn empty_input() {
        assert_eq!(dict().apply(""), "");
    }

    #[test]
    fn no_match_passthrough() {
        let input = "I went to the store yesterday";
        assert_eq!(dict().apply(input), input);
    }

    #[test]
    fn multiple_replacements_in_one_string() {
        assert_eq!(
            dict().apply("parse the Jason A.P.I. with an arrow function"),
            "parse the JSON API with an arrow function"
        );
    }

    #[test]
    fn at_dot_standalone() {
        assert_eq!(dict().apply("at .agent"), "@.agent");
    }

    // ── Default trait ────────────────────────────────────────────────────

    #[test]
    fn default_is_same_as_new() {
        let a = DeveloperDictionary::new();
        let b = DeveloperDictionary::default();
        assert_eq!(a.apply("Jason"), b.apply("Jason"));
    }
}
