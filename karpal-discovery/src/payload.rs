// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Karpal-discovery typed output payloads for the Lonis `Block` contract.
//!
//! Behind the `lonis` feature. A [`KarpalPayload`] is carried as the `payload`
//! of a `Block<KarpalPayload>`; in-process it is fully typed, and at the
//! subprocess boundary it serializes to `{"kind": "karpal.search", "data": …}`,
//! which the lonis host parses losslessly into `BlockKind::Extension`.
//!
//! This is the first consumer adoption of Lonis's `#[derive(BlockPayload)]`
//! (ADR-0004). The derive makes the two constraints this crate's subprocess
//! spike surfaced into compile-time guarantees:
//!
//! 1. **Adjacent tagging** (`{"kind", "data"}`) — the derive generates the
//!    adjacently-tagged serde impls; an internally-tagged payload can no longer
//!    silently fail at the seam.
//! 2. **Serde tag == `kind_name()`** — both come from one declaration
//!    (`#[lonis_payload(namespace = "karpal")]`), so `karpal.search` cannot
//!    diverge between the wire and in-process.
//!
//! The custom multi-line `render_human` (a query echo plus one line per result)
//! is preserved via the derive's `render_fn` hook — see `render_search`.

#![cfg(feature = "lonis")]

use serde::{Deserialize, Serialize};

use crate::ItemKind;

/// A minimal projection of a catalogued item for search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    /// Item name.
    pub name: String,
    /// Owning crate.
    pub crate_name: String,
    /// Fully-qualified module path.
    pub module_path: String,
    /// Item kind (`"trait"` / `"function"` / `"struct"` / `"enum"` /
    /// `"type_alias"` / `"macro"` / `"const"`). This string flows into
    /// `KarpalPayload::Search.results[].kind` — it is the value set's spec.
    pub kind: String,
}

impl ItemSummary {
    /// Project an [`ItemKind`] plus its identity into a search summary.
    #[must_use]
    pub fn from_record(name: &str, crate_name: &str, module_path: &str, kind: &ItemKind) -> Self {
        let kind_str = match kind {
            ItemKind::Trait(_) => "trait",
            ItemKind::Function(_) => "function",
            ItemKind::Struct(_) => "struct",
            ItemKind::Const(_) => "const",
            ItemKind::Enum(_) => "enum",
            ItemKind::TypeAlias(_) => "type_alias",
            ItemKind::Macro(_) => "macro",
        };
        Self {
            name: name.to_string(),
            crate_name: crate_name.to_string(),
            module_path: module_path.to_string(),
            kind: kind_str.to_string(),
        }
    }
}

/// A minimal projection of an overlay concept for command output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSummary {
    /// Stable concept id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// One-line summary.
    pub summary: String,
    /// Anchoring catalog references (`<crate>::<item>`).
    pub symbol_refs: Vec<String>,
    /// Stability tier (`"stable"` / `"experimental"` / `"research"`).
    pub stability: String,
}

impl ConceptSummary {
    /// Project an overlay [`ConceptRecord`](crate::overlay::ConceptRecord).
    #[must_use]
    pub fn from_record(record: &crate::overlay::ConceptRecord) -> Self {
        Self {
            id: record.id.clone(),
            name: record.name.clone(),
            summary: record.summary.clone(),
            symbol_refs: record.symbol_refs.clone(),
            stability: stability_str(record.stability).to_string(),
        }
    }
}

fn stability_str(tier: crate::overlay::StabilityTier) -> &'static str {
    match tier {
        crate::overlay::StabilityTier::Stable => "stable",
        crate::overlay::StabilityTier::Experimental => "experimental",
        crate::overlay::StabilityTier::Research => "research",
    }
}

/// The typed payload a karpal-discovery tool emits through the Lonis contract.
///
/// Derived via [`lonis_schema::BlockPayload`] — the serde wire tag,
/// `kind_name()`, and `schema_id()` all come from one declaration, so the
/// in-process kind and the host-visible `Extension.kind` (`karpal.search`)
/// cannot diverge. `render_human` delegates to `render_search`.
#[derive(Debug, Clone, lonis_schema::BlockPayload)]
#[lonis_payload(namespace = "karpal", render_fn = "render_search")]
pub enum KarpalPayload {
    /// `karpal search <query>` — public items whose name matches.
    Search {
        /// The query as received.
        query: String,
        /// Matching items.
        results: Vec<ItemSummary>,
    },
    /// `karpal detail` — one catalog item joined with overlay concepts and
    /// the implementation graph.
    Detail {
        /// The item.
        item: ItemSummary,
        /// Doc comment, when catalogued.
        docs: Option<String>,
        /// Types implementing the item (when it is a trait).
        implementors: Vec<String>,
        /// Overlay concepts anchored to this item.
        concepts: Vec<ConceptSummary>,
    },
    /// `karpal concepts` — curated overlay concepts matching a query.
    Concepts {
        /// The query as received (empty matches everything).
        query: String,
        /// Matching concepts.
        results: Vec<ConceptSummary>,
    },
    /// `karpal imports` — which catalog symbols (and hence which concepts)
    /// a workspace actually consumes.
    Imports {
        /// Distinct resolved catalog symbols.
        resolved: usize,
        /// Imports of catalog crates matching no item (drift signal).
        unresolved: usize,
        /// Glob imports of catalog crates.
        globs: usize,
        /// The curated concepts in use.
        concepts: Vec<ConceptSummary>,
    },
}

/// Custom human render for [`KarpalPayload`]: a query echo plus one line per
/// result. Hooked in via `render_fn = "render_search"` so the derive's wire
/// safety is kept without forcing the default `<kind>: <Debug>` render.
#[must_use]
fn render_search(payload: &KarpalPayload) -> String {
    match payload {
        KarpalPayload::Search { query, results } => {
            let mut out = format!("karpal search \"{query}\": {} match(es)\n", results.len());
            for item in results {
                out.push_str(&format!(
                    "  {} ({}) — {}\n",
                    item.name, item.kind, item.module_path
                ));
            }
            out
        }
        KarpalPayload::Detail {
            item,
            docs,
            implementors,
            concepts,
        } => {
            let mut out = format!("{} ({}) — {}\n", item.name, item.kind, item.module_path);
            if let Some(docs) = docs {
                out.push_str(&format!("  docs: {}\n", docs.lines().next().unwrap_or("")));
            }
            if !implementors.is_empty() {
                out.push_str(&format!(
                    "  implemented by {} type(s), e.g. {}\n",
                    implementors.len(),
                    implementors.first().map(String::as_str).unwrap_or_default()
                ));
            }
            if !concepts.is_empty() {
                let ids: Vec<&str> = concepts.iter().map(|c| c.id.as_str()).collect();
                out.push_str(&format!("  concepts: {}\n", ids.join(", ")));
            }
            out
        }
        KarpalPayload::Concepts { query, results } => {
            let mut out = format!("karpal concepts \"{query}\": {} match(es)\n", results.len());
            for concept in results {
                out.push_str(&format!(
                    "  {} [{}] — {}\n",
                    concept.id, concept.stability, concept.summary
                ));
            }
            out
        }
        KarpalPayload::Imports {
            resolved,
            unresolved,
            globs,
            concepts,
        } => {
            let mut out = format!(
                "karpal imports: {resolved} resolved, {unresolved} unresolved, {globs} glob(s)\n"
            );
            let ids: Vec<&str> = concepts.iter().map(|c| c.id.as_str()).collect();
            out.push_str(&format!("  concepts in use: {}\n", ids.join(", ")));
            out
        }
    }
}
