# Config Pipeline

Load application config from multiple sources using Alt, Traversable, Foldable, and Monoid.

## Overview

Real applications rarely load configuration from a single source. Environment variables, config files, and hardcoded defaults each provide a partial picture. This example builds a configuration pipeline that:

- Uses **Alt** to create fallback chains across multiple config sources (env, file, defaults).
- Uses **`do_!`** to sequence dependent lookups into a connection string.
- Uses **Traversable** for all-or-nothing batch validation of port numbers.
- Uses **Foldable** with **Monoid** to aggregate a human-readable config summary.

The full source is at `karpal-std/examples/config_pipeline.rs`.

## 1. Domain Types

The example defines a simple `AppConfig` struct representing a database connection configuration:

``` rust
#[derive(Debug, Clone, PartialEq)]
struct AppConfig {
    db_host: String,
    db_port: u16,
    db_name: String,
    max_connections: u16,
    timeout_ms: u64,
}
```

## 2. Simulated Config Sources

Three functions simulate different configuration sources. Each takes a key and returns `Option<String>` — `Some` if the source knows about that key, `None` otherwise.

``` rust
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
```

No single source has every key. Environment variables provide the host and port; the config file provides the database name and connection pool size; defaults fill in anything still missing, including the timeout.

## 3. Alt Fallback Chains

The [Alt](../reference/alt-family.md) trait provides an associative "or" operation on type constructors. For `Option`, `Alt::alt` returns the first `Some` value, falling through to the next source if the current one returns `None`.

``` rust
/// Try env first, then file, then defaults.
fn resolve(key: &str) -> Option<String> {
    OptionF::alt(OptionF::alt(from_env(key), from_file(key)), from_default(key))
}
```

This reads inside-out: try `from_env`, fall back to `from_file`, then fall back to `from_default`. Because `Alt` is associative, the grouping does not matter — only the left-to-right priority order.

For example, `resolve("DB_HOST")` returns `Some("prod-db.example.com")` from the environment, while `resolve("TIMEOUT_MS")` skips both env and file (neither has it) and returns `Some("5000")` from defaults.

## 4. Loading the Full Config

With `resolve` in hand, loading the full `AppConfig` is straightforward. String fields are resolved directly; numeric fields are resolved and then parsed:

``` rust
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
```

The `?` operator short-circuits the entire function if any key cannot be resolved or any parse fails, returning `None`.

## 5. Connection String with `do_!`

The [`do_!` macro](../reference/macros.md) provides monadic sequencing. Here it combines three resolved values into a formatted connection string:

``` rust
fn load_connection_string() -> Option<String> {
    do_! { OptionF;
        host = resolve("DB_HOST");
        port = resolve("DB_PORT");
        name = resolve("DB_NAME");
        Some(format!("postgres://{}:{}/{}", host, port, name))
    }
}
```

Each `name = expr` line unwraps the `Option`. If any call to `resolve` returns `None`, the entire block short-circuits. The final expression produces the connection string wrapped in `Some`.

## 6. Batch Validation with Traversable

[Traversable](../reference/foldable-traversable.md) provides all-or-nothing semantics: apply a fallible function to every element in a collection, and if any element fails, the entire result is `None`.

``` rust
fn parse_u16(s: String) -> Option<u16> {
    s.parse().ok()
}

fn validate_ports(ports: Vec<&str>) -> Option<Vec<u16>> {
    VecF::traverse::<OptionF, _, _, _>(
        ports.into_iter().map(String::from).collect(),
        parse_u16,
    )
}
```

`VecF::traverse` maps `parse_u16` over each element and collects the results. If every element parses successfully, the result is `Some(vec![...])`. If any element fails, the result is `None`:

``` rust
let good = validate_ports(vec!["80", "443", "8080"]);
// => Some([80, 443, 8080])

let bad = validate_ports(vec!["80", "not_a_port", "8080"]);
// => None
```

This is strictly stronger than filtering out failures — it guarantees that either *all* values are valid or the caller knows something went wrong.

## 7. Config Summary with Foldable and Monoid

[Foldable](../reference/foldable-traversable.md) provides structural traversal, and [Monoid](../reference/algebraic.md) provides an identity element and associative combination. Together, `fold_map` transforms each element and concatenates the results:

``` rust
fn summarize_keys(keys: Vec<&str>) -> String {
    VecF::fold_map(
        keys.into_iter().map(String::from).collect::<Vec<_>>(),
        |key| {
            match resolve(&key) {
                Some(val) => format!("  {} = {}\n", key, val),
                None => format!("  {} = <missing>\n", key),
            }
        },
    )
}
```

For `String`, the Monoid instance uses the empty string as the identity and string concatenation as the combining operation. The result is a single string summarizing all resolved (or missing) configuration keys.

## 8. The `main` Function

The `main` function exercises each section and prints the results:

``` rust
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

    // 3. do_! for independent lookups
    println!("\n--- Connection string (do_!) ---");
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
        "DB_HOST", "DB_PORT", "DB_NAME", "MAX_CONNECTIONS", "TIMEOUT_MS", "MISSING",
    ]);
    print!("{}", summary);
}
```

## Run It

From the workspace root:

``` rust
cargo run -p karpal-std --example config_pipeline
```

## Traits Used

| Trait                | Purpose in this example                                        | Reference                                                        |
|----------------------|----------------------------------------------------------------|------------------------------------------------------------------|
| `Alt`                | Fallback chains across config sources                          | [Alt Family](../reference/alt-family.md)                       |
| `Monad` (via `do_!`) | Sequential composition of dependent lookups                    | [Functor Family](../reference/functor-family.md)               |
| `Traversable`        | All-or-nothing batch validation                                | [Foldable & Traversable](../reference/foldable-traversable.md) |
| `Foldable`           | Structural traversal with `fold_map`                           | [Foldable & Traversable](../reference/foldable-traversable.md) |
| `Monoid`             | String concatenation as the combining operation for `fold_map` | [Semigroup & Monoid](../reference/algebraic.md)                |


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


