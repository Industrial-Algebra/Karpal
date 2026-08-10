// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Karpal-discovery typed output payloads for the Lonis `Block` contract.
//!
//! Behind the `lonis` feature. A [`KarpalPayload`] is carried as the `payload`
//! of a `Block<KarpalPayload>`; in-process it is fully typed, and at the
//! subprocess boundary it serializes to `{"kind": <name>, "data": <fields>}`,
//! which the lonis host parses losslessly into `BlockKind::Extension`.
//!
//! ## Requirements surfaced (spike)
//!
//! Two Lonis wire constraints a vertical payload enum must satisfy, which
//! Lonis should eventually document or offer a derive for:
//!
//! 1. **Adjacent tagging.** The enum must serialize as `{"kind": …, "data": …}`
//!    to match `BlockKind`'s wire form — hence `#[serde(tag = "kind",
//!    content = "data")]` below. Internally-tagged (`#[serde(tag = "kind")]`)
//!    payloads would *not* round-trip through `BlockKind::Extension`.
//! 2. **Serde tag == `kind_name()`.** The host sees the serde `kind` tag (it
//!    becomes the `Extension.kind` string), *not* `BlockPayload::kind_name()`.
//!    They must agree, and the kind should be namespaced (`karpal_search`) to
//!    avoid collisions across verticals.

#![cfg(feature = "lonis")]

use serde::{Deserialize, Serialize};

use lonis_schema::BlockPayload;

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
    /// `"type_alias"`).
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
            ItemKind::Enum(_) => "enum",
            ItemKind::TypeAlias(_) => "type_alias",
        };
        Self {
            name: name.to_string(),
            crate_name: crate_name.to_string(),
            module_path: module_path.to_string(),
            kind: kind_str.to_string(),
        }
    }
}

/// The typed payload a karpal-discovery tool emits through the Lonis contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum KarpalPayload {
    /// `karpal search <query>` — public items whose name matches.
    #[serde(rename = "karpal_search")]
    Search {
        /// The query as received.
        query: String,
        /// Matching items.
        results: Vec<ItemSummary>,
    },
}

impl BlockPayload for KarpalPayload {
    fn kind_name(&self) -> &str {
        match self {
            Self::Search { .. } => "karpal_search",
        }
    }

    fn schema_id(&self) -> String {
        format!("lonis.block/{}/v1", self.kind_name())
    }

    fn render_human(&self) -> String {
        match self {
            Self::Search { query, results } => {
                let mut out = format!("karpal search \"{query}\": {} match(es)\n", results.len());
                for item in results {
                    out.push_str(&format!(
                        "  {} ({}) — {}\n",
                        item.name, item.kind, item.module_path
                    ));
                }
                out
            }
        }
    }
}
