// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for the curated concept overlay — Phase 19-B slice 1.
//! The overlay layers human/math-meaningful concept names and relationships
//! over the structural catalog (19-A), and a validator enforces that every
//! overlay reference resolves to a real catalog item (the ROADMAP's "CI
//! rejects overlays referencing missing symbols" drift gate).

use std::path::{Path, PathBuf};

use karpal_discovery::{
    extract::extract_workspace,
    overlay::{
        ConceptOverlay, ConceptRecord, CostHint, OverlayDrift, RelationKind, StabilityTier,
        load_concept_overlay,
    },
};

/// A minimal catalog fixture: one crate exposing the named public traits.
fn catalog_with(
    crate_name: &str,
    traits: &[&str],
) -> (tempfile::TempDir, karpal_discovery::Catalog) {
    let dir = tempfile::tempdir().unwrap();
    let crate_dir = dir.path().join(crate_name);
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    let toml = format!(
        "\
[package]
name = \"{crate_name}\"
version = \"0.1.0\"
edition = \"2021\"
"
    );
    std::fs::write(crate_dir.join("Cargo.toml"), toml).unwrap();
    let lib: String = traits
        .iter()
        .map(|t| format!("pub trait {t} {{}}\n"))
        .collect();
    std::fs::write(crate_dir.join("src/lib.rs"), lib).unwrap();
    let catalog = extract_workspace(dir.path());
    (dir, catalog)
}

fn one_concept(id: &str, symbol_ref: &str) -> ConceptRecord {
    ConceptRecord {
        id: id.to_string(),
        name: id.to_string(),
        summary: "fixture".to_string(),
        aliases: Vec::new(),
        math_concepts: Vec::new(),
        symbol_refs: vec![symbol_ref.to_string()],
        stability: StabilityTier::Stable,
        cost: CostHint::Free,
    }
}

#[test]
fn embedded_overlay_loads_the_seed_concepts() {
    let overlay = load_concept_overlay();
    let ids: Vec<&str> = overlay.concepts.iter().map(|c| c.id.as_str()).collect();
    assert!(
        ids.contains(&"functor"),
        "seed must include functor: {ids:?}"
    );
    assert!(ids.contains(&"monad"), "seed must include monad: {ids:?}");
    assert!(
        !overlay.concepts.is_empty(),
        "embedded overlay should carry a non-empty seed"
    );
}

#[test]
fn valid_overlay_validates_against_the_catalog() {
    let (_dir, catalog) = catalog_with("demo", &["Functor", "Monad"]);
    let overlay = ConceptOverlay {
        catalog_version: "0.1.0".to_string(),
        concepts: vec![one_concept("functor", "demo::Functor")],
        relations: Vec::new(),
    };
    assert!(overlay.validate(&catalog).is_ok());
}

#[test]
fn missing_symbol_ref_is_drift() {
    let (_dir, catalog) = catalog_with("demo", &["Functor"]);
    let overlay = ConceptOverlay {
        catalog_version: "0.1.0".to_string(),
        concepts: vec![one_concept("functor", "demo::Bogus")],
        relations: Vec::new(),
    };
    let drift = overlay.validate(&catalog).expect_err("should drift");
    assert!(
        drift.iter().any(|d| matches!(
            d,
            OverlayDrift::MissingSymbol { symbol_ref, .. } if symbol_ref == "demo::Bogus"
        )),
        "expected a MissingSymbol drift for demo::Bogus, got {drift:?}"
    );
}

#[test]
fn missing_crate_is_drift() {
    let (_dir, catalog) = catalog_with("demo", &["Functor"]);
    let overlay = ConceptOverlay {
        catalog_version: "0.1.0".to_string(),
        concepts: vec![one_concept("functor", "ghost-crate::Functor")],
        relations: Vec::new(),
    };
    assert!(overlay.validate(&catalog).is_err());
}

#[test]
fn dangling_relation_endpoint_is_drift() {
    let (_dir, catalog) = catalog_with("demo", &["Functor", "Applicative"]);
    let overlay = ConceptOverlay {
        catalog_version: "0.1.0".to_string(),
        concepts: vec![
            one_concept("functor", "demo::Functor"),
            one_concept("applicative", "demo::Applicative"),
        ],
        relations: vec![karpal_discovery::overlay::ConceptRelation {
            from: "applicative".to_string(),
            to: "nonexistent".to_string(),
            kind: RelationKind::Generalizes,
        }],
    };
    let drift = overlay.validate(&catalog).expect_err("should drift");
    assert!(
        drift
            .iter()
            .any(|d| matches!(d, OverlayDrift::DanglingRelation { .. })),
        "expected a DanglingRelation drift, got {drift:?}"
    );
}

// -- the CI drift gate: the embedded seed must resolve against the real
//    Karpal workspace --

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn embedded_seed_validates_against_the_real_workspace() {
    let overlay = load_concept_overlay();
    let catalog = extract_workspace(&workspace_root());
    overlay
        .validate(&catalog)
        .expect("the curated seed must resolve against the real Karpal catalog");
}
