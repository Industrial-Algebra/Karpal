// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for conventional (auto-detected) target discovery —
//! Phase 19-C slice 2. Cargo infers targets from filesystem layout by
//! convention (`src/lib.rs` → lib, `src/main.rs` + `src/bin/*` → bins,
//! `examples/*`, `tests/*`, `benches/*`); this slice makes the inspector
//! infer the same, deduplicated against explicit targets and gated by the
//! `autobins`/`autoexamples`/`autotests`/`autobenches` flags.

use std::path::{Path, PathBuf};

use karpal_discovery::inspect::inspect_workspace;

const ROOT_TOML: &str = "\
[workspace]
members = [\"auto-demo\", \"dedup-demo\", \"flags-demo\"]
resolver = \"2\"
";

const AUTO_DEMO_TOML: &str = "\
[package]
name = \"auto-demo\"
version = \"0.1.0\"
edition = \"2021\"
";

const DEDUP_DEMO_TOML: &str = "\
[package]
name = \"dedup-demo\"
version = \"0.1.0\"
edition = \"2021\"

[[bin]]
name = \"cli\"
path = \"src/main.rs\"
";

const FLAGS_DEMO_TOML: &str = "\
[package]
name = \"flags-demo\"
version = \"0.1.0\"
edition = \"2021\"
autobins = false
autoexamples = false
";

/// Build the auto-target fixture workspace (source files are empty — only
/// their existence drives discovery).
fn auto_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), ROOT_TOML).unwrap();
    // auto-demo: a full conventional layout, no explicit targets.
    let demo = dir.path().join("auto-demo");
    std::fs::create_dir_all(demo.join("src/bin/multi")).unwrap();
    std::fs::create_dir_all(demo.join("examples/multimain")).unwrap();
    std::fs::create_dir_all(demo.join("tests")).unwrap();
    std::fs::create_dir_all(demo.join("benches")).unwrap();
    for rel in [
        "src/lib.rs",
        "src/main.rs",
        "src/bin/extra.rs",
        "src/bin/multi/main.rs",
        "examples/demo.rs",
        "examples/multimain/main.rs",
        "tests/integ.rs",
        "benches/speed.rs",
    ] {
        std::fs::write(demo.join(rel), "").unwrap();
    }
    std::fs::write(demo.join("Cargo.toml"), AUTO_DEMO_TOML).unwrap();

    // dedup-demo: an explicit bin claiming src/main.rs, plus a src/bin file.
    let dedup = dir.path().join("dedup-demo");
    std::fs::create_dir_all(dedup.join("src/bin")).unwrap();
    std::fs::write(dedup.join("src/main.rs"), "").unwrap();
    std::fs::write(dedup.join("src/bin/other.rs"), "").unwrap();
    std::fs::write(dedup.join("Cargo.toml"), DEDUP_DEMO_TOML).unwrap();

    // flags-demo: autobins/autoexamples off, but tests/ still auto-discovered.
    let flags = dir.path().join("flags-demo");
    std::fs::create_dir_all(flags.join("src/bin")).unwrap();
    std::fs::create_dir_all(flags.join("examples")).unwrap();
    std::fs::create_dir_all(flags.join("tests")).unwrap();
    for rel in ["src/main.rs", "src/bin/x.rs", "examples/d.rs", "tests/t.rs"] {
        std::fs::write(flags.join(rel), "").unwrap();
    }
    std::fs::write(flags.join("Cargo.toml"), FLAGS_DEMO_TOML).unwrap();

    dir
}

fn names(targets: &[karpal_discovery::NamedTarget]) -> Vec<&str> {
    targets.iter().map(|t| t.name.as_str()).collect()
}

#[test]
fn auto_lib_is_inferred_from_src_lib_rs() {
    let dir = auto_fixture();
    let snapshot = inspect_workspace(dir.path());
    let lib = snapshot
        .crates
        .get("auto-demo")
        .unwrap()
        .targets
        .lib
        .as_ref()
        .expect("auto lib inferred");
    assert_eq!(lib.name, "auto_demo"); // package name with `-` → `_`
    assert!(lib.auto_discovered);
}

#[test]
fn auto_bins_covers_main_and_src_bin_and_subdir_main() {
    let dir = auto_fixture();
    let snapshot = inspect_workspace(dir.path());
    let bins = &snapshot.crates["auto-demo"].targets.bins;
    // src/main.rs → auto_demo; src/bin/extra.rs → extra; src/bin/multi/main.rs → multi.
    let mut got: Vec<&str> = bins.iter().map(|b| b.name.as_str()).collect();
    got.sort_unstable();
    assert_eq!(got, vec!["auto_demo", "extra", "multi"]);
    assert!(bins.iter().all(|b| b.auto_discovered));
}

#[test]
fn auto_examples_tests_and_benches_are_inferred() {
    let dir = auto_fixture();
    let snapshot = inspect_workspace(dir.path());
    let t = &snapshot.crates["auto-demo"].targets;
    assert_eq!(names(&t.examples), vec!["demo", "multimain"]);
    assert_eq!(names(&t.integration_tests), vec!["integ"]);
    assert_eq!(names(&t.benches), vec!["speed"]);
}

#[test]
fn explicit_bin_claiming_src_main_rs_is_not_duplicated_as_auto() {
    let dir = auto_fixture();
    let snapshot = inspect_workspace(dir.path());
    let bins = &snapshot.crates["dedup-demo"].targets.bins;
    // cli is explicit (claims src/main.rs); other is auto-discovered; no
    // phantom "dedup_demo" auto-main bin since src/main.rs is already taken.
    let cli = bins
        .iter()
        .find(|b| b.name == "cli")
        .expect("explicit cli bin");
    assert!(!cli.auto_discovered);
    let other = bins
        .iter()
        .find(|b| b.name == "other")
        .expect("auto other bin");
    assert!(other.auto_discovered);
    assert!(
        !bins.iter().any(|b| b.name == "dedup_demo"),
        "src/main.rs must not yield a second auto bin when an explicit bin claims it"
    );
    assert_eq!(bins.len(), 2);
}

#[test]
fn auto_flags_disable_bin_and_example_discovery_but_not_tests() {
    let dir = auto_fixture();
    let snapshot = inspect_workspace(dir.path());
    let t = &snapshot.crates["flags-demo"].targets;
    assert!(
        t.bins.is_empty(),
        "autobins=false suppresses src/main.rs and src/bin/*"
    );
    assert!(
        t.examples.is_empty(),
        "autoexamples=false suppresses examples/*"
    );
    // autotests defaults true → tests/t.rs still discovered.
    assert_eq!(names(&t.integration_tests), vec!["t"]);
}

// -- real-workspace assertion --

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn karpal_core_lib_is_auto_discovered() {
    let snapshot = inspect_workspace(&workspace_root());
    let lib = snapshot
        .crates
        .get("karpal-core")
        .unwrap()
        .targets
        .lib
        .as_ref()
        .expect("karpal-core has a lib target");
    // karpal-core declares no explicit [lib] → the lib is conventionally
    // inferred from src/lib.rs.
    assert!(lib.auto_discovered);
    assert_eq!(lib.name, "karpal_core");
}
