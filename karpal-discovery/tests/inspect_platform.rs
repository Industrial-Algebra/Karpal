// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for platform-constraint detection — Phase 19-C slice 3.
//! The inspector reads the crate root and records the `std` linkage mode: the
//! key signal for capability *availability* (a `no_std` project cannot use a
//! std-only karpal crate, and vice versa).

use std::path::{Path, PathBuf};

use karpal_discovery::{StdMode, inspect::inspect_workspace};

const ROOT_TOML: &str = "\
[workspace]
members = [\"std-crate\", \"nostd-crate\", \"cond-crate\"]
resolver = \"2\"
";

fn pkg(name: &str) -> String {
    format!(
        "\
[package]
name = \"{name}\"
version = \"0.1.0\"
edition = \"2021\"
"
    )
}

/// Build the platform fixture: three crates with distinct `std` modes.
fn platform_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), ROOT_TOML).unwrap();
    for (name, lib_rs) in [
        ("std-crate", "//! a plain std crate\npub fn x() {}\n"),
        (
            "nostd-crate",
            "//! an unconditional no_std crate\n#![no_std]\n",
        ),
        (
            "cond-crate",
            "//! no_std only when the `std` feature is off\n\
             #![cfg_attr(not(feature = \"std\"), no_std)]\n",
        ),
    ] {
        let crate_dir = dir.path().join(name);
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("Cargo.toml"), pkg(name)).unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), lib_rs).unwrap();
    }
    dir
}

#[test]
fn std_crate_links_std() {
    let dir = platform_fixture();
    let snapshot = inspect_workspace(dir.path());
    assert_eq!(snapshot.crates["std-crate"].platform.std_mode, StdMode::Std);
}

#[test]
fn unconditional_no_std_is_detected() {
    let dir = platform_fixture();
    let snapshot = inspect_workspace(dir.path());
    assert_eq!(
        snapshot.crates["nostd-crate"].platform.std_mode,
        StdMode::NoStd
    );
}

#[test]
fn conditional_no_std_via_cfg_attr_is_detected() {
    let dir = platform_fixture();
    let snapshot = inspect_workspace(dir.path());
    // The karpal house style: `#![cfg_attr(not(feature = "std"), no_std)]`.
    assert_eq!(
        snapshot.crates["cond-crate"].platform.std_mode,
        StdMode::ConditionalNoStd
    );
}

// -- real-workspace assertions --

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn karpal_core_is_conditionally_no_std() {
    let snapshot = inspect_workspace(&workspace_root());
    // karpal-core: `#![cfg_attr(not(feature = "std"), no_std)]` — the no_std gate.
    assert_eq!(
        snapshot.crates["karpal-core"].platform.std_mode,
        StdMode::ConditionalNoStd
    );
}

#[test]
fn karpal_discovery_is_std_only() {
    let snapshot = inspect_workspace(&workspace_root());
    // karpal-discovery is documented as the one std-only crate (no no_std gate).
    assert_eq!(
        snapshot.crates["karpal-discovery"].platform.std_mode,
        StdMode::Std
    );
}
