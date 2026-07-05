# Data Transformation

ETL pipeline: parse records, transform fields, aggregate stats using Functor, Chain, Lens, Foldable, and Monoid.

## Overview

This example builds a small ETL (Extract, Transform, Load) pipeline that processes raw string records into typed transactions, applies transformations via lenses, and aggregates results using monoidal folding. It demonstrates how several Karpal abstractions compose naturally into a real-world data processing workflow:

- **Domain types** — raw input records and typed output structs, with a `Summary` type that implements `Semigroup` and `Monoid` for aggregation.
- **`do_!` and Chain** — monadic parsing that short-circuits on the first invalid field.
- **Lens** — composable getters and setters for individual struct fields.
- **Functor** — mapping transformations over collections of transactions.
- **Traversable** — all-or-nothing batch parsing that fails if any single record is invalid.
- **Foldable + Monoid** — aggregating transactions into summaries, including grouped-by-category breakdowns.

## 1. Domain Types

The pipeline starts with `RawRecord`, where every field is a `String` (as you might receive from a CSV parser or HTTP request). The goal is to parse these into strongly-typed `Transaction` values.

``` rust
#[derive(Debug, Clone)]
struct RawRecord {
    id: String,
    name: String,
    amount: String,
    category: String,
}

#[derive(Debug, Clone)]
struct Transaction {
    id: u32,
    name: String,
    amount_cents: i64,
    category: String,
}
```

For aggregation, we define a `Summary` type that tracks the number of transactions and their total value in cents. By implementing `Semigroup` and `Monoid`, we can combine summaries using `fold_map` without writing any manual accumulation logic.

``` rust
#[derive(Debug, Clone)]
struct Summary {
    count: i64,
    total_cents: i64,
}

impl Semigroup for Summary {
    fn combine(self, other: Self) -> Self {
        Summary {
            count: self.count + other.count,
            total_cents: self.total_cents + other.total_cents,
        }
    }
}

impl Monoid for Summary {
    fn empty() -> Self {
        Summary {
            count: 0,
            total_cents: 0,
        }
    }
}
```

The `Semigroup::combine` implementation adds counts and totals together. The `Monoid::empty` value is the identity element — zero transactions with zero total — which serves as the starting point for any fold.

## 2. Parsing with `do_!` (Chain)

Each raw record needs two fields parsed: `id` (a `u32`) and `amount` (a floating-point dollar value converted to cents). If either parse fails, the whole record is invalid. The `do_!` macro makes this sequential validation read top-to-bottom, with automatic short-circuiting on `None`:

``` rust
fn parse_record(raw: RawRecord) -> Option<Transaction> {
    let name = raw.name.clone();
    let category = raw.category.clone();
    do_! { OptionF;
        id = raw.id.parse::<u32>().ok();
        amount = raw.amount.parse::<f64>().ok();
        Some(Transaction {
            id,
            name: name.clone(),
            amount_cents: (amount * 100.0) as i64,
            category: category.clone(),
        })
    }
}
```

Each `name = expr` line unwraps the `Option` returned by the right-hand side. If `raw.id.parse::<u32>().ok()` returns `None`, the entire block immediately evaluates to `None` without attempting to parse the amount. This is the `Chain` (monadic bind) behavior provided by `OptionF`.

To parse an entire batch of records with all-or-nothing semantics, we use `Traversable`:

``` rust
fn parse_all(records: Vec<RawRecord>) -> Option<Vec<Transaction>> {
    VecF::traverse::<OptionF, _, _, _>(records, parse_record)
}
```

`VecF::traverse` applies `parse_record` to every element in the vector and collects the results. If all records parse successfully, you get `Some(vec_of_transactions)`. If any single record fails, the entire result is `None`. This is the "all-or-nothing" guarantee of `Traversable`.

## 3. Lens Field Access

To modify individual fields of a `Transaction` without manually destructuring the struct, we define lenses. A `SimpleLens<S, A>` provides a getter (`S -> A`) and a setter (`(S, A) -> S`) for a single field:

``` rust
fn amount_lens() -> SimpleLens<Transaction, i64> {
    Lens::new(
        |t: &Transaction| t.amount_cents,
        |t, amount_cents| Transaction {
            amount_cents,
            ..t
        },
    )
}

fn name_lens() -> SimpleLens<Transaction, String> {
    Lens::new(
        |t: &Transaction| t.name.clone(),
        |t, name| Transaction { name, ..t },
    )
}
```

The getter closure reads the field; the setter closure returns a new `Transaction` with that one field replaced, using Rust's struct update syntax (`..t`) to copy the remaining fields. Lenses are first-class values — you can store them, pass them to functions, and compose them with `.then()`.

## 4. Functor Transforms

With lenses in hand, we can define transformation functions that modify a specific field across an entire collection. `VecF::fmap` applies a function to every element of a `Vec`, and the lens's `.over()` method applies a function to the focused field:

``` rust
/// Apply a discount: reduce amount by a percentage.
fn apply_discount(transactions: Vec<Transaction>, pct: f64) -> Vec<Transaction> {
    let lens = amount_lens();
    VecF::fmap(transactions, |t| {
        lens.over(t, |a| (a as f64 * (1.0 - pct / 100.0)) as i64)
    })
}

/// Normalize names to uppercase.
fn normalize_names(transactions: Vec<Transaction>) -> Vec<Transaction> {
    let lens = name_lens();
    VecF::fmap(transactions, |t| {
        lens.over(t, |n| n.to_uppercase())
    })
}
```

`apply_discount` uses `amount_lens` to reach into each transaction and scale the `amount_cents` field. `normalize_names` uses `name_lens` to uppercase the `name` field. Neither function needs to know about the other fields in `Transaction` — the lens handles the boilerplate of reading, modifying, and writing back.

## 5. Foldable + Monoid Aggregation

The final stage of the pipeline aggregates transactions into summaries. Because `Summary` implements `Monoid`, we can use `VecF::fold_map` to convert each transaction into a single-element summary and then combine them all:

``` rust
fn summarize(transactions: &[Transaction]) -> Summary {
    VecF::fold_map(transactions.to_vec(), |t| Summary {
        count: 1,
        total_cents: t.amount_cents,
    })
}
```

`fold_map` maps each element to a `Summary` (with count 1 and that transaction's amount), then combines all the summaries using `Semigroup::combine`, starting from `Monoid::empty()`. For an empty collection, it returns the identity summary (0 transactions, 0 total).

For grouped aggregation, we partition by category and summarize each group independently:

``` rust
fn summarize_by_category(transactions: &[Transaction]) -> Vec<(String, Summary)> {
    let mut categories: Vec<String> = transactions.iter().map(|t| t.category.clone()).collect();
    categories.sort();
    categories.dedup();

    categories
        .into_iter()
        .map(|cat| {
            let filtered: Vec<Transaction> = transactions
                .iter()
                .filter(|t| t.category == cat)
                .cloned()
                .collect();
            (cat, summarize(&filtered))
        })
        .collect()
}
```

Each category gets its own `Summary`, computed by the same `summarize` function. The monoidal structure means the aggregation logic is defined once (in `Semigroup` and `Monoid`) and reused everywhere.

## 6. The Complete Pipeline

The `main` function ties everything together. It creates sample data, parses it, applies transformations, and prints aggregate results:

``` rust
fn main() {
    // Sample data
    let records = vec![
        RawRecord { id: "1".into(), name: "Alice".into(),
                     amount: "99.99".into(), category: "electronics".into() },
        RawRecord { id: "2".into(), name: "Bob".into(),
                     amount: "24.50".into(), category: "books".into() },
        RawRecord { id: "3".into(), name: "Carol".into(),
                     amount: "149.00".into(), category: "electronics".into() },
        RawRecord { id: "4".into(), name: "Dave".into(),
                     amount: "12.75".into(), category: "books".into() },
    ];

    // 1. Parse all records (Traversable)
    let transactions = parse_all(records).expect("All records should parse");

    // 2. Transform with Functor + Lens
    let discounted = apply_discount(transactions.clone(), 10.0);
    let normalized = normalize_names(transactions.clone());

    // 3. Aggregate with Foldable + Monoid
    let summary = summarize(&transactions);
    let by_category = summarize_by_category(&transactions);

    // 4. Demonstrate failed parse
    let bad_records = vec![
        RawRecord { id: "5".into(), name: "Eve".into(),
                     amount: "50.00".into(), category: "food".into() },
        RawRecord { id: "bad".into(), name: "Frank".into(),
                     amount: "30.00".into(), category: "food".into() },
    ];
    let result = parse_all(bad_records); // None -- "bad" is not a valid u32
}
```

The pipeline flows in a clear sequence: raw strings are parsed into typed values, transformed using lenses, and aggregated using monoidal folds. Each stage uses a different Karpal abstraction, but they compose seamlessly because they all operate on the same standard types.

## Run It

From the workspace root:

``` rust
cargo run -p karpal-std --example data_transformation
```

Expected output:

    === Data Transformation Example ===

    --- Parse records (Traversable) ---
      #1: Alice - $99.99 (electronics)
      #2: Bob - $24.50 (books)
      #3: Carol - $149.00 (electronics)
      #4: Dave - $12.75 (books)

    --- Apply 10% discount (Functor + Lens) ---
      #1: $89.99
      #2: $22.05
      #3: $134.10
      #4: $11.47

    --- Normalize names (Functor + Lens) ---
      #1: ALICE
      #2: BOB
      #3: CAROL
      #4: DAVE

    --- Overall summary (Foldable + Monoid) ---
      4 transactions, total: $286.24

    --- By category ---
      books: 2 transactions, total: $37.25
      electronics: 2 transactions, total: $248.99

    --- Failed parse (bad data) ---
      parse_all result: None

## Traits Used

| Trait         | Role in this example                                                         | Reference                                                        |
|---------------|------------------------------------------------------------------------------|------------------------------------------------------------------|
| `Semigroup`   | Combines two `Summary` values by adding counts and totals                    | [Semigroup & Monoid](../reference/algebraic.md)                |
| `Monoid`      | Provides the identity `Summary` (zero count, zero total) for folding         | [Semigroup & Monoid](../reference/algebraic.md)                |
| `Functor`     | `VecF::fmap` applies discount and name normalization across transactions     | [Functor Family](../reference/functor-family.md)               |
| `Chain`       | Powers the `do_!` macro for sequential parsing with short-circuit on failure | [Functor Family](../reference/functor-family.md)               |
| `Foldable`    | `VecF::fold_map` aggregates transactions into monoidal summaries             | [Foldable & Traversable](../reference/foldable-traversable.md) |
| `Traversable` | `VecF::traverse` parses all records with all-or-nothing semantics            | [Foldable & Traversable](../reference/foldable-traversable.md) |
| `Lens`        | Provides composable getters/setters for `amount_cents` and `name` fields     | [Optics](../reference/optics.md)                               |


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


