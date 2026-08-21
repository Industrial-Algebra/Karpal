// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Publish-order drift gate (Phase 19-H, hardened after the 0.9.0 run):
//! every workspace member must appear in `publish.yml`'s
//! `cargo publish -p <crate>` sequence (coverage), and the sequence must
//! respect the workspace dependency graph — every `karpal-*` path dependency
//! publishes before its dependent (ordering). The 0.9.0 run published
//! `karpal-schubert-types` before its dependency `karpal-higher`; a
//! redundant duplicate step masked as recovery. Coverage alone would not
//! have caught it; ordering does.

use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

fn workspace_members() -> Vec<String> {
    let toml =
        std::fs::read_to_string(workspace_root().join("Cargo.toml")).expect("workspace Cargo.toml");
    let line = toml
        .lines()
        .find(|l| l.trim_start().starts_with("members = ["))
        .expect("members line");
    line[line.find('[').expect("open bracket") + 1..line.len() - 1]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn publish_order() -> Vec<String> {
    let workflow = std::fs::read_to_string(workspace_root().join(".github/workflows/publish.yml"))
        .expect("publish.yml");
    workflow
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("run: cargo publish -p ")
                .map(|c| c.trim().to_string())
        })
        .collect()
}

#[test]
fn every_workspace_member_is_in_the_publish_order() {
    let members = workspace_members();
    assert!(!members.is_empty());
    let published = publish_order();

    for member in &members {
        assert!(
            published.iter().any(|c| c == member),
            "workspace member `{member}` has no `cargo publish -p {member}` step in publish.yml"
        );
    }
}

#[test]
fn each_crate_publishes_exactly_once() {
    let published = publish_order();
    let mut seen = std::collections::BTreeSet::new();
    for crate_name in &published {
        assert!(
            seen.insert(crate_name),
            "`{crate_name}` has duplicate publish steps — duplicates can hide a misordered \
             first occurrence (the 0.9.0 failure mode)"
        );
    }
}

#[test]
fn publish_order_respects_workspace_dependencies() {
    let published = publish_order();
    let position = |crate_name: &str| -> usize {
        published
            .iter()
            .position(|c| c == crate_name)
            .unwrap_or_else(|| panic!("`{crate_name}` not in publish.yml"))
    };

    for member in workspace_members() {
        let manifest = std::fs::read_to_string(workspace_root().join(&member).join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("{} Cargo.toml: {e}", member));
        let mut in_dependencies = false;
        let deps: Vec<&str> = manifest
            .lines()
            .filter_map(|l| {
                let trimmed = l.trim();
                if trimmed.starts_with('[') {
                    // Only normal dependencies constrain publish order;
                    // path-only dev-dependencies are stripped at publish
                    // time (karpal-verify-derive dev-depends on
                    // karpal-verify while the real edge runs the other way).
                    in_dependencies = trimmed == "[dependencies]";
                    return None;
                }
                let (name, rest) = trimmed.split_once(" = {")?;
                if in_dependencies && name.starts_with("karpal-") && rest.contains("path = ") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        for dep in deps {
            assert!(
                position(dep) < position(&member),
                "`{dep}` must publish before `{member}` (path dependency), but appears later \
                 in publish.yml — the 0.9.0 run failed exactly this way"
            );
        }
    }
}
