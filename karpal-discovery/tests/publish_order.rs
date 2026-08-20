// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Publish-order drift gate (Phase 19-H): every workspace member must
//! appear in `publish.yml`'s `cargo publish -p <crate>` sequence, so a
//! newly added crate cannot silently miss its publish step. Publish order
//! correctness (dependencies before dependents) is enforced in review; this
//! gate enforces coverage.

use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn every_workspace_member_is_in_the_publish_order() {
    let members: Vec<String> = {
        let toml = std::fs::read_to_string(workspace_root().join("Cargo.toml"))
            .expect("workspace Cargo.toml");
        let line = toml
            .lines()
            .find(|l| l.trim_start().starts_with("members = ["))
            .expect("members line");
        line[line.find('[').expect("open bracket") + 1..line.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    assert!(!members.is_empty());

    let workflow = std::fs::read_to_string(workspace_root().join(".github/workflows/publish.yml"))
        .expect("publish.yml");
    let published: Vec<&str> = workflow
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            l.strip_prefix("run: cargo publish -p ")
        })
        .collect();

    for member in &members {
        assert!(
            published.contains(&member.as_str()),
            "workspace member `{member}` has no `cargo publish -p {member}` step in publish.yml"
        );
    }
}
