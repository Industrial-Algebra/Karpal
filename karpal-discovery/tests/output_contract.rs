// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Output-contract golden tests (Phase 19-H) — the wire surface of the
//! `karpal` binary is pinned so it cannot drift silently:
//!
//! - **Provider JSON** (manifest, tools list, tool contracts) is compared
//!   byte-for-byte — it is fully static content.
//! - **Block outputs** are golden on their payload `data` (the volatile
//!   provenance timestamp `attribution.provenance.when` is normalized, and
//!   the envelope skeleton is asserted field-by-field).
//!
//! Regenerate deliberately with `KARPAL_UPDATE_GOLDENS=1 cargo test …`; a
//! golden change is a contract change and belongs in review.

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

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Run the binary with args + stdin, returning (stdout, stderr, success).
fn run(args: &[&str], stdin: &str) -> (String, String, bool) {
    let mut child = Command::new(karpal_bin())
        .args(args)
        .current_dir(workspace_root())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn karpal");
    use std::io::Write as _;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

thread_local! {
    static REGENERATED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Compare against a golden file, or regenerate when
/// `KARPAL_UPDATE_GOLDENS=1` (each test reports its regenerations at the
/// end via [`check_regen`]).
fn assert_golden(name: &str, content: &str) {
    let path = golden_dir().join(name);
    if std::env::var_os("KARPAL_UPDATE_GOLDENS").is_some() {
        std::fs::write(&path, content).expect("regenerate golden");
        REGENERATED.with(|r| r.borrow_mut().push(name.to_string()));
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("golden `{name}` missing ({e}); regenerate with KARPAL_UPDATE_GOLDENS=1")
    });
    if expected != content {
        panic!(
            "output contract drift for `{name}`.\n--- golden ---\n{expected}\n--- actual ---\n{content}\nIf intentional: KARPAL_UPDATE_GOLDENS=1 cargo test -p karpal-discovery --features lonis --test output_contract"
        );
    }
}

/// Panic (once, at test end) listing any goldens regenerated this test.
fn check_regen() {
    REGENERATED.with(|r| {
        let names = r.borrow();
        if !names.is_empty() {
            panic!(
                "golden(s) regenerated: {names:?} — inspect the diff and un-set KARPAL_UPDATE_GOLDENS"
            );
        }
    });
}

/// Extract `payload.data` as a comparison string from a block-array stdout,
/// asserting the envelope skeleton alongside.
fn payload_data(stdout: &str, expected_kind: &str) -> String {
    let blocks: serde_json::Value = serde_json::from_str(stdout.trim()).expect("block array JSON");
    let block = blocks
        .as_array()
        .and_then(|b| b.first())
        .unwrap_or_else(|| {
            panic!("one block in the array, got: {stdout}");
        });
    assert_eq!(
        block["schema_version"], "lonis.block/v1",
        "envelope version"
    );
    assert_eq!(
        block["attribution"]["provenance"]["producer"], "karpal-discovery",
        "envelope producer"
    );
    assert!(
        block["attribution"]["provenance"]["when"]
            .as_str()
            .is_some_and(|w| !w.is_empty()),
        "envelope timestamp present (normalized out of the golden)"
    );
    assert_eq!(block["payload"]["kind"], expected_kind, "payload kind");
    serde_json::to_string_pretty(&block["payload"]["data"]).expect("data serializes")
}

#[test]
fn provider_manifest_is_golden() {
    let (out, err, ok) = run(&["--mode", "json", "manifest"], "");
    assert!(ok, "stderr: {err}");
    assert_golden("manifest.json", out.trim_end());
    check_regen();
}

#[test]
fn provider_tools_list_is_golden() {
    let (out, err, ok) = run(&["--mode", "json", "tools", "list"], "");
    assert!(ok, "stderr: {err}");
    assert_golden("tools_list.json", out.trim_end());
    check_regen();
}

#[test]
fn provider_tool_contracts_are_golden() {
    for name in [
        "karpal.search",
        "karpal.detail",
        "karpal.concepts",
        "karpal.imports",
        "karpal.recommend",
        "karpal.plan",
        "karpal.probe_list",
        "karpal.probe_describe",
        "karpal.probe_run",
    ] {
        let (out, err, ok) = run(&["--mode", "json", "tools", "describe", name], "");
        assert!(ok, "{name}: {err}");
        let file = format!("describe_{}.json", name.replace('.', "_"));
        assert_golden(&file, out.trim_end());
    }
    check_regen();
}

#[test]
fn search_block_payload_is_golden() {
    let (out, err, ok) = run(
        &["call", "karpal.search"],
        r#"{"workspace": ".", "query": "Functor"}"#,
    );
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.search");
    assert_golden("search_functor.data.json", &data);
    check_regen();
}

#[test]
fn detail_block_payload_is_golden() {
    let (out, err, ok) = run(
        &["call", "karpal.detail"],
        r#"{"workspace": ".", "item": "Functor", "crate": "karpal-core"}"#,
    );
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.detail");
    assert_golden("detail_functor.data.json", &data);
    check_regen();
}

#[test]
fn recommend_block_payload_is_golden() {
    let (out, err, ok) = run(&["call", "karpal.recommend"], r#"{"goal": "monad"}"#);
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.recommend");
    assert_golden("recommend_monad.data.json", &data);
    check_regen();
}

#[test]
fn recommend_weak_recall_diagnostics_are_golden() {
    // 0.9.1 contract (Lonis #21 R7): when nothing recalls with confidence,
    // the payload explains itself (note + nearest vocabulary) instead of
    // returning a silent empty list.
    let (out, err, ok) = run(
        &["call", "karpal.recommend"],
        r#"{"goal": "elephant pajamas waltzing"}"#,
    );
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.recommend");
    assert!(
        serde_json::from_str::<serde_json::Value>(&data)
            .expect("recommend payload JSON")
            .get("diagnostics")
            .and_then(|d| d.get("note"))
            .is_some(),
        "weak recall carries diagnostics.note"
    );
    assert_golden("recommend_nomatch.data.json", &data);
    check_regen();
}

#[test]
fn plan_block_payload_is_golden() {
    let (out, err, ok) = run(&["call", "karpal.plan"], r#"{"goal": "monad"}"#);
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.plan");
    assert_golden("plan_monad.data.json", &data);
    check_regen();
}

#[test]
fn concepts_block_payload_is_golden() {
    let (out, err, ok) = run(&["call", "karpal.concepts"], r#"{"query": "sheaf"}"#);
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.concepts");
    assert_golden("concepts_sheaf.data.json", &data);
    check_regen();
}

#[test]
fn probe_run_block_payload_is_golden() {
    let (out, err, ok) = run(
        &["call", "karpal.probe_run"],
        r#"{"id": "schubert-intersection"}"#,
    );
    assert!(ok, "stderr: {err}");
    let data = payload_data(&out, "karpal.probe_run");
    assert_golden("probe_run_schubert.data.json", &data);
    check_regen();
}
