// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! AI-agent library discovery CLI for the Karpal ecosystem.
//!
//! Walks the Karpal workspace source tree, extracts public API items
//! from each crate using `syn`, and exposes them via progressive-discovery
//! CLI commands: search, detail, crates, hierarchy, example.
//!
//! All commands support `--json` for machine-readable output.

mod indexer;

use indexer::WorkspaceIndex;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: karpal-index <command> [args] [--json]");
        eprintln!("Commands: search <query> | detail <name> | crates | hierarchy <trait>");
        process::exit(1);
    }

    // Extract --json flag
    let json_mode = args.iter().any(|a| a == "--json");
    let cmd_args: Vec<String> = args.iter().filter(|a| *a != "--json").cloned().collect();

    let workspace_root = env::current_dir().expect("current directory");
    let index = WorkspaceIndex::build(&workspace_root);

    match cmd_args[1].as_str() {
        "search" => {
            let query = cmd_args.get(2).map(|s| s.as_str()).unwrap_or("");
            cmd_search(&index, query, json_mode);
        }
        "detail" => {
            let name = cmd_args.get(2).expect("usage: detail <name>");
            cmd_detail(&index, name, json_mode);
        }
        "crates" => {
            cmd_crates(&index, json_mode);
        }
        "hierarchy" => {
            let name = cmd_args.get(2).expect("usage: hierarchy <trait>");
            cmd_hierarchy(&index, name, json_mode);
        }
        _ => {
            eprintln!("Unknown command: {}", cmd_args[1]);
            process::exit(1);
        }
    }
}

fn cmd_search(index: &WorkspaceIndex, query: &str, json: bool) {
    let results = index.search(query);
    if json {
        let json_results: Vec<&indexer::ApiItem> = results;
        println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
        return;
    }
    if results.is_empty() {
        println!("(no results for \"{}\")", query);
        return;
    }
    for item in &results {
        println!(
            "{:<30} {:<15} {}",
            item.name,
            item.kind,
            item.summary.as_deref().unwrap_or("")
        );
    }
}

fn cmd_detail(index: &WorkspaceIndex, name: &str, json: bool) {
    match index.find(name) {
        Some(item) => {
            if json {
                println!("{}", serde_json::to_string_pretty(item).unwrap());
                return;
            }
            println!("{} [{}]", item.name, item.kind);
            println!("  crate: {}", item.crate_name);
            println!("  path:  {}", item.path);
            if let Some(ref sig) = item.signature {
                println!("  sig:   {}", sig);
            }
            if let Some(ref docs) = item.docs {
                println!("  docs:  {}", docs);
            }
            if !item.supertraits.is_empty() {
                println!("  supertraits: {}", item.supertraits.join(", "));
            }
            if !item.methods.is_empty() {
                println!("  methods:");
                for m in &item.methods {
                    println!("    - {}", m);
                }
            }
            if !item.implementors.is_empty() {
                println!("  implementors:");
                for i in &item.implementors {
                    println!("    - {}", i);
                }
            }
            if !item.trait_impls.is_empty() {
                println!("  trait impls:");
                for t in &item.trait_impls {
                    println!("    - {}", t);
                }
            }
        }
        None => {
            if json {
                println!("null");
            } else {
                println!("{}: not found", name);
            }
        }
    }
}

fn cmd_crates(index: &WorkspaceIndex, json: bool) {
    #[derive(serde::Serialize)]
    struct CrateInfo<'a> {
        name: &'a str,
        items: usize,
        description: &'a str,
    }

    let crates: Vec<CrateInfo> = index
        .crate_descriptions
        .iter()
        .map(|(name, desc)| CrateInfo {
            name,
            items: index.items.iter().filter(|i| i.crate_name == *name).count(),
            description: desc,
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&crates).unwrap());
        return;
    }
    for c in &crates {
        println!("{:<25} {:>4} items  {}", c.name, c.items, c.description);
    }
}

fn cmd_hierarchy(index: &WorkspaceIndex, name: &str, json: bool) {
    #[derive(serde::Serialize)]
    struct Hierarchy<'a> {
        name: &'a str,
        kind: &'a str,
        supertraits: &'a [String],
        subtraits: &'a [String],
        implementors: &'a [String],
    }

    match index.find(name) {
        Some(item) => {
            if json {
                let h = Hierarchy {
                    name: &item.name,
                    kind: item.kind,
                    supertraits: &item.supertraits,
                    subtraits: &item.subtraits,
                    implementors: &item.implementors,
                };
                println!("{}", serde_json::to_string_pretty(&h).unwrap());
                return;
            }
            println!("{} [{}]", item.name, item.kind);
            if !item.supertraits.is_empty() {
                println!("  supertraits:");
                for s in &item.supertraits {
                    println!("    - {}", s);
                }
            }
            if !item.subtraits.is_empty() {
                println!("  subtraits:");
                for s in &item.subtraits {
                    println!("    - {}", s);
                }
            }
            if !item.implementors.is_empty() {
                println!("  implementors:");
                for i in &item.implementors {
                    println!("    - {}", i);
                }
            }
        }
        None => {
            if json {
                println!("null");
            } else {
                println!("{}: not found", name);
            }
        }
    }
}
