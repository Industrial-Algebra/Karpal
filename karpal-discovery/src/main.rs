// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The `karpal` binary — a Lonis-native composable CLI (ADR-0003 wire
//! protocol): JSON input on stdin, blocks on stdout, structured `ToolError`
//! on stderr. Hostable by `lonis-core`'s `SubprocessTool`.
//!
//! Built only with the `lonis` feature (`required-features` on the bin). The
//! first argument selects the operation; the input JSON on stdin carries the
//! operation's parameters (notably the target **workspace** — which must come
//! via stdin because `SubprocessTool` clears the environment and uses a
//! neutral cwd).

#![cfg(feature = "lonis")]

use std::io::Read as _;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use karpal_discovery::Catalog;
use karpal_discovery::payload::{ItemSummary, KarpalPayload};
use lonis_schema::{Attribution, Block, ToolError};

/// Input for the `search` operation.
#[derive(Deserialize)]
struct SearchInput {
    /// Path to the workspace root to catalogue.
    workspace: String,
    /// Substring query (case-insensitive) against item names.
    query: String,
}

fn main() {
    let operation = std::env::args().nth(1).unwrap_or_default();
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let input: Value = serde_json::from_str(raw.trim()).unwrap_or(Value::Null);

    match operation.as_str() {
        "search" => run_search(input),
        "" => fail(
            "invalid_input",
            "missing operation (try `karpal search`)",
            2,
        ),
        other => fail(
            "unknown_operation",
            &format!("unknown operation `{other}`"),
            2,
        ),
    }
}

fn run_search(input: Value) {
    let parsed: SearchInput = match serde_json::from_value(input) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(
                "invalid_input",
                &format!("expected `{{\"workspace\": …, \"query\": …}}`: {error}"),
                4,
            );
        }
    };
    let catalog = extract(&parsed.workspace);
    let results = search_items(&catalog, &parsed.query);
    let block = Block::new(
        Attribution::new("karpal-discovery", "karpal-discovery"),
        KarpalPayload::Search {
            query: parsed.query,
            results,
        },
    );
    // ADR-0003: a JSON array of blocks on stdout.
    let blocks = vec![block];
    println!(
        "{}",
        serde_json::to_string(&blocks).expect("blocks serialize")
    );
}

/// Build the catalog for a workspace path.
fn extract(workspace: &str) -> Catalog {
    karpal_discovery::extract::extract_workspace(Path::new(workspace))
}

/// Case-insensitive substring search over every catalogued item's name.
fn search_items(catalog: &Catalog, query: &str) -> Vec<ItemSummary> {
    let needle = query.to_lowercase();
    let mut out = Vec::new();
    for krate in catalog.crates.values() {
        for item in &krate.items {
            if item.name.to_lowercase().contains(&needle) {
                out.push(ItemSummary::from_record(
                    &item.name,
                    &item.crate_name,
                    &item.module_path,
                    &item.kind,
                ));
            }
        }
    }
    out
}

/// Emit a structured `ToolError` on stderr and exit with its code.
fn fail(kind: &str, message: &str, code: u8) -> ! {
    let error = ToolError::new(kind, message, code);
    if let Ok(wire) = serde_json::to_string(&error) {
        eprintln!("{wire}");
    }
    std::process::exit(i32::from(code));
}
