// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Provider-surface tests (Phase 19-D/E) — the `karpal` binary as a
//! conforming Lonis `SubprocessProvider` (ADR-0006 v0 surface), exercised
//! through the real host adapter from `lonis-core`:
//!
//! - `karpal --mode json manifest`       → provider manifest,
//! - `karpal --mode json tools list`     → tool summaries,
//! - `karpal --mode json tools describe` → tool contracts,
//! - `karpal --mode json call <name>`    → ADR-0003 invocation.
//!
//! The discovery commands (19-E) ride the same surface: `karpal.search`
//! (catalog search), `karpal.detail` (item detail + overlay concepts),
//! `karpal.concepts` (overlay browse), `karpal.imports` (imported-symbol
//! analysis). The legacy bare-argv `karpal search` spike convention stays
//! working (covered by `subprocess_hosting.rs`).

#![cfg(feature = "lonis")]

use std::path::{Path, PathBuf};

use lonis_core::{SubprocessProvider, Tool};
use lonis_schema::Determinism::Deterministic;
use lonis_schema::SideEffects::ReadOnly;
use lonis_schema::{BlockKind, ToolError};

fn karpal_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_karpal"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

fn provider() -> SubprocessProvider {
    SubprocessProvider::new(karpal_bin())
}

#[test]
fn manifest_is_discoverable() {
    let manifest = provider().manifest().expect("manifest");
    assert_eq!(manifest.name, "karpal");
    assert_eq!(manifest.provider_type, "external-executable");
    assert_eq!(manifest.protocol_version, "0");
    assert!(manifest.tools.contains(&"karpal.search".to_string()));
    assert!(manifest.tools.contains(&"karpal.detail".to_string()));
    assert!(manifest.tools.contains(&"karpal.concepts".to_string()));
    assert!(manifest.tools.contains(&"karpal.imports".to_string()));
}

#[test]
fn tools_list_carries_descriptions() {
    let tools = provider().tools().expect("tools list");
    assert!(
        tools.iter().any(|t| t.name == "karpal.search"
            && t.description.as_deref().is_some_and(|d| !d.is_empty()))
    );
    assert_eq!(
        tools.len(),
        4,
        "search, detail, concepts, imports: {tools:?}"
    );
}

#[test]
fn tool_contracts_declare_readonly_determinism() {
    let provider = provider();
    for name in [
        "karpal.search",
        "karpal.detail",
        "karpal.concepts",
        "karpal.imports",
    ] {
        let contract = provider
            .describe(name)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(contract.name.as_str(), name.replace('.', ":"));
        assert_eq!(contract.determinism, Deterministic);
        assert_eq!(contract.side_effects, ReadOnly);
        assert!(
            contract.input_schema.0.contains("karpal."),
            "input schema ref should name the tool: {}",
            contract.input_schema.0
        );
    }
}

#[test]
fn unknown_tool_description_is_a_structured_error() {
    let error = provider().describe("karpal.bogus").expect_err("bogus tool");
    let ToolError { kind, .. } = &error;
    assert_eq!(kind, "unknown_tool");
}

fn call(
    name: &str,
    input: serde_json::Value,
) -> Result<Vec<lonis_schema::Block<BlockKind>>, ToolError> {
    provider().tool(name).invoke(input)
}

fn payload_of(block: &lonis_schema::Block<BlockKind>) -> &BlockKind {
    block.payload()
}

#[test]
fn search_works_through_the_call_surface() {
    let blocks = call(
        "karpal.search",
        serde_json::json!({ "workspace": workspace_root(), "query": "Functor" }),
    )
    .expect("search via call");
    assert_eq!(blocks.len(), 1);
    let BlockKind::Extension { kind, data } = payload_of(&blocks[0]) else {
        panic!("expected Extension payload");
    };
    assert_eq!(kind, "karpal.search");
    assert!(!data["results"].as_array().unwrap().is_empty());
}

#[test]
fn detail_joins_catalog_and_overlay() {
    let blocks = call(
        "karpal.detail",
        serde_json::json!({ "workspace": workspace_root(), "item": "Functor", "crate": "karpal-core" }),
    )
    .expect("detail via call");
    let BlockKind::Extension { kind, data } = payload_of(&blocks[0]) else {
        panic!("expected Extension payload");
    };
    assert_eq!(kind, "karpal.detail");
    assert_eq!(data["item"]["name"], "Functor");
    // the overlay join is visible: the functor concept anchors this item
    let concepts = data["concepts"].as_array().expect("concepts array");
    assert!(
        concepts.iter().any(|c| c["id"] == "functor"),
        "{concepts:?}"
    );
    // implementors come from the catalog's impl graph
    let implementors = data["implementors"].as_array().expect("implementors array");
    assert!(
        !implementors.is_empty(),
        "karpal implements Functor somewhere"
    );
}

#[test]
fn concepts_browses_the_overlay() {
    let blocks = call(
        "karpal.concepts",
        serde_json::json!({ "workspace": workspace_root(), "query": "monad" }),
    )
    .expect("concepts via call");
    let BlockKind::Extension { kind, data } = payload_of(&blocks[0]) else {
        panic!("expected Extension payload");
    };
    assert_eq!(kind, "karpal.concepts");
    let results = data["results"].as_array().expect("results array");
    let ids: Vec<&str> = results.iter().filter_map(|r| r["id"].as_str()).collect();
    // "monad" matches monad itself and freer-monad (alternative), etc.
    assert!(ids.contains(&"monad"), "{ids:?}");
    // every hit carries its symbol refs — the anchor back into the catalog
    assert!(
        results
            .iter()
            .all(|r| r["symbol_refs"].as_array().is_some())
    );
}

#[test]
fn imports_analyzes_the_real_workspace() {
    let blocks = call(
        "karpal.imports",
        serde_json::json!({ "workspace": workspace_root() }),
    )
    .expect("imports via call");
    let BlockKind::Extension { kind, data } = payload_of(&blocks[0]) else {
        panic!("expected Extension payload");
    };
    assert_eq!(kind, "karpal.imports");
    // karpal consumes its own catalog: real resolved symbols, real concepts
    assert!(
        data["resolved"].as_u64().unwrap() >= 1,
        "karpal imports its own items"
    );
    let concepts = data["concepts"].as_array().expect("concepts array");
    let ids: Vec<&str> = concepts.iter().filter_map(|c| c["id"].as_str()).collect();
    assert!(ids.contains(&"functor"), "{ids:?}");
    assert_eq!(
        data["unresolved"].as_u64(),
        Some(0),
        "karpal's own imports should resolve"
    );
}

#[test]
fn legacy_bare_search_argv_still_works() {
    // The SubprocessTool spike convention (argv = ["search"]) predates the
    // provider surface and stays supported.
    let tool = lonis_core::SubprocessTool::new(
        lonis_schema::ToolId::new("karpal:search").unwrap(),
        karpal_bin(),
    )
    .with_args(vec!["search".to_string()]);
    let blocks = tool
        .invoke(serde_json::json!({ "workspace": workspace_root(), "query": "Functor" }))
        .expect("legacy search");
    assert_eq!(blocks.len(), 1);
}
