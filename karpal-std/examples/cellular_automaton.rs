//! Cellular Automaton Example
//!
//! Demonstrates the Comonad pattern: a 1D cellular automaton using
//! Extend (apply rule at every position) and Comonad (extract current cell)
//! over NonEmptyVec.
//!
//! Run with: cargo run -p karpal-std --example cellular_automaton

use karpal_std::prelude::*;

// --- Rule: look at current cell and neighbors ---

/// A simple 1D rule: a cell becomes alive (1) if exactly one of its
/// two neighbors is alive, otherwise it dies (0). This is similar to
/// Rule 90 (XOR of neighbors).
fn rule_90(grid: &NonEmptyVec<u8>) -> u8 {
    let tails = grid.tails();
    let current = <NonEmptyVecF as Comonad>::extract(grid);
    let len = grid.len();

    // Get left neighbor (wrapping)
    let left = if len > 1 {
        // The last tail gives us the rightmost element as neighbor context
        *tails.tail.last().map(|t| &t.head).unwrap_or(&grid.head)
    } else {
        current
    };

    // Get right neighbor
    let right = if grid.tail.is_empty() {
        grid.head // wrap around
    } else {
        grid.tail[0]
    };

    // XOR of neighbors
    left ^ right
}

/// Simpler rule: majority vote of (left, current, right).
/// Cell is alive if 2+ of the three are alive.
fn rule_majority(grid: &NonEmptyVec<u8>) -> u8 {
    let current = <NonEmptyVecF as Comonad>::extract(grid);
    let len = grid.len();

    let left = if len > 1 {
        let tails = grid.tails();
        *tails.tail.last().map(|t| &t.head).unwrap_or(&grid.head)
    } else {
        current
    };

    let right = if grid.tail.is_empty() {
        grid.head
    } else {
        grid.tail[0]
    };

    let sum = left as u16 + current as u16 + right as u16;
    if sum >= 2 { 1 } else { 0 }
}

// --- Evolution via Extend ---

/// Evolve the grid one step using Extend: applies the rule at every position.
fn step(grid: NonEmptyVec<u8>, rule: fn(&NonEmptyVec<u8>) -> u8) -> NonEmptyVec<u8> {
    NonEmptyVecF::extend(grid, rule)
}

/// Evolve for n steps.
fn evolve(
    initial: NonEmptyVec<u8>,
    rule: fn(&NonEmptyVec<u8>) -> u8,
    steps: usize,
) -> Vec<NonEmptyVec<u8>> {
    let mut history = vec![initial.clone()];
    let mut current = initial;
    for _ in 0..steps {
        current = step(current, rule);
        history.push(current.clone());
    }
    history
}

// --- Display ---

fn display_grid(grid: &NonEmptyVec<u8>) -> String {
    let mut s = String::new();
    for cell in grid.iter() {
        s.push(if *cell == 1 { '#' } else { '.' });
    }
    s
}

fn main() {
    println!("=== Cellular Automaton Example ===\n");
    println!("Using Extend (Comonad) to evolve a 1D cellular automaton.\n");

    // Initial state: single cell in the middle of a 21-cell grid
    let width = 21;
    let mid = width / 2;
    let mut cells: Vec<u8> = vec![0; width];
    cells[mid] = 1;
    let initial = NonEmptyVec::new(cells[0], cells[1..].to_vec());

    // Rule 90 (XOR of neighbors)
    println!("--- Rule 90 (XOR of neighbors) ---");
    let history = evolve(initial.clone(), rule_90, 10);
    for (i, grid) in history.iter().enumerate() {
        println!("  {:>2}: {}", i, display_grid(grid));
    }

    // Demonstrate Comonad::extract
    println!("\n--- Comonad::extract (read focused cell) ---");
    println!(
        "  Head of initial grid: {}",
        <NonEmptyVecF as Comonad>::extract(&initial)
    );

    // Majority rule
    println!("\n--- Majority rule ---");
    // Start with a more interesting pattern
    let pattern = NonEmptyVec::new(1, vec![0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0]);
    println!("  Initial: {}", display_grid(&pattern));
    let history = evolve(pattern, rule_majority, 8);
    for (i, grid) in history.iter().enumerate() {
        println!("  {:>2}: {}", i, display_grid(grid));
    }

    // Demonstrate Extend::duplicate
    println!("\n--- Extend::duplicate (all focused views) ---");
    let small = NonEmptyVec::new(1, vec![2, 3]);
    let duplicated: NonEmptyVec<NonEmptyVec<u8>> = NonEmptyVecF::duplicate(small);
    println!("  Original: [1, 2, 3]");
    println!("  Duplicated (each row is a focused view):");
    for (i, view) in duplicated.iter().enumerate() {
        let cells: Vec<String> = view.iter().map(|c| c.to_string()).collect();
        println!("    focus {}: [{}]", i, cells.join(", "));
    }
}
