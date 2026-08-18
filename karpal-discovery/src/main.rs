// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The `karpal` binary — a Lonis-native composable CLI and a conforming
//! `SubprocessProvider` (ADR-0006 v0 surface):
//!
//! - `karpal --mode json manifest` — provider manifest,
//! - `karpal --mode json tools list` — tool summaries,
//! - `karpal --mode json tools describe <name>` — tool contracts,
//! - `karpal --mode json call <name>` — ADR-0003 invocation (JSON on stdin,
//!   blocks on stdout, structured `ToolError` on stderr).
//!
//! The discovery tools (Phase 19-E) are reachable both through `call` and as
//! legacy bare argv words (the `SubprocessTool` spike convention):
//! `search` (catalog items), `detail` (item + overlay + impl graph),
//! `concepts` (overlay browse), `imports` (imported-symbol analysis).
//!
//! Built only with the `lonis` feature (`required-features` on the bin).
//! The target **workspace** arrives via stdin because `SubprocessTool`
//! clears the environment and uses a neutral cwd.

#![cfg(feature = "lonis")]

use std::io::Read as _;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use karpal_discovery::payload::{ConceptSummary, ItemSummary, KarpalPayload};
use lonis_schema::{Attribution, Block, ToolError};

/// One entry in the static tool registry — the single source of truth for
/// the provider manifest, the tool list, and the tool contracts.
struct ToolSpec {
    /// Dotted tool name (`karpal.search`).
    name: &'static str,
    /// Short purpose description.
    description: &'static str,
    /// Input schema reference.
    input_schema: &'static str,
    /// Output schema reference.
    output_schema: &'static str,
}

/// The tool surface this binary hosts. All tools are read-only and
/// deterministic (bounded only by the inspected workspace).
const TOOLS: [ToolSpec; 4] = [
    ToolSpec {
        name: "karpal.search",
        description: "Search public items (traits, functions, types, macros) across a workspace catalog.",
        input_schema: "karpal.search.input/v1",
        output_schema: "karpal.search.output/v1",
    },
    ToolSpec {
        name: "karpal.detail",
        description: "One catalog item with docs, implementors, and anchored overlay concepts.",
        input_schema: "karpal.detail.input/v1",
        output_schema: "karpal.detail.output/v1",
    },
    ToolSpec {
        name: "karpal.concepts",
        description: "Browse curated mathematical concepts (names, aliases, problem shapes) from the embedded overlay.",
        input_schema: "karpal.concepts.input/v1",
        output_schema: "karpal.concepts.output/v1",
    },
    ToolSpec {
        name: "karpal.imports",
        description: "Analyze a workspace's use statements: resolved catalog symbols, drift, and concepts in use.",
        input_schema: "karpal.imports.input/v1",
        output_schema: "karpal.imports.output/v1",
    },
];

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // v0 provider surface prefix: `--mode json`.
    if args.first().map(String::as_str) == Some("--mode") {
        if args.get(1).map(String::as_str) != Some("json") {
            fail("invalid_input", "only `--mode json` is supported", 2);
        }
        args.drain(0..2);
    }
    let operation = args.first().cloned().unwrap_or_default();
    let rest: &[String] = &args[1.min(args.len())..];

    match operation.as_str() {
        "manifest" => run_manifest(),
        "tools" => match rest {
            [list] if list == "list" => run_tools_list(),
            [describe, name] if describe == "describe" => run_describe(name),
            _ => fail(
                "invalid_input",
                "expected `tools list` or `tools describe <name>`",
                2,
            ),
        },
        "call" => match rest.first() {
            Some(name) => route(name, read_input()),
            None => fail("invalid_input", "expected `call <tool-name>`", 2),
        },
        // Legacy bare tool words (SubprocessTool spike convention).
        "" => fail(
            "invalid_input",
            "missing operation (try `karpal search` or `karpal --mode json manifest`)",
            2,
        ),
        other => route(other, read_input()),
    }
}

/// Route one invocation (by dotted or bare tool name) to its handler.
fn route(name: &str, input: Value) {
    match name {
        "karpal.search" | "search" => run_search(input),
        "karpal.detail" | "detail" => run_detail(input),
        "karpal.concepts" | "concepts" => run_concepts(input),
        "karpal.imports" | "imports" => run_imports(input),
        other => fail(
            "unknown_tool",
            &format!("unknown tool `{other}` (see `karpal --mode json tools list`)"),
            2,
        ),
    }
}

/// Read the stdin JSON (or `Null` when empty/unparseable — handlers report).
fn read_input() -> Value {
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    serde_json::from_str(raw.trim()).unwrap_or(Value::Null)
}

// -- provider discovery surface (ADR-0006 v0) -------------------------------

fn run_manifest() {
    let manifest = serde_json::json!({
        "name": "karpal",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Category-theoretic discovery for the Karpal workspace: catalog, concepts, and imports.",
        "provider_type": "external-executable",
        "protocol_version": "0",
        "tools": TOOLS.iter().map(|t| t.name).collect::<Vec<_>>(),
        "display_name": "Karpal Discovery",
        "capabilities": [],
    });
    println!(
        "{}",
        serde_json::to_string(&manifest).expect("manifest serializes")
    );
}

fn run_tools_list() {
    let list = serde_json::json!({
        "provider": "karpal",
        "tools": TOOLS.iter().map(|t| serde_json::json!({
            "name": t.name,
            "description": t.description,
        })).collect::<Vec<_>>(),
    });
    println!(
        "{}",
        serde_json::to_string(&list).expect("tool list serializes")
    );
}

fn run_describe(name: &str) {
    let Some(spec) = TOOLS.iter().find(|t| t.name == name) else {
        fail(
            "unknown_tool",
            &format!("unknown tool `{name}` (see `karpal --mode json tools list`)"),
            2,
        );
    };
    let contract = lonis_schema::ToolContract {
        name: lonis_schema::ToolId::new(name.replace('.', ":"))
            .unwrap_or_else(|e| panic!("tool id `{name}` should be valid: {e}")),
        description: spec.description.to_string(),
        input_schema: lonis_schema::SchemaRef(spec.input_schema.to_string()),
        output_schema: lonis_schema::SchemaRef(spec.output_schema.to_string()),
        determinism: lonis_schema::Determinism::Deterministic,
        side_effects: lonis_schema::SideEffects::ReadOnly,
        cost: lonis_schema::Cost::Low,
        capabilities: Vec::new(),
    };
    println!(
        "{}",
        serde_json::to_string(&contract).expect("contract serializes")
    );
}

// -- discovery tools (Phase 19-E) --------------------------------------------

/// Input for the `search` operation.
#[derive(Deserialize)]
struct SearchInput {
    /// Path to the workspace root to catalogue.
    workspace: String,
    /// Substring query (case-insensitive) against item names.
    query: String,
}

/// Input for the `detail` operation.
#[derive(Deserialize)]
struct DetailInput {
    /// Path to the workspace root to catalogue.
    workspace: String,
    /// Item name (the bare identifier).
    item: String,
    /// Owning crate, when known (disambiguates same-named items).
    #[serde(rename = "crate", default)]
    crate_name: Option<String>,
}

/// Input for the `concepts` operation.
#[derive(Deserialize)]
struct ConceptsInput {
    /// Substring query against ids, names, aliases, math concepts, and
    /// problem shapes (case-insensitive; empty matches everything).
    #[serde(default)]
    query: String,
}

/// Input for the `imports` operation.
#[derive(Deserialize)]
struct ImportsInput {
    /// Path to the workspace to analyze (self-analysis: imports resolve
    /// against the workspace's own catalog).
    workspace: String,
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
    emit(KarpalPayload::Search {
        query: parsed.query,
        results,
    });
}

fn run_detail(input: Value) {
    let parsed: DetailInput = match serde_json::from_value(input) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(
                "invalid_input",
                &format!("expected `{{\"workspace\": …, \"item\": …, \"crate\": …?}}`: {error}"),
                4,
            );
        }
    };
    let catalog = extract(&parsed.workspace);
    let found = catalog
        .crates
        .iter()
        .filter(|(name, _)| {
            parsed
                .crate_name
                .as_ref()
                .is_none_or(|c| c == name.as_str())
        })
        .flat_map(|(_, krate)| krate.items.iter().map(move |item| (krate, item)))
        .find(|(_, item)| item.name == parsed.item);
    let Some((krate, item)) = found else {
        fail(
            "not_found",
            &format!("no catalogued item named `{}`", parsed.item),
            4,
        );
    };
    let overlay = karpal_discovery::load_concept_overlay();
    let symbol_ref = format!("{}::{}", item.crate_name, item.name);
    let concepts: Vec<ConceptSummary> = overlay
        .concepts
        .iter()
        .filter(|c| c.symbol_refs.contains(&symbol_ref))
        .map(ConceptSummary::from_record)
        .collect();
    let implementors = if matches!(item.kind, karpal_discovery::catalog::ItemKind::Trait(_)) {
        catalog.implementors_of(&item.name)
    } else {
        Vec::new()
    };
    let _ = krate;
    emit(KarpalPayload::Detail {
        item: ItemSummary::from_record(&item.name, &item.crate_name, &item.module_path, &item.kind),
        docs: item.docs.clone(),
        implementors,
        concepts,
    });
}

fn run_concepts(input: Value) {
    let parsed: ConceptsInput = match serde_json::from_value(input) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(
                "invalid_input",
                &format!("expected `{{\"query\": …?}}`: {error}"),
                4,
            );
        }
    };
    let overlay = karpal_discovery::load_concept_overlay();
    let needle = parsed.query.to_lowercase();
    let results: Vec<ConceptSummary> = overlay
        .concepts
        .iter()
        .filter(|c| {
            needle.is_empty()
                || c.id.to_lowercase().contains(&needle)
                || c.name.to_lowercase().contains(&needle)
                || c.aliases.iter().any(|a| a.to_lowercase().contains(&needle))
                || c.math_concepts
                    .iter()
                    .any(|m| m.to_lowercase().contains(&needle))
                || c.problem_shapes
                    .iter()
                    .any(|p| p.to_lowercase().contains(&needle))
        })
        .map(ConceptSummary::from_record)
        .collect();
    emit(KarpalPayload::Concepts {
        query: parsed.query,
        results,
    });
}

fn run_imports(input: Value) {
    let parsed: ImportsInput = match serde_json::from_value(input) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(
                "invalid_input",
                &format!("expected `{{\"workspace\": …}}`: {error}"),
                4,
            );
        }
    };
    let root = Path::new(&parsed.workspace);
    let catalog = karpal_discovery::extract::extract_workspace(root);
    let report = karpal_discovery::analyze_imports(root, &catalog);
    let overlay = karpal_discovery::load_concept_overlay();
    let concepts: Vec<ConceptSummary> = report
        .concepts_used(&overlay)
        .into_iter()
        .map(ConceptSummary::from_record)
        .collect();
    emit(KarpalPayload::Imports {
        resolved: report.resolved.len(),
        unresolved: report.unresolved.len(),
        globs: report.globs.len(),
        concepts,
    });
}

/// Emit one payload as a single-element block array (ADR-0003).
fn emit(payload: KarpalPayload) {
    let block = Block::new(
        Attribution::new("karpal-discovery", "karpal-discovery"),
        payload,
    );
    let blocks = vec![block];
    println!(
        "{}",
        serde_json::to_string(&blocks).expect("blocks serialize")
    );
}

/// Build the catalog for a workspace path.
fn extract(workspace: &str) -> karpal_discovery::Catalog {
    karpal_discovery::extract::extract_workspace(Path::new(workspace))
}

/// Case-insensitive substring search over every catalogued item's name.
fn search_items(catalog: &karpal_discovery::Catalog, query: &str) -> Vec<ItemSummary> {
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
