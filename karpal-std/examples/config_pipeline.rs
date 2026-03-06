//! Config Pipeline Example
//!
//! Demonstrates Alt (fallback chains), Traversable (all-or-nothing validation),
//! Monoid (merging), and ado_! (independent lookups).
//!
//! Run with: cargo run -p karpal-std --example config_pipeline

use karpal_std::prelude::*;

// --- Domain types ---

#[derive(Debug, Clone, PartialEq)]
struct AppConfig {
    db_host: String,
    db_port: u16,
    db_name: String,
    max_connections: u16,
    timeout_ms: u64,
}

// --- Simulated config sources ---

fn from_env(key: &str) -> Option<String> {
    // Simulate environment variables (only DB_HOST and DB_PORT are set)
    match key {
        "DB_HOST" => Some("prod-db.example.com".into()),
        "DB_PORT" => Some("5432".into()),
        _ => None,
    }
}

fn from_file(key: &str) -> Option<String> {
    // Simulate a config file (has DB_NAME and MAX_CONNECTIONS)
    match key {
        "DB_NAME" => Some("myapp".into()),
        "MAX_CONNECTIONS" => Some("20".into()),
        _ => None,
    }
}

fn from_default(key: &str) -> Option<String> {
    // Hardcoded defaults for everything
    match key {
        "DB_HOST" => Some("localhost".into()),
        "DB_PORT" => Some("5432".into()),
        "DB_NAME" => Some("app".into()),
        "MAX_CONNECTIONS" => Some("10".into()),
        "TIMEOUT_MS" => Some("5000".into()),
        _ => None,
    }
}

// --- Alt: fallback chain ---

/// Try env first, then file, then defaults.
fn resolve(key: &str) -> Option<String> {
    OptionF::alt(
        OptionF::alt(from_env(key), from_file(key)),
        from_default(key),
    )
}

// --- Traversable: validate all-or-nothing ---

/// Parse a string as a u16, returning None on failure.
fn parse_u16(s: String) -> Option<u16> {
    s.parse().ok()
}

/// Parse a string as a u64, returning None on failure.
fn parse_u64(s: String) -> Option<u64> {
    s.parse().ok()
}

// --- Putting it together ---

/// Load full config using Alt for fallbacks and ado_! for independent lookups.
fn load_config() -> Option<AppConfig> {
    // Resolve each key independently via Alt fallback chains
    let db_host = resolve("DB_HOST")?;
    let db_name = resolve("DB_NAME")?;

    // For numeric fields, resolve then parse
    let db_port = resolve("DB_PORT").and_then(parse_u16)?;
    let max_connections = resolve("MAX_CONNECTIONS").and_then(parse_u16)?;
    let timeout_ms = resolve("TIMEOUT_MS").and_then(parse_u64)?;

    Some(AppConfig {
        db_host,
        db_port,
        db_name,
        max_connections,
        timeout_ms,
    })
}

/// Demonstrate do_! for combining sequential Option lookups.
fn load_connection_string() -> Option<String> {
    do_! { OptionF;
        host = resolve("DB_HOST");
        port = resolve("DB_PORT");
        name = resolve("DB_NAME");
        Some(format!("postgres://{}:{}/{}", host, port, name))
    }
}

/// Demonstrate Traversable: validate a batch of port strings.
fn validate_ports(ports: Vec<&str>) -> Option<Vec<u16>> {
    VecF::traverse::<OptionF, _, _, _>(ports.into_iter().map(String::from).collect(), parse_u16)
}

/// Demonstrate Foldable + Monoid: aggregate config descriptions.
fn summarize_keys(keys: Vec<&str>) -> String {
    VecF::fold_map(
        keys.into_iter().map(String::from).collect::<Vec<_>>(),
        |key| match resolve(&key) {
            Some(val) => format!("  {} = {}\n", key, val),
            None => format!("  {} = <missing>\n", key),
        },
    )
}

fn main() {
    println!("=== Config Pipeline Example ===\n");

    // 1. Alt fallback chains
    println!("--- Resolving individual keys (Alt fallback) ---");
    println!("DB_HOST:         {:?}", resolve("DB_HOST"));
    println!("DB_PORT:         {:?}", resolve("DB_PORT"));
    println!("DB_NAME:         {:?}", resolve("DB_NAME"));
    println!("MAX_CONNECTIONS: {:?}", resolve("MAX_CONNECTIONS"));
    println!("TIMEOUT_MS:      {:?}", resolve("TIMEOUT_MS"));
    println!("UNKNOWN_KEY:     {:?}", resolve("UNKNOWN_KEY"));

    // 2. Full config loading
    println!("\n--- Loading full config ---");
    match load_config() {
        Some(config) => println!("{:#?}", config),
        None => println!("Failed to load config!"),
    }

    // 3. ado_! for independent lookups
    println!("\n--- Connection string (ado_!) ---");
    println!("{:?}", load_connection_string());

    // 4. Traversable: all-or-nothing validation
    println!("\n--- Batch port validation (Traversable) ---");
    let good_ports = vec!["80", "443", "8080"];
    let good_result = validate_ports(good_ports.clone());
    println!("Valid ports {:?}: {:?}", good_ports, good_result);

    let bad_ports = vec!["80", "not_a_port", "8080"];
    let bad_result = validate_ports(bad_ports.clone());
    println!("Mixed ports {:?}: {:?}", bad_ports, bad_result);

    // 5. Foldable + Monoid: summarize
    println!("\n--- Config summary (Foldable + Monoid) ---");
    let summary = summarize_keys(vec![
        "DB_HOST",
        "DB_PORT",
        "DB_NAME",
        "MAX_CONNECTIONS",
        "TIMEOUT_MS",
        "MISSING",
    ]);
    print!("{}", summary);
}
