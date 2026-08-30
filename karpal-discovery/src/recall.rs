// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Token-level recall helpers (0.9.1).
//!
//! 0.9.0's recall matched the *whole* goal as a substring of five curated
//! fields — canonical phrases only; the `summary` field (the richest text
//! each concept has) was never consulted, and plain-language goals from the
//! Knopper practitioner run ("least upper bound join", "commuting two
//! layers") returned zero. These helpers provide vocabulary-level recall:
//! tokenize, normalize with a light stemmer, and count matched query
//! tokens. Whole-phrase substring matching remains the higher-relevance
//! tier in the planner; this tier widens the seed set beneath it.

/// Function words that carry no domain signal.
const STOPWORDS: &[&str] = &[
    "and", "the", "for", "with", "into", "from", "that", "this", "these", "those", "are", "was",
    "were", "not", "but", "any", "all", "its", "one", "over", "via", "can", "how", "what", "when",
    "you", "your", "use", "using",
];

/// Tokenize for recall: lowercase, split on non-alphanumeric, drop tokens
/// shorter than 3 chars, drop stopwords, deduplicate preserving order.
///
/// `"Bidirectional focus: get/put-style access"` yields
/// `["bidirectional", "focus", "get", "put", "style", "access"]`.
#[must_use]
pub fn query_tokens(text: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 3 && !STOPWORDS.contains(t))
        .filter(|t| seen.insert((*t).to_string()))
        .map(|t| t.to_string())
        .collect()
}

/// A deliberately naive, symmetric stemmer: consistent on both query and
/// field sides, which is what matters for matching. Strips a trailing
/// possessive, then one of `s` / `ing` / `ed` when enough remains:
/// `layers → layer`, `bounds → bound`, `commuting → commut`,
/// `focused → focus`.
fn stem(token: &str) -> String {
    let t = token.strip_suffix("'s").unwrap_or(token);
    if let Some(s) = t.strip_suffix('s').filter(|s| s.len() >= 3) {
        return s.to_string();
    }
    if let Some(s) = t.strip_suffix("ing").filter(|s| s.len() >= 4) {
        return s.to_string();
    }
    if let Some(s) = t.strip_suffix("ed").filter(|s| s.len() >= 4) {
        return s.to_string();
    }
    t.to_string()
}

/// Two (stemmed) tokens match when equal, or one is a prefix of the other
/// with at least four stem characters — `commut` ↔ `commute` ✓,
/// `focus` ↔ `focused` ✓, but `put` ↔ `putting` ✗ (stem too short).
fn stems_match(a: &str, b: &str) -> bool {
    let (a, b) = (stem(a), stem(b));
    a == b || (a.len() >= 4 && b.starts_with(&a)) || (b.len() >= 4 && a.starts_with(&b))
}

/// How many distinct query tokens find a match in `text`'s tokens.
#[must_use]
pub fn text_matches(query_tokens: &[String], text: &str) -> usize {
    let text_stems: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(stem)
        .collect();
    query_tokens
        .iter()
        .filter(|q| text_stems.iter().any(|t| stems_match(q, t)))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_splits_punctuation_and_slashes() {
        assert_eq!(
            query_tokens("Bidirectional focus: get/put-style access"),
            vec![
                "bidirectional".to_string(),
                "focus".to_string(),
                "get".to_string(),
                "put".to_string(),
                "style".to_string(),
                "access".to_string()
            ]
        );
    }

    #[test]
    fn tokenization_drops_short_tokens_stopwords_and_duplicates() {
        assert_eq!(
            query_tokens("the get and the put of a get"),
            vec!["get".to_string(), "put".to_string()]
        );
    }

    #[test]
    fn plural_stems_match() {
        assert!(stems_match("layers", "layer"));
        assert!(stems_match("bounds", "bound"));
        assert!(stems_match("commutes", "commuting"));
        assert!(stems_match("focused", "focus"));
    }

    #[test]
    fn prefix_matching_requires_four_stem_characters() {
        assert!(stems_match("commut", "commute"));
        assert!(!stems_match("put", "putting"));
    }

    #[test]
    fn summary_counts_matched_query_tokens() {
        let q = query_tokens("least upper bound join");
        assert_eq!(
            text_matches(
                &q,
                "A poset with binary meet and join — least upper and greatest lower bounds \
                 always exist."
            ),
            4
        );
    }
}
