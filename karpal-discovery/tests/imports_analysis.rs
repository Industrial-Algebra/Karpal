// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for imported-symbol analysis — the deferred 19-B/19-C
//! slice. `analyze_imports` parses a target project's `use` statements and
//! resolves them against a structural catalog (the karpal API surface),
//! reporting which catalog symbols the project actually consumes, which
//! catalog-crate imports fail to resolve, and which are glob imports. The
//! overlay join then maps consumed symbols to curated mathematical concepts.

use std::path::Path;

use karpal_discovery::imports::{ResolvedImport, analyze_imports};
use karpal_discovery::overlay::load_concept_overlay;

/// A catalog fixture: one crate exposing the named public traits (plus a
/// nested module trait when requested via `m::Nested` style tests).
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
    let catalog = karpal_discovery::extract::extract_workspace(dir.path());
    (dir, catalog)
}

/// A target project fixture: writes the given sources under `src/`.
fn target_project(sources: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (file, source) in sources {
        let path = dir.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    dir
}

fn resolved_of<'a>(
    report: &'a karpal_discovery::imports::ImportsReport,
    symbol_ref: &str,
) -> &'a ResolvedImport {
    report
        .resolved
        .iter()
        .find(|r| r.symbol_ref == symbol_ref)
        .unwrap_or_else(|| panic!("`{symbol_ref}` should be resolved: {:?}", report.resolved))
}

// The real tests, against a fixture crate whose package name maps from an
// underscored use path (`karpal-core` ↔ `karpal_core`).

fn fixture(target_sources: &[(&str, &str)]) -> karpal_discovery::imports::ImportsReport {
    let (_cat_dir, catalog) = catalog_with("karpal-core", &["Functor", "Apply", "Applicative"]);
    let target = target_project(target_sources);
    analyze_imports(target.path(), &catalog)
}

#[test]
fn simple_use_resolves() {
    let report = fixture(&[("src/lib.rs", "use karpal_core::Functor;\n")]);
    let r = resolved_of(&report, "karpal-core::Functor");
    assert_eq!(r.local_name, "Functor");
    assert_eq!(r.occurrences, 1);
    assert_eq!(r.files, 1);
    assert!(matches!(
        r.kind,
        Some(karpal_discovery::catalog::ItemKind::Trait { .. })
    ));
}

#[test]
fn grouped_and_renamed_imports_resolve() {
    let report = fixture(&[(
        "src/lib.rs",
        "use karpal_core::{Apply, Applicative as Ap};\n",
    )]);
    assert_eq!(
        resolved_of(&report, "karpal-core::Apply").local_name,
        "Apply"
    );
    assert_eq!(
        resolved_of(&report, "karpal-core::Applicative").local_name,
        "Ap"
    );
}

#[test]
fn module_qualified_paths_resolve_and_bare_modules_are_not_noise() {
    // Re-export tolerance: the leaf ident resolves regardless of intermediate
    // module segments; a bare module import is not an unresolved item.
    let (_cat_dir, _catalog) = catalog_with("karpal-core", &[]);
    // a real file module (as real projects use), not an inline one
    let crate_src = _cat_dir.path().join("karpal-core/src");
    std::fs::write(crate_src.join("lib.rs"), "pub mod functor;\n").unwrap();
    std::fs::write(crate_src.join("functor.rs"), "pub trait Nested {}\n").unwrap();
    let catalog = karpal_discovery::extract::extract_workspace(_cat_dir.path());

    let target = target_project(&[
        ("src/lib.rs", "use karpal_core::functor::Nested;\n"),
        ("src/other.rs", "use karpal_core::functor;\n"),
    ]);
    let report = analyze_imports(target.path(), &catalog);
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-core::Nested"),
        "module-qualified leaf resolves: {:?}",
        report.resolved
    );
    assert!(
        report.unresolved.is_empty(),
        "bare module import is not unresolved noise: {:?}",
        report.unresolved
    );
}

#[test]
fn glob_imports_are_recorded_not_expanded() {
    let report = fixture(&[("src/lib.rs", "use karpal_core::*;\n")]);
    assert_eq!(report.globs.len(), 1, "one glob entry: {:?}", report.globs);
    assert!(report.globs[0].path.ends_with("karpal_core::*"));
    assert!(report.resolved.is_empty());
    assert!(report.unresolved.is_empty());
}

#[test]
fn unresolved_catalog_imports_are_reported() {
    let report = fixture(&[("src/lib.rs", "use karpal_core::Bogus;\n")]);
    assert_eq!(report.unresolved.len(), 1, "{:?}", report.unresolved);
    assert_eq!(report.unresolved[0].path, "karpal_core::Bogus");
    assert!(report.resolved.is_empty());
}

#[test]
fn external_crates_are_ignored() {
    let report = fixture(&[(
        "src/lib.rs",
        "use std::collections::HashMap;\nuse serde::Serialize;\n",
    )]);
    assert!(report.resolved.is_empty());
    assert!(report.unresolved.is_empty());
    assert!(report.globs.is_empty());
}

#[test]
fn counts_aggregate_across_files() {
    let report = fixture(&[
        (
            "src/lib.rs",
            "use karpal_core::Functor;\nuse karpal_core::Functor as F;\n",
        ),
        ("src/other.rs", "use karpal_core::Functor;\n"),
    ]);
    let r = resolved_of(&report, "karpal-core::Functor");
    assert_eq!(r.occurrences, 3);
    assert_eq!(r.files, 2);
    // the bare name is preferred over aliases when both occur
    assert_eq!(r.local_name, "Functor");
}

#[test]
fn overlay_join_maps_symbols_to_concepts() {
    let report = fixture(&[("src/lib.rs", "use karpal_core::{Functor, Apply};\n")]);
    let overlay = load_concept_overlay();
    let concepts = report.concepts_used(&overlay);
    let ids: Vec<&str> = concepts.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"functor"), "{ids:?}");
    assert!(ids.contains(&"apply"), "{ids:?}");
    // direct symbol lookup also works
    assert_eq!(
        overlay
            .concept_for_symbol("karpal-core::Functor")
            .map(|c| c.id.clone()),
        Some("functor".to_string())
    );
}

#[test]
fn use_statements_inside_functions_are_collected() {
    let report = fixture(&[("src/lib.rs", "fn f() { use karpal_core::Functor; }\n")]);
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-core::Functor")
    );
}

// -- the real workspace: karpal consumes its own catalog --

#[test]
fn karpal_workspace_imports_resolve_against_its_own_catalog() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves");
    let catalog = karpal_discovery::extract::extract_workspace(&root);
    let report = analyze_imports(&root, &catalog);
    // karpal-effect/src/classes.rs: `use karpal_core::functor::Functor;`
    // (module-qualified — proves leaf tolerance on real code)
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-core::Functor"),
        "Functor should resolve: {:?}",
        &report.resolved[..report.resolved.len().min(5)]
    );
    assert!(
        report
            .resolved
            .iter()
            .any(|r| r.symbol_ref == "karpal-core::HKT")
    );
    // and the overlay join yields real concepts from real usage
    let overlay = load_concept_overlay();
    let ids: Vec<String> = report
        .concepts_used(&overlay)
        .iter()
        .map(|c| c.id.clone())
        .collect();
    assert!(ids.contains(&"functor".to_string()), "{ids:?}");
    assert!(ids.contains(&"applicative".to_string()), "{ids:?}");
}
