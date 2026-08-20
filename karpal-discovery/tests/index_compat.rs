// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! `karpal-index` compatibility-mode tests (Phase 19-H migration). The
//! `karpal` binary accepts `--index-compat` followed by the legacy argv
//! (`search <q> | detail <name> | crates | hierarchy <trait>`, `--json`),
//! emitting `karpal-index`'s JSON shapes over the new catalog so existing
//! invocations migrate without changing their parsing.
//!
//! Documented divergences from the legacy binary (byte-parity is not the
//! goal; field-set parity is): `path` is the module path (no `:line`
//! suffix), `summary` is the first doc sentence where docs exist, and
//! `subtraits` is empty (the legacy indexer never populated it either —
//! parity).

#![cfg(feature = "lonis")]

use std::path::{Path, PathBuf};
use std::process::Command;

fn karpal_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_karpal"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

fn compat(args: &[&str]) -> String {
    let mut argv = vec!["--index-compat"];
    argv.extend_from_slice(args);
    let out = Command::new(karpal_bin())
        .args(&argv)
        .current_dir(workspace_root())
        .output()
        .expect("run karpal");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const API_ITEM_FIELDS: [&str; 12] = [
    "name",
    "kind",
    "crate_name",
    "path",
    "signature",
    "docs",
    "summary",
    "supertraits",
    "subtraits",
    "methods",
    "implementors",
    "trait_impls",
];

#[test]
fn compat_search_emits_api_items_json() {
    let out = compat(&["search", "Functor", "--json"]);
    let items: Vec<serde_json::Value> = serde_json::from_str(out.trim()).expect("JSON array");
    assert!(!items.is_empty(), "Functor matches: {out}");
    for item in &items {
        let obj = item.as_object().expect("item is an object");
        for field in API_ITEM_FIELDS {
            assert!(
                obj.contains_key(field),
                "field `{field}` present (karpal-index contract): {}",
                serde_json::to_string(item).unwrap()
            );
        }
        let kind = item["kind"].as_str().expect("kind is a string");
        assert!(
            [
                "trait",
                "function",
                "struct",
                "enum",
                "type_alias",
                "macro",
                "const"
            ]
            .contains(&kind),
            "kind from the legacy value set: {kind}"
        );
    }
    // case-insensitive substring on names, sorted by name (legacy semantics)
    let names: Vec<&str> = items.iter().filter_map(|i| i["name"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "sorted by name");
    assert!(names.iter().all(|n| n.to_lowercase().contains("functor")));
    // items resolve in the real workspace
    assert!(names.contains(&"Functor"), "{names:?}");
}

#[test]
fn compat_search_is_case_insensitive_substring() {
    let out = compat(&["search", "monad", "--json"]);
    let items: Vec<serde_json::Value> = serde_json::from_str(out.trim()).expect("JSON array");
    let names: Vec<&str> = items.iter().filter_map(|i| i["name"].as_str()).collect();
    assert!(names.contains(&"Monad"), "{names:?}");
    assert!(
        names.iter().any(|n| n.to_lowercase().contains("monad")),
        "{names:?}"
    );
}

#[test]
fn compat_detail_reports_a_trait_with_supertraits() {
    let out = compat(&["detail", "Applicative", "--json"]);
    let item: serde_json::Value = serde_json::from_str(out.trim()).expect("JSON object");
    assert_eq!(item["name"], "Applicative");
    assert_eq!(item["crate_name"], "karpal-core");
    assert_eq!(item["kind"], "trait");
    let supertraits: Vec<&str> = item["supertraits"]
        .as_array()
        .expect("supertraits array")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(
        supertraits.contains(&"Apply"),
        "Applicative : Apply: {supertraits:?}"
    );
    // implementors come from the catalog's impl graph
    let implementors: Vec<&str> = item["implementors"]
        .as_array()
        .expect("implementors array")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(!implementors.is_empty(), "karpal implements Applicative");
}

#[test]
fn compat_detail_unknown_name_is_null() {
    let out = compat(&["detail", "NoSuchItem", "--json"]);
    assert_eq!(out.trim(), "null", "legacy not-found JSON is null");
}

#[test]
fn compat_crates_lists_workspace_crates() {
    let out = compat(&["crates", "--json"]);
    let crates: Vec<serde_json::Value> = serde_json::from_str(out.trim()).expect("JSON array");
    let names: Vec<&str> = crates.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"karpal-core"), "{names:?}");
    assert!(names.contains(&"karpal-discovery"), "{names:?}");
    for c in &crates {
        for field in ["name", "items", "description"] {
            assert!(c.as_object().is_some_and(|o| o.contains_key(field)), "{c}");
        }
    }
}

#[test]
fn compat_hierarchy_reports_supertraits_and_implementors() {
    let out = compat(&["hierarchy", "Monad", "--json"]);
    let h: serde_json::Value = serde_json::from_str(out.trim()).expect("JSON object");
    for field in ["name", "kind", "supertraits", "subtraits", "implementors"] {
        assert!(h.as_object().is_some_and(|o| o.contains_key(field)), "{h}");
    }
    let supers: Vec<&str> = h["supertraits"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(
        supers.contains(&"Applicative") && supers.contains(&"Chain"),
        "{supers:?}"
    );
}

#[test]
fn compat_human_mode_search_prints_rows() {
    let out = compat(&["search", "Functor"]);
    assert!(
        out.lines().any(|l| l.contains("Functor")),
        "human rows carry the item name: {out}"
    );
}
