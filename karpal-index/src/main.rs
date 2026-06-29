// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! AI-agent library discovery CLI for the Karpal ecosystem.
//!
//! Walks the Karpal workspace source tree, extracts public API items
//! from each crate using `syn`, and exposes them via progressive-discovery
//! CLI commands: search, detail, hierarchy, crates, example.

mod indexer;

use indexer::WorkspaceIndex;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: karpal-index <command> [args]");
        eprintln!("Commands: search <query> | detail <name> | crates | hierarchy <trait>");
        process::exit(1);
    }

    let workspace_root = env::current_dir().expect("current directory");
    let index = WorkspaceIndex::build(&workspace_root);

    match args[1].as_str() {
        "search" => {
            let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
            cmd_search(&index, query);
        }
        "detail" => {
            let name = args.get(2).expect("usage: detail <name>");
            cmd_detail(&index, name);
        }
        "crates" => {
            cmd_crates(&index);
        }
        "hierarchy" => {
            let name = args.get(2).expect("usage: hierarchy <trait>");
            cmd_hierarchy(&index, name);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            process::exit(1);
        }
    }
}

fn cmd_search(index: &WorkspaceIndex, query: &str) {
    let results = index.search(query);
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

fn cmd_detail(index: &WorkspaceIndex, name: &str) {
    match index.find(name) {
        Some(item) => {
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
            println!("{}: not found", name);
        }
    }
}

fn cmd_crates(index: &WorkspaceIndex) {
    for (name, desc) in &index.crate_descriptions {
        let count = index.items.iter().filter(|i| i.crate_name == *name).count();
        println!("{:<25} {:>4} items  {}", name, count, desc);
    }
}

fn cmd_hierarchy(index: &WorkspaceIndex, name: &str) {
    match index.find(name) {
        Some(item) => {
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
        None => println!("{}: not found", name),
    }
}
