# Karpal

A comprehensive algebraic structures library for Rust, built on
Higher-Kinded Types (HKTs) via GATs.

Karpal provides HKT encoding, a full functor hierarchy (Functor through Monad,
Alt/Plus/Alternative, Foldable, Traversable, and more), algebraic typeclasses
(Semigroup, Monoid), a profunctor hierarchy (Profunctor, Strong, Choice),
profunctor optics (Lens, Prism, composition), and `do_!`/`ado_!` notation macros — all with
`no_std` support and property-based law verification.

## Workspace

| Crate | Description |
|-------|-------------|
| [`karpal-core`](karpal-core/) | HKT encoding, functor hierarchy, Semigroup, Monoid, macros |
| [`karpal-profunctor`](karpal-profunctor/) | Profunctor, Strong, Choice, FnP |
| [`karpal-optics`](karpal-optics/) | Profunctor optics (Lens, Prism, composition) |
| [`karpal-std`](karpal-std/) | Standard prelude re-exports |

`karpal-core` and `karpal-profunctor` are `no_std` compatible with optional
`std`/`alloc` feature gates.

## Why Karpal?

Rust has `Option::map`, `Result::and_then`, and `Iterator::collect`. They work
great — but they're ad-hoc. Every container re-invents the same patterns with
slightly different names, and there's no way to write a function that's generic
over "any container that supports mapping" or "any container that supports
sequencing effects."

Karpal gives those patterns names and laws, so you can abstract over them.

### Flatten nested error handling with `do_!`

Deeply nested `.and_then()` chains are a common Rust pain point:

```rust
// Without do_! — rightward drift with every step
fn process(input: &str) -> Option<String> {
    parse_id(input).and_then(|id| {
        lookup_user(id).and_then(|user| {
            check_permissions(&user).and_then(|role| {
                Some(format!("{} logged in as {:?}", user.name, role))
            })
        })
    })
}

// With do_! — reads top-to-bottom, each step can use previous bindings
fn process(input: &str) -> Option<String> {
    do_! { OptionF;
        id = parse_id(input);
        user = lookup_user(id);
        role = check_permissions(&user);
        Some(format!("{} logged in as {:?}", user.name, role))
    }
}
```

### Validate an entire batch — or fail fast

`Traversable` turns a `Vec<Option<T>>` into an `Option<Vec<T>>`: either
every element succeeds, or the whole thing fails. No manual loop, no
early-return boilerplate:

```rust
use karpal_core::Traversable;
use karpal_core::hkt::{OptionF, VecF};

let raw = vec!["10", "20", "30"];
let parsed: Option<Vec<i32>> = VecF::traverse::<OptionF, _, _, _>(
    raw,
    |s| s.parse::<i32>().ok(),
);
assert_eq!(parsed, Some(vec![10, 20, 30]));

// One bad value → entire result is None
let raw = vec!["10", "nope", "30"];
let parsed: Option<Vec<i32>> = VecF::traverse::<OptionF, _, _, _>(
    raw,
    |s| s.parse::<i32>().ok(),
);
assert_eq!(parsed, None);
```

### Combine independent results with `ado_!`

When computations don't depend on each other, `ado_!` makes it clear
they're independent — unlike `do_!`/`and_then` which always imply
sequencing:

```rust
use karpal_core::{ado_, Applicative};
use karpal_core::hkt::OptionF;

fn load_config(env: &[(&str, &str)]) -> Option<String> {
    let find = |key: &str| env.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string());
    ado_! { OptionF;
        host = find("DB_HOST");
        port = find("DB_PORT");
        name = find("DB_NAME");
        yield format!("postgres://{}:{}/{}", host, port, name)
    }
}

let env = vec![("DB_HOST", "localhost"), ("DB_PORT", "5432"), ("DB_NAME", "app")];
assert_eq!(load_config(&env), Some("postgres://localhost:5432/app".into()));

// Missing any key → None, no partial results
let env = vec![("DB_HOST", "localhost")];
assert_eq!(load_config(&env), None);
```

### Fallback chains with `Alt`

Try a sequence of strategies, taking the first success:

```rust
use karpal_core::Alt;
use karpal_core::hkt::OptionF;

fn resolve_setting(overrides: &str, config: &str, default: i32) -> Option<i32> {
    let from_override = overrides.parse().ok();
    let from_config = config.parse().ok();
    let from_default = Some(default);

    OptionF::alt(OptionF::alt(from_override, from_config), from_default)
}

assert_eq!(resolve_setting("42", "10", 0), Some(42));  // override wins
assert_eq!(resolve_setting("bad", "10", 0), Some(10)); // fallback to config
assert_eq!(resolve_setting("bad", "bad", 0), Some(0)); // fallback to default
```

### Aggregate with `Foldable` and `Monoid`

Summarize a collection into any `Monoid` — the fold knows nothing about
the specific type, just that it has an associative `combine` and an `empty`:

```rust
use karpal_core::{Foldable, Monoid, Semigroup};
use karpal_core::hkt::VecF;

struct Stats { count: i32, total: i32 }

impl Semigroup for Stats {
    fn combine(self, other: Self) -> Self {
        Stats { count: self.count + other.count, total: self.total + other.total }
    }
}
impl Monoid for Stats {
    fn empty() -> Self { Stats { count: 0, total: 0 } }
}

let orders = vec![150, 89, 210, 45];
let stats: Stats = VecF::fold_map(orders, |price| Stats { count: 1, total: price });
assert_eq!(stats.count, 4);
assert_eq!(stats.total, 494);
```

### Profunctor optics — first-class field accessors

A `Lens` + `transform` produces a reusable, first-class function that
modifies a field deep inside a struct. You can store it, pass it around,
or compose it with other functions — something you can't do with
a plain `struct.field = value`:

```rust
use karpal_optics::Lens;
use karpal_profunctor::FnP;

struct Sensor { id: u32, reading: f64, location: String }

let reading_lens = Lens::new(
    |s: &Sensor| s.reading,
    |s, r| Sensor { reading: r, ..s },
);

// Build a reusable calibration function — it's just a Box<dyn Fn>
let calibrate: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 1.02 + 0.5);
let calibrate_sensor = reading_lens.transform::<FnP>(calibrate);

// Apply it to any sensor — the function carries the "which field" knowledge
let raw = Sensor { id: 1, reading: 98.6, location: "Lab A".into() };
let calibrated = calibrate_sensor(raw);
assert!((calibrated.reading - 101.072).abs() < 0.001);

// The same lens also gives you direct get/set/over
let s = Sensor { id: 2, reading: 50.0, location: "Lab B".into() };
assert_eq!(reading_lens.get(&s), 50.0);
let zeroed = reading_lens.set(s, 0.0);
assert_eq!(zeroed.reading, 0.0);
```

### Prism — pattern-matching as a value

A `Prism` focuses on one variant of an enum, the dual of `Lens` for sum types:

```rust
use karpal_optics::Prism;

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

let circle = Prism::new(
    |s| match s {
        Shape::Circle(r) => Ok(r),
        Shape::Rectangle(w, h) => Err(Shape::Rectangle(w, h)),
    },
    Shape::Circle,
);

// preview — extract focus if variant matches
assert_eq!(circle.preview(&Shape::Circle(5.0)), Some(5.0));
assert_eq!(circle.preview(&Shape::Rectangle(3.0, 4.0)), None);

// over — modify focus only if matched, pass through otherwise
let doubled = circle.over(Shape::Circle(5.0), |r| r * 2.0);
assert_eq!(doubled, Shape::Circle(10.0));

// review — construct the variant
assert_eq!(circle.review(7.0), Shape::Circle(7.0));
```

### Lens composition — focus deep into nested structs

Chain lenses with `.then()` to focus multiple levels deep:

```rust
use karpal_optics::Lens;

struct Company { name: String, ceo: Person }
struct Person  { name: String, age: u32 }

let ceo = Lens::new(
    |c: &Company| c.ceo.clone(),
    |c, ceo| Company { ceo, ..c },
);
let age = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

// Compose: Company → ceo → age
let ceo_age = ceo.then(age);

let co = Company {
    name: "Acme".into(),
    ceo: Person { name: "Alice".into(), age: 30 },
};

assert_eq!(ceo_age.get(&co), 30);
let updated = ceo_age.over(co, |a| a + 1);
assert_eq!(updated.ceo.age, 31);
```

## Requirements

- **Nightly Rust** (edition 2024) — pinned via `rust-toolchain.toml`

## Development

```sh
# Set up pre-commit hooks
./scripts/setup-hooks.sh

# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
```

## License

MIT OR Apache-2.0
