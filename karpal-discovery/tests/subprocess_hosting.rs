// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Subprocess-hosting spike: host the `karpal` binary through Lonis's
//! `SubprocessTool` (ADR-0003 wire protocol) and assert the typed
//! `KarpalPayload` round-trips losslessly through the `BlockKind::Extension`
//! seam. Run with `--features lonis`.

#![cfg(feature = "lonis")]

use std::path::{Path, PathBuf};

use lonis_core::{SubprocessTool, Tool};
use lonis_schema::{BlockKind, ToolId};

/// The built `karpal` binary path.
fn karpal_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_karpal"))
}

/// The Karpal workspace root (two levels up from this crate).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn search_round_trips_through_the_extension_seam() {
    let tool = SubprocessTool::new(ToolId::new("karpal:search").unwrap(), karpal_bin())
        .with_args(vec!["search".to_string()]);
    let input = serde_json::json!({
        "workspace": workspace_root(),
        "query": "Functor",
    });

    let blocks = tool.invoke(input).expect("karpal search succeeds");
    assert_eq!(blocks.len(), 1, "one search block");

    let payload = blocks[0].payload();
    assert_eq!(payload.kind_name(), "karpal.search");

    // The vertical payload crossed the subprocess seam as `Extension`,
    // losslessly — the host never knew `KarpalPayload`, only `BlockKind`.
    let BlockKind::Extension { kind, data } = payload else {
        panic!("expected Extension, got {payload:?}");
    };
    assert_eq!(kind, "karpal.search");
    assert_eq!(data["query"], "Functor");
    let results = data["results"].as_array().expect("results is an array");
    assert!(
        results.iter().any(|item| item["name"] == "Functor"),
        "Functor appears in results: {results:?}"
    );
    // The envelope is the standard lonis block contract.
    let wire = serde_json::to_value(&blocks[0]).expect("serialize");
    assert_eq!(wire["schema_version"], "lonis.block/v1");
    assert_eq!(
        wire["attribution"]["provenance"]["producer"],
        "karpal-discovery"
    );
}

#[test]
fn search_block_validates_against_the_curated_envelope_schema() {
    // The spike-validation gate queued since the SubprocessTool spike: the
    // wire block the `karpal` binary emits must satisfy Lonis 0.1.0's
    // curated `block-v1.json` envelope schema (ADR-0005). A `karpal.search`
    // block crosses as `Extension`, so this also proves the seam branch of
    // the oneOf composes with a real JSON Schema engine.
    let tool = SubprocessTool::new(ToolId::new("karpal:search").unwrap(), karpal_bin())
        .with_args(vec!["search".to_string()]);
    let input = serde_json::json!({
        "workspace": workspace_root(),
        "query": "Functor",
    });
    let blocks = tool.invoke(input).expect("karpal search succeeds");
    let wire = serde_json::to_value(&blocks[0]).expect("serialize");

    let envelope = lonis_schema::block::schemas::block_schema(
        lonis_schema::block::schemas::BlockSchemaKind::Block,
    )
    .expect("embedded envelope schema loads");
    let validator =
        jsonschema::validator_for(&envelope.document).expect("envelope schema compiles");
    validator
        .validate(&wire)
        .expect("the karpal.search wire block satisfies block-v1.json");
}

#[test]
fn invalid_input_propagates_a_structured_tool_error() {
    let tool = SubprocessTool::new(ToolId::new("karpal:search").unwrap(), karpal_bin())
        .with_args(vec!["search".to_string()]);
    // No `workspace` / `query` → the binary emits a ToolError on stderr + exit 4.
    let result = tool.invoke(serde_json::Value::Null);
    let error = result.expect_err("invalid input should fail");
    assert_eq!(error.exit_code, 4);
    assert_eq!(error.kind, "invalid_input");
}
