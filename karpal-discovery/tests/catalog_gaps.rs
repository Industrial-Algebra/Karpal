// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Catalog-gap tests (Phase 19-D/E follow-up): the imports drift signal
//! surfaced three structural blind spots in the 19-A catalog, all fixed
//! here —
//!
//! 1. **Consts** — `pub const` items were not catalogued at all
//!    (`DEFAULT_REPORT_STEM` and friends).
//! 2. **Derive names** — a `#[proc_macro_derive(Name)] pub fn derive_name`
//!    was catalogued under the (never importable) fn name; the public API
//!    name is the derive name.
//! 3. **Re-exports** — `pub use` re-exports (cross-crate and intra-crate,
//!    including renames) were invisible; `use karpal_verify::AmariX` style
//!    imports resolved to nothing.

use std::path::Path;

use karpal_discovery::catalog::{ItemKind, MacroFlavor};
use karpal_discovery::extract::extract_workspace;

/// Fixture: one crate with the given `src/lib.rs` content.
fn crate_with(name: &str, lib: &str) -> (tempfile::TempDir, karpal_discovery::Catalog) {
    let dir = tempfile::tempdir().unwrap();
    let crate_dir = dir.path().join(name);
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), lib).unwrap();
    let catalog = extract_workspace(dir.path());
    (dir, catalog)
}

fn items_of<'a>(
    catalog: &'a karpal_discovery::Catalog,
    krate: &str,
) -> &'a [karpal_discovery::catalog::ItemRecord] {
    &catalog
        .crates
        .get(krate)
        .unwrap_or_else(|| panic!("crate `{krate}` in catalog"))
        .items
}

#[test]
fn consts_are_catalogued() {
    let (_dir, catalog) = crate_with(
        "consts",
        "pub const STEM: &str = \"report\";\npub const N: usize = 3;\n",
    );
    let items = items_of(&catalog, "consts");
    let stem = items
        .iter()
        .find(|i| i.name == "STEM")
        .expect("STEM catalogued");
    assert!(matches!(stem.kind, ItemKind::Const(_)));
}

#[test]
fn derive_macros_are_catalogued_under_the_derive_name() {
    // The fn name is an implementation detail — proc-macro crates export
    // only the derive name, so that is the catalog's public-surface name.
    let (_dir, catalog) = crate_with(
        "derives",
        "#[proc_macro_derive(VerifySemigroup, attributes(verify))]\npub fn derive_verify_semigroup(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }\n",
    );
    let items = items_of(&catalog, "derives");
    assert!(
        items.iter().any(|i| i.name == "VerifySemigroup"),
        "derive name catalogued: {:?}",
        items.iter().map(|i| i.name.as_str()).collect::<Vec<_>>()
    );
    assert!(
        !items.iter().any(|i| i.name == "derive_verify_semigroup"),
        "fn name is not the importable name"
    );
    let record = items
        .iter()
        .find(|i| i.name == "VerifySemigroup")
        .expect("VerifySemigroup item");
    match &record.kind {
        ItemKind::Macro(m) => {
            assert_eq!(m.flavor, MacroFlavor::Derive);
            assert_eq!(m.derives.as_deref(), Some("VerifySemigroup"));
        }
        other => panic!("expected Macro, got {other:?}"),
    }
}

#[test]
fn pub_use_reexports_are_recorded_with_renames() {
    let (_dir, catalog) = crate_with(
        "reexporter",
        "pub struct MonteCarloVerifier { pub p: f64 }\npub use self::MonteCarloVerifier as AmariMonteCarloVerifier;\npub use other_crate::{Alpha, Beta as B};\npub use std::collections::HashMap;\npub use globby::*;\n",
    );
    let record = catalog.crates.get("reexporter").expect("crate");
    let names: Vec<(&str, &str)> = record
        .reexports
        .iter()
        .map(|r| (r.name.as_str(), r.origin.as_str()))
        .collect();
    // rename recorded with its origin
    assert!(
        names.contains(&("AmariMonteCarloVerifier", "self::MonteCarloVerifier")),
        "{names:?}"
    );
    // grouped cross-crate re-exports, leaf + rename
    assert!(
        names
            .iter()
            .any(|(n, o)| *n == "Alpha" && o.contains("other_crate")),
        "{names:?}"
    );
    assert!(names.iter().any(|(n, _)| *n == "B"), "{names:?}");
    // std re-exports and globs are skipped (noise for catalog resolution)
    assert!(!names.iter().any(|(n, _)| *n == "HashMap"), "{names:?}");
    assert!(
        record.reexports.iter().all(|r| !r.origin.ends_with("*")),
        "{names:?}"
    );
}

#[test]
fn imports_resolve_through_reexport_renames() {
    // The motivating real-workspace shape: an intra-crate rename re-export
    // (`MonteCarloVerifier as AmariMonteCarloVerifier`) plus a cross-crate
    // re-export (`pub use karpal_proof_derive::VerifySemigroup`).
    let dir = tempfile::tempdir().unwrap();
    for (name, lib) in [
        (
            "karpal-verify",
            "pub struct MonteCarloVerifier { pub p: f64 }\npub use self::MonteCarloVerifier as AmariMonteCarloVerifier;\n",
        ),
        (
            "karpal-proof-derive",
            "#[proc_macro_derive(VerifySemigroup, attributes(verify))]\npub fn derive_verify_semigroup(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }\n",
        ),
        (
            "karpal-proof",
            "pub use karpal_proof_derive::VerifySemigroup;\n",
        ),
    ] {
        let crate_dir = dir.path().join(name);
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), lib).unwrap();
    }
    let catalog = extract_workspace(dir.path());

    // A consumer project importing through both re-export shapes.
    let target = tempfile::tempdir().unwrap();
    std::fs::write(
        target.path().join("lib.rs"),
        "use karpal_verify::AmariMonteCarloVerifier;\nuse karpal_proof::VerifySemigroup;\n",
    )
    .unwrap();
    let report = karpal_discovery::analyze_imports(Path::new(target.path()), &catalog);
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-verify::MonteCarloVerifier"),
        "rename re-export resolves to the defining item: {:?}",
        report.resolved
    );
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-proof-derive::VerifySemigroup"),
        "cross-crate re-export resolves to the defining crate: {:?}",
        report.resolved
    );
    assert!(report.unresolved.is_empty(), "{:?}", report.unresolved);
}
