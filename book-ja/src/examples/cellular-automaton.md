# Cellular Automaton

1D cellular automaton using Extend (Comonad) over NonEmptyVec.

## Overview

A **cellular automaton** is a grid of cells that evolves in discrete steps. At each step, every cell updates its value based on a *rule* that inspects the cell and its neighbors. The classic approach in functional programming is to model this with a **comonad**.

The key insight is that a comonad provides two operations that map directly onto the cellular automaton pattern:

- **`extract`** reads the "focused" cell — the current position in the grid.
- **`extend`** takes a function that computes a new value from a focused context, and applies it at *every* position in the grid. This is exactly how a cellular automaton rule works: the rule sees the neighborhood around a position, and `extend` runs it everywhere.

In Karpal, `NonEmptyVec` implements `Extend` and `Comonad`. The `extend` method generates all possible focused views of the grid (via `tails`) and applies the rule function to each one, producing the next generation in a single call.

## Rule Functions

Each rule receives the entire grid as a `&NonEmptyVec<u8>`, with the *head* of the vector acting as the current cell. The rule inspects neighbors by looking at adjacent positions and returns the new value for that cell.

### Rule 90 (XOR of neighbors)

A cell becomes alive (`1`) if exactly one of its two neighbors is alive, otherwise it dies (`0`). This produces the classic Sierpinski triangle pattern when started from a single seed cell.

``` rust
fn rule_90(grid: &NonEmptyVec<u8>) -> u8 {
    let tails = grid.tails();
    let current = <NonEmptyVecF as Comonad>::extract(grid);
    let len = grid.len();

    // Get left neighbor (wrapping)
    let left = if len > 1 {
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
```

### Majority rule

A simpler rule: the cell is alive if two or more of (left, current, right) are alive. This tends to smooth out noise and converge toward uniform regions.

``` rust
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
```

## Evolution via Extend

The `step` function is the core of the automaton. It calls `NonEmptyVecF::extend` with the grid and a rule, producing the next generation. Extend applies the rule at every position by generating all focused views of the grid and mapping the rule over each one.

``` rust
fn step(grid: NonEmptyVec<u8>, rule: fn(&NonEmptyVec<u8>) -> u8) -> NonEmptyVec<u8> {
    NonEmptyVecF::extend(grid, rule)
}
```

The `evolve` function iterates `step` for a given number of generations, collecting the full history so it can be displayed as a space-time diagram.

``` rust
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
```

## Display

The display helper renders each generation as a string of `#` (alive) and `.` (dead) characters, making the pattern visible in the terminal.

``` rust
fn display_grid(grid: &NonEmptyVec<u8>) -> String {
    let mut s = String::new();
    for cell in grid.iter() {
        s.push(if *cell == 1 { '#' } else { '.' });
    }
    s
}
```

## Putting It Together

The `main` function sets up an initial grid with a single seed cell in the center, runs Rule 90 for 10 generations, then demonstrates the majority rule on a more complex pattern. It also shows `Comonad::extract` and `Extend::duplicate` directly.

``` rust
fn main() {
    // Initial state: single cell in the middle of a 21-cell grid
    let width = 21;
    let mid = width / 2;
    let mut cells: Vec<u8> = vec![0; width];
    cells[mid] = 1;
    let initial = NonEmptyVec::new(cells[0], cells[1..].to_vec());

    // Rule 90 (XOR of neighbors)
    let history = evolve(initial.clone(), rule_90, 10);
    for (i, grid) in history.iter().enumerate() {
        println!("  {:>2}: {}", i, display_grid(grid));
    }

    // Comonad::extract reads the focused cell
    let head = <NonEmptyVecF as Comonad>::extract(&initial);

    // Majority rule on a different pattern
    let pattern = NonEmptyVec::new(1, vec![0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0]);
    let history = evolve(pattern, rule_majority, 8);

    // Extend::duplicate shows all focused views
    let small = NonEmptyVec::new(1, vec![2, 3]);
    let duplicated: NonEmptyVec<NonEmptyVec<u8>> = NonEmptyVecF::duplicate(small);
}
```

## Run It

From the workspace root, run:

``` rust
cargo run -p karpal-std --example cellular_automaton
```

You will see the Rule 90 Sierpinski triangle pattern growing from a single seed, followed by the majority rule smoothing a random-looking pattern into stable regions.

## Traits Used

| Trait     | Role in this example                                                                                                                         | Reference                                          |
|-----------|----------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------|
| `Comonad` | Provides `extract` to read the focused cell value from a `NonEmptyVec`.                                                                      | [Comonad Family](../reference/comonad-family.md) |
| `Extend`  | Provides `extend` to apply a rule at every position, producing the next generation. Also provides `duplicate` to view all focused positions. | [Comonad Family](../reference/comonad-family.md) |
| `HKT`     | `NonEmptyVecF` is the type constructor marker that implements `Extend` and `Comonad`.                                                        | [Functor Family](../reference/functor-family.md) |


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


