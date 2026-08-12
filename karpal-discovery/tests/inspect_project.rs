// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for the project inspector — Phase 19-C slice 1, a
//! read-only Cargo.toml metadata extractor producing a [`ProjectSnapshot`].
//!
//! The inspector parses `Cargo.toml` with a real TOML parser (no `cargo`
//! spawn, no mutation) and captures workspace meta, per-crate package
//! metadata, dependencies (with source discrimination), features, and
//! explicit targets. It is the structural foundation for the future `inspect`
//! command and the planner's project-context input.

use std::path::{Path, PathBuf};

use karpal_discovery::{DepKind, DepSource, ProjectSnapshot, inspect::inspect_workspace};

const ROOT_TOML: &str = "\
[workspace]
members = [\"crate-a\", \"crate-b\"]
resolver = \"2\"
";

const CRATE_A_TOML: &str = "\
[package]
name = \"crate-a\"
version = \"0.1.0\"
edition = \"2021\"
rust-version = \"1.70\"
description = \"A fixture crate.\"
license = \"Apache-2.0\"

[dependencies]
serde = { version = \"1.0\", features = [\"derive\"] }
crate-b = { path = \"../crate-b\", default-features = false }
optional-dep = { version = \"2.0\", optional = true }

[dev-dependencies]
tempfile = \"3\"

[build-dependencies]
anyhow = \"1\"

[features]
default = [\"foo\"]
foo = [\"dep:serde\"]
bar = []

[lib]
name = \"crate_a\"
path = \"src/lib.rs\"

[[bin]]
name = \"cli\"
path = \"src/main.rs\"

[[example]]
name = \"demo\"

[[test]]
name = \"integration\"

[[bench]]
name = \"throughput\"
";

const CRATE_B_TOML: &str = "\
[package]
name = \"crate-b\"
version = \"0.2.0\"
edition = \"2021\"

[dependencies]
toml = \"1\"
";

fn fixture_snapshot() -> (tempfile::TempDir, ProjectSnapshot) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), ROOT_TOML).unwrap();
    for (sub, toml) in [("crate-a", CRATE_A_TOML), ("crate-b", CRATE_B_TOML)] {
        let crate_dir = dir.path().join(sub);
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(crate_dir.join("Cargo.toml"), toml).unwrap();
    }
    let snapshot = inspect_workspace(dir.path());
    (dir, snapshot)
}

/// Fetch a crate snapshot, panicking clearly if absent.
fn crate_of<'a>(snapshot: &'a ProjectSnapshot, name: &str) -> &'a karpal_discovery::CrateSnapshot {
    snapshot
        .crates
        .get(name)
        .unwrap_or_else(|| panic!("crate `{name}` should be in the snapshot"))
}

#[test]
fn virtual_workspace_meta_is_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let workspace = snapshot.workspace.as_ref().expect("workspace meta present");
    assert_eq!(workspace.members, vec!["crate-a", "crate-b"]);
    assert_eq!(workspace.resolver.as_deref(), Some("2"));
}

#[test]
fn both_member_crates_are_snapshotted() {
    let (_dir, snapshot) = fixture_snapshot();
    assert!(snapshot.crates.contains_key("crate-a"));
    assert!(snapshot.crates.contains_key("crate-b"));
}

#[test]
fn package_metadata_is_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let pkg = &crate_of(&snapshot, "crate-a").package;
    assert_eq!(pkg.name, "crate-a");
    assert_eq!(pkg.version, "0.1.0");
    assert_eq!(pkg.edition.as_deref(), Some("2021"));
    assert_eq!(pkg.rust_version.as_deref(), Some("1.70"));
    assert_eq!(pkg.license.as_deref(), Some("Apache-2.0"));
}

#[test]
fn registry_dependency_is_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let deps = &crate_of(&snapshot, "crate-a").dependencies;
    let serde = deps.get("serde").expect("serde dep");
    assert_eq!(serde.kind, DepKind::Normal);
    assert!(!serde.optional);
    assert!(serde.default_features);
    assert_eq!(serde.features, vec!["derive"]);
    match &serde.source {
        DepSource::Registry { version_req } => assert_eq!(version_req, "1.0"),
        other => panic!("serde source is {other:?}, expected Registry"),
    }
}

#[test]
fn path_dependency_and_default_features_false_are_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let deps = &crate_of(&snapshot, "crate-a").dependencies;
    let crate_b = deps.get("crate-b").expect("crate-b path dep");
    assert_eq!(crate_b.kind, DepKind::Normal);
    assert!(!crate_b.default_features);
    match &crate_b.source {
        DepSource::Path { rel_path } => assert_eq!(rel_path, "../crate-b"),
        other => panic!("crate-b source is {other:?}, expected Path"),
    }
}

#[test]
fn optional_registry_dependency_is_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let deps = &crate_of(&snapshot, "crate-a").dependencies;
    let opt = deps.get("optional-dep").expect("optional-dep");
    assert!(opt.optional);
    assert!(matches!(
        &opt.source,
        DepSource::Registry { version_req } if version_req == "2.0"
    ));
}

#[test]
fn dev_and_build_dependencies_are_classified() {
    let (_dir, snapshot) = fixture_snapshot();
    let deps = &crate_of(&snapshot, "crate-a").dependencies;
    let tempfile = deps.get("tempfile").expect("tempfile dev-dep");
    assert_eq!(tempfile.kind, DepKind::Dev);
    let anyhow = deps.get("anyhow").expect("anyhow build-dep");
    assert_eq!(anyhow.kind, DepKind::Build);
}

#[test]
fn features_are_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let features = &crate_of(&snapshot, "crate-a").features;
    assert_eq!(features.get("default"), Some(&vec!["foo".to_string()]));
    assert_eq!(features.get("foo"), Some(&vec!["dep:serde".to_string()]));
    assert_eq!(features.get("bar"), Some(&vec![]));
}

#[test]
fn targets_are_captured() {
    let (_dir, snapshot) = fixture_snapshot();
    let targets = &crate_of(&snapshot, "crate-a").targets;
    let lib = targets.lib.as_ref().expect("lib target");
    assert_eq!(lib.name, "crate_a");
    assert_eq!(lib.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(targets.bins.len(), 1);
    assert_eq!(targets.bins[0].name, "cli");
    assert_eq!(targets.examples.len(), 1);
    assert_eq!(targets.examples[0].name, "demo");
    assert_eq!(targets.integration_tests.len(), 1);
    assert_eq!(targets.integration_tests[0].name, "integration");
    assert_eq!(targets.benches.len(), 1);
    assert_eq!(targets.benches[0].name, "throughput");
}

#[test]
fn empty_target_kinds_default_to_empty() {
    let (_dir, snapshot) = fixture_snapshot();
    let targets = &crate_of(&snapshot, "crate-b").targets;
    // crate-b declares no targets explicitly — all collections empty.
    assert!(targets.lib.is_none());
    assert!(targets.bins.is_empty());
}

#[test]
fn content_hash_is_stable_and_nonempty() {
    let (dir, snapshot_a) = fixture_snapshot();
    let hash_a = &snapshot_a.content_hash;
    assert!(!hash_a.is_empty());
    let snapshot_b = inspect_workspace(dir.path());
    assert_eq!(&snapshot_b.content_hash, hash_a);
}

// -- real-workspace assertions (the actual Karpal workspace) --

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn karpal_workspace_is_virtual() {
    let snapshot = inspect_workspace(&workspace_root());
    let workspace = snapshot
        .workspace
        .as_ref()
        .expect("Karpal has workspace meta");
    assert!(workspace.members.contains(&"karpal-discovery".to_string()));
    assert_eq!(workspace.resolver.as_deref(), Some("3"));
}

#[test]
fn karpal_discovery_git_deps_are_classified() {
    let snapshot = inspect_workspace(&workspace_root());
    let deps = &crate_of(&snapshot, "karpal-discovery").dependencies;
    // lonis-schema is a git dep pinned to a rev — the pre-publication pattern.
    let lonis = deps.get("lonis-schema").expect("lonis-schema dep");
    assert!(lonis.optional);
    assert!(lonis.features.iter().any(|f| f == "derive"));
    match &lonis.source {
        DepSource::Git { url, rev, .. } => {
            assert!(url.contains("Industrial-Algebra/Lonis"));
            assert!(rev.as_ref().is_some_and(|r| r.starts_with("4aa23a6")));
        }
        other => panic!("lonis-schema source is {other:?}, expected Git"),
    }
    // A registry dep for contrast.
    let syn = deps.get("syn").expect("syn dep");
    assert!(matches!(syn.source, DepSource::Registry { .. }));
}

#[test]
fn karpal_discovery_feature_and_binary_target() {
    let snapshot = inspect_workspace(&workspace_root());
    let kd = crate_of(&snapshot, "karpal-discovery");
    assert!(kd.features.contains_key("lonis"));
    assert!(kd.targets.bins.iter().any(|b| b.name == "karpal"));
    // karpal-index is an explicit binary crate.
    let ki = crate_of(&snapshot, "karpal-index");
    assert!(ki.targets.bins.iter().any(|b| b.name == "karpal-index"));
}
