// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Integration test: extract the real Karpal workspace and assert the
//! extractor finds well-known public traits with the expected shape. Asserts
//! only on stable, long-standing symbols so this is not brittle to ordinary
//! API growth.

use std::path::Path;

use karpal_discovery::{ItemKind, extract::extract_workspace};

fn workspace_root() -> std::path::PathBuf {
    // `karpal-discovery/` lives directly under the workspace root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn catalogues_the_real_karpal_workspace() {
    let catalog = extract_workspace(&workspace_root());
    assert!(
        catalog.crate_count() >= 10,
        "expected many member crates, got {}",
        catalog.crate_count()
    );
    assert!(
        catalog.crates.contains_key("karpal-core"),
        "karpal-core catalogued"
    );
}

#[test]
fn functor_trait_has_expected_shape() {
    let catalog = extract_workspace(&workspace_root());
    let functor = catalog.find_item("Functor").expect("Functor is catalogued");
    assert_eq!(functor.crate_name, "karpal-core");
    let ItemKind::Trait(tr) = &functor.kind else {
        panic!("Functor is a trait");
    };
    // Karpal's HKT encoding: `Functor: HKT` and the `Of` associated type lives
    // on the `HKT` supertrait, so `Functor` itself has no associated items.
    assert!(
        tr.supertraits.contains(&"HKT".to_string()),
        "Functor's supertrait is HKT: {tr:?}"
    );
    // `fmap` is the canonical required method.
    assert!(
        tr.methods.iter().any(|m| m.name == "fmap"),
        "Functor has fmap: {:?}",
        tr.methods
    );
}
