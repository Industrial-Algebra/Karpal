//! Data Transformation Example
//!
//! Demonstrates an ETL-style pipeline using Functor (transforms),
//! Chain/do_! (parsing), Lens (field access), and Foldable + Monoid (aggregation).
//!
//! Run with: cargo run -p karpal-std --example data_transformation

use karpal_std::prelude::*;

// --- Domain types ---

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

// --- Parsing with do_! (Chain) ---

/// Parse a raw record into a Transaction, failing on bad data.
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

// --- Lens for field access ---

fn amount_lens() -> SimpleLens<Transaction, i64> {
    Lens::new(
        |t: &Transaction| t.amount_cents,
        |t, amount_cents| Transaction { amount_cents, ..t },
    )
}

fn name_lens() -> SimpleLens<Transaction, String> {
    Lens::new(
        |t: &Transaction| t.name.clone(),
        |t, name| Transaction { name, ..t },
    )
}

// --- Functor: transform fields ---

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
    VecF::fmap(transactions, |t| lens.over(t, |n| n.to_uppercase()))
}

// --- Foldable + Monoid: aggregate ---

/// Summarize a collection of transactions.
fn summarize(transactions: &[Transaction]) -> Summary {
    VecF::fold_map(transactions.to_vec(), |t| Summary {
        count: 1,
        total_cents: t.amount_cents,
    })
}

/// Summarize by category.
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

// --- Traversable: parse all-or-nothing ---

fn parse_all(records: Vec<RawRecord>) -> Option<Vec<Transaction>> {
    VecF::traverse::<OptionF, _, _, _>(records, parse_record)
}

fn main() {
    println!("=== Data Transformation Example ===\n");

    // Sample data
    let records = vec![
        RawRecord {
            id: "1".into(),
            name: "Alice".into(),
            amount: "99.99".into(),
            category: "electronics".into(),
        },
        RawRecord {
            id: "2".into(),
            name: "Bob".into(),
            amount: "24.50".into(),
            category: "books".into(),
        },
        RawRecord {
            id: "3".into(),
            name: "Carol".into(),
            amount: "149.00".into(),
            category: "electronics".into(),
        },
        RawRecord {
            id: "4".into(),
            name: "Dave".into(),
            amount: "12.75".into(),
            category: "books".into(),
        },
    ];

    // 1. Parse all records (Traversable)
    println!("--- Parse records (Traversable) ---");
    let transactions = parse_all(records).expect("All records should parse");
    for t in &transactions {
        println!(
            "  #{}: {} - ${:.2} ({})",
            t.id,
            t.name,
            t.amount_cents as f64 / 100.0,
            t.category
        );
    }

    // 2. Transform with Functor + Lens
    println!("\n--- Apply 10% discount (Functor + Lens) ---");
    let discounted = apply_discount(transactions.clone(), 10.0);
    for t in &discounted {
        println!("  #{}: ${:.2}", t.id, t.amount_cents as f64 / 100.0);
    }

    println!("\n--- Normalize names (Functor + Lens) ---");
    let normalized = normalize_names(transactions.clone());
    for t in &normalized {
        println!("  #{}: {}", t.id, t.name);
    }

    // 3. Aggregate with Foldable + Monoid
    println!("\n--- Overall summary (Foldable + Monoid) ---");
    let summary = summarize(&transactions);
    println!(
        "  {} transactions, total: ${:.2}",
        summary.count,
        summary.total_cents as f64 / 100.0
    );

    println!("\n--- By category ---");
    for (cat, s) in summarize_by_category(&transactions) {
        println!(
            "  {}: {} transactions, total: ${:.2}",
            cat,
            s.count,
            s.total_cents as f64 / 100.0
        );
    }

    // 4. Failed parse demonstration
    println!("\n--- Failed parse (bad data) ---");
    let bad_records = vec![
        RawRecord {
            id: "5".into(),
            name: "Eve".into(),
            amount: "50.00".into(),
            category: "food".into(),
        },
        RawRecord {
            id: "bad".into(),
            name: "Frank".into(),
            amount: "30.00".into(),
            category: "food".into(),
        },
    ];
    println!("  parse_all result: {:?}", parse_all(bad_records));
}
