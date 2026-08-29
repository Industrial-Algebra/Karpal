// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Curation-breadth tests for the concept overlay (Phase 19-B slice 2).
//! The seed must cover the workspace's mathematically significant public API
//! across every crate family, exercise every relation kind, and carry problem
//! shapes on the concepts agents most often search for. The drift gate in
//! `concept_overlay.rs` re-validates every symbol_ref against the live
//! workspace, so breadth here cannot outrun reality.

use karpal_discovery::overlay::{ConceptOverlay, RelationKind, load_concept_overlay};

#[test]
fn seed_covers_each_crate_family() {
    let overlay = load_concept_overlay();
    let expected = [
        // karpal-core: functor family and the rest of the hierarchy
        "selective",
        "extend",
        "comonad",
        "foldable",
        "traversable",
        "alt",
        "plus",
        "alternative",
        "functor-filter",
        "bifunctor",
        "contravariant",
        "invariant",
        "divide",
        "decide",
        "divisible",
        "conclude",
        "adjunction",
        "dinatural-transformation",
        "comonad-transformers",
        // karpal-profunctor
        "profunctor",
        "strong",
        "choice",
        "traversing",
        // karpal-topos
        "small-category",
        "presheaf",
        "representable-functor",
        "sieve",
        "grothendieck-topology",
        "sheaf",
        "subobject-classifier",
        "finite-limits",
        "yoneda-lemma",
        // karpal-free
        "free-monad",
        "cofree-comonad",
        "free-applicative",
        "free-alternative",
        "freer-monad",
        "yoneda-embedding",
        "day-convolution",
        "codensity-monad",
        "kan-extension",
        // karpal-diagram
        "string-diagram",
        "monoidal-structure",
        "trace",
        "coherence",
        // karpal-arrow
        "semigroupoid",
        "category",
        "arrow",
        "arrow-choice",
        "arrow-loop",
        "arrow-apply",
        "arrow-zero",
        "arrow-plus",
        // karpal-recursion
        "recursion-schemes",
        // karpal-effect
        "monad-transformers",
        "reader-transformer",
        "state-transformer",
        "writer-transformer",
        "except-transformer",
        "st-bridges",
        // karpal-optics
        "optic",
        // karpal-higher
        "two-category",
        "bicategory",
        "enriched-category",
        "ffunctor",
        // karpal-algebra
        "semiring",
        "ring",
        "field",
        "group",
        "abelian-group",
        "module",
        "vector-space",
        "lattice",
        "bounded-lattice",
        "heyting-algebra",
    ];
    let have: Vec<&str> = overlay.concepts.iter().map(|c| c.id.as_str()).collect();
    for id in expected {
        assert!(have.contains(&id), "seed is missing curated concept `{id}`");
    }
}

#[test]
fn seed_relations_exercise_every_relation_kind() {
    let overlay = load_concept_overlay();
    let kinds: Vec<RelationKind> = overlay.relations.iter().map(|r| r.kind).collect();
    for kind in [
        RelationKind::Generalizes,
        RelationKind::ComposesWith,
        RelationKind::AlternativeTo,
        RelationKind::DualOf,
    ] {
        assert!(
            kinds.contains(&kind),
            "seed should exercise relation kind {kind:?}"
        );
    }
}

#[test]
fn seed_relations_cover_every_concept() {
    // Every curated concept should participate in the relationship graph —
    // an island concept is likely an under-curated one.
    let overlay = load_concept_overlay();
    let connected: std::collections::BTreeSet<&str> = overlay
        .relations
        .iter()
        .flat_map(|r| [r.from.as_str(), r.to.as_str()])
        .collect();
    for concept in &overlay.concepts {
        assert!(
            connected.contains(concept.id.as_str()),
            "concept `{}` participates in no relation",
            concept.id
        );
    }
}

#[test]
fn key_concepts_carry_problem_shapes() {
    let overlay: ConceptOverlay = load_concept_overlay();
    let key = [
        "functor",
        "applicative",
        "monad",
        "foldable",
        "traversable",
        "alternative",
        "reader-transformer",
        "state-transformer",
        "writer-transformer",
        "except-transformer",
    ];
    for id in key {
        let record = overlay
            .concepts
            .iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("concept `{id}` missing"));
        assert!(
            !record.problem_shapes.is_empty(),
            "`{id}` should carry at least one problem shape"
        );
    }
}

#[test]
fn every_symbol_ref_is_qualified() {
    // All curated refs use `<crate>::<item>`; a bare name would silently
    // weaken resolution.
    let overlay = load_concept_overlay();
    for concept in &overlay.concepts {
        for symbol_ref in &concept.symbol_refs {
            assert!(
                symbol_ref.contains("::"),
                "concept `{}` has unqualified symbol_ref `{symbol_ref}`",
                concept.id
            );
        }
    }
}

/// The overlay's `catalog_version` records the Karpal release it was
/// curated against — it must track the workspace version exactly (the
/// Rabbit Hole found it stamped `0.9.0-dev` while the workspace was at
/// 0.9.0: the one field named "version" was decorative). The bump PR
/// that forgets to restamp fails here.
#[test]
fn catalog_version_tracks_the_workspace_version() {
    let overlay = karpal_discovery::load_concept_overlay();
    assert_eq!(
        overlay.catalog_version,
        env!("CARGO_PKG_VERSION"),
        "overlay catalog_version must equal the crate (workspace) version — \
         restamp data/concepts.toml on every version bump"
    );
}
