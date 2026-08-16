// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The curated concept overlay (Phase 19-B) — semantic metadata layered over
//! the generated structural catalog (19-A).
//!
//! Where the structural catalog records *what exists* (traits, functions,
//! types, …), the overlay records *what it means*: mathematical concept names,
//! aliases for search, problem shapes, and directed relationships (a concept
//! generalizes / composes with / is an alternative to another). The overlay is
//! hand-curated and ships embedded in the crate (`include_str!` of
//! [`data/concepts.toml`][`crate`]), so a crates.io install needs no source
//! checkout (per the ROADMAP §"Hybrid catalog").
//!
//! The defining guarantee is the **drift gate**: [`ConceptOverlay::validate`]
//! cross-references every `symbol_ref` and relation endpoint against a live
//! [`Catalog`]. CI runs it against the workspace so an overlay referencing a
//! renamed or removed symbol fails loudly instead of silently describing a
//! structure that no longer exists.
//!
//! Slice 1 established the model, the embedded loader, and the validator;
//! slice 2 expanded curation to the workspace's mathematically significant
//! public API (83 concepts across every crate, with problem shapes and
//! `generalizes`/`composes_with`/`alternative_to`/`dual_of` relations).
//! Later slices add recommended probes and cost refinement.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;

/// The embedded curated overlay (checked in at `data/concepts.toml`).
const CONCEPTS_TOML: &str = include_str!("../data/concepts.toml");

/// The curated overlay: a version tag, the concept records, and the directed
/// relationships between them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptOverlay {
    /// The Karpal release the overlay was curated against (informational; not
    /// enforced by [`validate`](Self::validate)).
    pub catalog_version: String,
    /// Curated concepts.
    pub concepts: Vec<ConceptRecord>,
    /// Directed relationships between concept `id`s.
    #[serde(default)]
    pub relations: Vec<ConceptRelation>,
}

/// One curated concept: a named mathematical/software idea anchored to
/// structural items via `symbol_refs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRecord {
    /// Stable machine identifier (referenced by [`ConceptRelation`] endpoints).
    pub id: String,
    /// Concise display name.
    pub name: String,
    /// Human-readable purpose / problem shape.
    pub summary: String,
    /// Alternative names used in search.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Mathematical / software concepts associated with this idea.
    #[serde(default)]
    pub math_concepts: Vec<String>,
    /// Problem shapes this concept answers — phrased the way a user or agent
    /// would state the problem (e.g. "sequence dependent effectful steps").
    #[serde(default)]
    pub problem_shapes: Vec<String>,
    /// Referenced structural items, `<crate>::<item>` (must resolve in the
    /// catalog).
    pub symbol_refs: Vec<String>,
    /// API stability tier.
    pub stability: StabilityTier,
    /// Expected relative runtime / integration cost.
    pub cost: CostHint,
}

/// A directed relationship between two concepts (by `id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelation {
    /// Source concept `id`.
    pub from: String,
    /// Target concept `id`.
    pub to: String,
    /// The relationship kind.
    pub kind: RelationKind,
}

/// API stability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityTier {
    /// Stable public API suitable for production use.
    Stable,
    /// Public API that may evolve during the current release series.
    Experimental,
    /// Research-facing capability with intentionally limited guarantees.
    Research,
}

/// Relative execution / integration cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostHint {
    /// Allocation-free, constant-time — the cheapest tier.
    Free,
    /// Cheap but not free (small allocations, linear scans).
    Low,
    /// Moderate work (non-trivial allocation, sub-quadratic).
    Moderate,
    /// Potentially expensive (verification, search, large allocation).
    High,
}

/// The kind of a directed concept relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    /// The source concept is a specialization / subtype of the target
    /// (e.g. `applicative` generalizes `functor`).
    Generalizes,
    /// The two concepts compose in a recognized pattern.
    ComposesWith,
    /// The source is an alternative to the target for a given problem shape.
    AlternativeTo,
    /// The source is the categorical dual of the target (e.g. `comonad` is
    /// the dual of `monad`).
    DualOf,
}

/// A drift finding from [`ConceptOverlay::validate`] — an overlay reference
/// that does not resolve against the structural catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayDrift {
    /// A `symbol_ref` (`<crate>::<item>`) names a crate or item absent from
    /// the catalog.
    MissingSymbol {
        /// The concept `id` carrying the dangling reference.
        concept: String,
        /// The unresolved `<crate>::<item>`.
        symbol_ref: String,
    },
    /// A relation endpoint is not a known concept `id`.
    DanglingRelation {
        /// The source `id`.
        from: String,
        /// The target `id`.
        to: String,
        /// The (now-dangling) relationship kind.
        kind: RelationKind,
    },
    /// A concept carries no `symbol_refs` at all — it floats free of the
    /// structural catalog.
    Unanchored {
        /// The unanchored concept `id`.
        concept: String,
    },
    /// Two concepts share an `id`, making relation endpoints ambiguous.
    DuplicateId {
        /// The duplicated `id`.
        id: String,
    },
}

impl ConceptOverlay {
    /// Validate the overlay against a structural catalog: every `symbol_ref`
    /// must resolve to a catalog item, and every relation endpoint must be a
    /// known concept `id`. Returns `Ok(())` if the overlay is consistent, or
    /// the collected drift otherwise. This is the ROADMAP's "CI rejects
    /// overlays referencing missing symbols" gate.
    pub fn validate(&self, catalog: &Catalog) -> Result<(), Vec<OverlayDrift>> {
        let mut drift = Vec::new();
        let mut seen_ids = BTreeSet::new();
        let known_ids: BTreeSet<&str> = self.concepts.iter().map(|c| c.id.as_str()).collect();

        for concept in &self.concepts {
            if !seen_ids.insert(concept.id.as_str()) {
                drift.push(OverlayDrift::DuplicateId {
                    id: concept.id.clone(),
                });
            }
            if concept.symbol_refs.is_empty() {
                drift.push(OverlayDrift::Unanchored {
                    concept: concept.id.clone(),
                });
            }
            for symbol_ref in &concept.symbol_refs {
                if !symbol_resolves(catalog, symbol_ref) {
                    drift.push(OverlayDrift::MissingSymbol {
                        concept: concept.id.clone(),
                        symbol_ref: symbol_ref.clone(),
                    });
                }
            }
        }
        for rel in &self.relations {
            if !known_ids.contains(rel.from.as_str()) || !known_ids.contains(rel.to.as_str()) {
                drift.push(OverlayDrift::DanglingRelation {
                    from: rel.from.clone(),
                    to: rel.to.clone(),
                    kind: rel.kind,
                });
            }
        }

        if drift.is_empty() { Ok(()) } else { Err(drift) }
    }
}

/// Resolve a `<crate>::<item>` symbol ref against the catalog.
fn symbol_resolves(catalog: &Catalog, symbol_ref: &str) -> bool {
    let Some((crate_name, item_name)) = symbol_ref.split_once("::") else {
        return false;
    };
    catalog
        .crates
        .get(crate_name)
        .is_some_and(|record| record.items.iter().any(|item| item.name == item_name))
}

/// Load the embedded curated concept overlay.
///
/// The data ships in the crate via `include_str!`, so this never performs I/O
/// and a crates.io install needs no source checkout. A failure here is a
/// packaging defect (the checked-in `data/concepts.toml` is malformed).
#[must_use]
pub fn load_concept_overlay() -> ConceptOverlay {
    toml::from_str(CONCEPTS_TOML).expect("embedded concepts.toml is valid curated data")
}
