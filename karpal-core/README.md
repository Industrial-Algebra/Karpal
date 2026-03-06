# karpal-core

Core algebraic structures for Rust: HKT encoding, a full functor hierarchy
(Functor through Monad), Semigroup, Monoid, and `do_!`/`ado_!` notation macros.

## What's inside

### HKT encoding

GAT-based Higher-Kinded Type encoding with zero dependencies:

```rust
use karpal_core::hkt::{HKT, HKT2, OptionF, ResultF, VecF, ResultBF, TupleF};

// HKT  — OptionF::Of<T> = Option<T>, ResultF<E>::Of<T> = Result<T, E>, VecF::Of<T> = Vec<T>
// HKT2 — ResultBF::P<A, B> = Result<B, A>, TupleF::P<A, B> = (A, B)
```

### Functor hierarchy

| Trait | Supertrait | Instances |
|-------|-----------|-----------|
| Functor | HKT | OptionF, ResultF, VecF |
| Apply | Functor | OptionF, ResultF, VecF |
| Applicative | Apply | OptionF, ResultF, VecF |
| Chain | Apply | OptionF, ResultF, VecF |
| Monad | Applicative + Chain | blanket impl |
| Alt | Functor | OptionF, ResultF, VecF |
| Plus | Alt | OptionF, VecF |
| Alternative | Applicative + Plus | blanket impl |
| Foldable | HKT | OptionF, ResultF, VecF |
| Traversable | Functor + Foldable | OptionF, ResultF, VecF |
| FunctorFilter | Functor | OptionF, VecF |
| Selective | Applicative | OptionF |
| Contravariant | HKT | PredicateF |
| Bifunctor | HKT2 | ResultBF, TupleF |
| NaturalTransformation | HKT | OptionToVec, VecHeadToOption |

### Chain and `do_!` — sequential, dependent computations

When each step depends on the result of the previous one, `do_!` replaces
nested `.and_then()` chains with flat, readable notation:

```rust
use karpal_core::{do_, Applicative};
use karpal_core::hkt::OptionF;

// A multi-step lookup where each step can fail
fn resolve(items: &[(u32, &str)], aliases: &[(&str, &str)], id: u32) -> Option<String> {
    do_! { OptionF;
        name = items.iter().find(|(i, _)| *i == id).map(|(_, n)| *n);
        alias = aliases.iter().find(|(n, _)| *n == name).map(|(_, a)| *a);
        OptionF::pure(format!("{} ({})", name, alias))
    }
}

let items = vec![(1, "alice"), (2, "bob")];
let aliases = vec![("alice", "A"), ("bob", "B")];
assert_eq!(resolve(&items, &aliases, 1), Some("alice (A)".into()));
assert_eq!(resolve(&items, &aliases, 99), None); // first step fails → short-circuits
```

### Applicative and `ado_!` — independent computations combined

Unlike `do_!`, `ado_!` makes it explicit that the computations don't
depend on each other. All bindings are evaluated independently, then
combined in the `yield`:

```rust
use karpal_core::{ado_, Applicative};
use karpal_core::hkt::OptionF;

// All three lookups are independent — if any fails, the whole thing fails
fn load_config(env: &[(&str, &str)]) -> Option<String> {
    let find = |key: &str| env.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string());
    ado_! { OptionF;
        host = find("HOST");
        port = find("PORT");
        yield format!("{}:{}", host, port)
    }
}

let env = vec![("HOST", "localhost"), ("PORT", "5432")];
assert_eq!(load_config(&env), Some("localhost:5432".into()));

let env = vec![("HOST", "localhost")]; // PORT missing
assert_eq!(load_config(&env), None);
```

### Traversable — batch operations that fail fast

"Run this fallible operation on every element; if any one fails, the
whole batch fails." `traverse` does this without manual loops or
early-return boilerplate:

```rust
use karpal_core::Traversable;
use karpal_core::hkt::{OptionF, VecF};

let ids = vec!["100", "200", "300"];
let parsed: Option<Vec<u64>> = VecF::traverse::<OptionF, _, _, _>(
    ids, |s| s.parse::<u64>().ok(),
);
assert_eq!(parsed, Some(vec![100, 200, 300]));

// One invalid entry poisons the whole batch
let ids = vec!["100", "not_a_number", "300"];
let parsed: Option<Vec<u64>> = VecF::traverse::<OptionF, _, _, _>(
    ids, |s| s.parse::<u64>().ok(),
);
assert_eq!(parsed, None);
```

### Alt — fallback chains

Try multiple strategies in order, taking the first success:

```rust
use karpal_core::Alt;
use karpal_core::hkt::OptionF;

fn resolve_timeout(flag: Option<u64>, env: Option<u64>, default: u64) -> u64 {
    // flag overrides env, env overrides default
    OptionF::alt(OptionF::alt(flag, env), Some(default)).unwrap()
}

assert_eq!(resolve_timeout(Some(5), Some(30), 60), 5);
assert_eq!(resolve_timeout(None, Some(30), 60), 30);
assert_eq!(resolve_timeout(None, None, 60), 60);
```

### Foldable — generic aggregation with Monoid

Summarize any foldable structure using any `Monoid`. The fold knows
nothing about the container or the summary type — it just needs
`combine` and `empty`:

```rust
use karpal_core::{Foldable, Monoid, Semigroup};
use karpal_core::hkt::VecF;

struct Histogram { buckets: Vec<(String, u32)> }

impl Semigroup for Histogram {
    fn combine(mut self, other: Self) -> Self {
        for (key, count) in other.buckets {
            if let Some(entry) = self.buckets.iter_mut().find(|(k, _)| *k == key) {
                entry.1 += count;
            } else {
                self.buckets.push((key, count));
            }
        }
        self
    }
}
impl Monoid for Histogram {
    fn empty() -> Self { Histogram { buckets: vec![] } }
}

let events = vec!["click", "view", "click", "view", "view"];
let hist: Histogram = VecF::fold_map(events, |e| {
    Histogram { buckets: vec![(e.to_string(), 1)] }
});
let clicks = hist.buckets.iter().find(|(k, _)| k == "click").unwrap().1;
let views = hist.buckets.iter().find(|(k, _)| k == "view").unwrap().1;
assert_eq!(clicks, 2);
assert_eq!(views, 3);
```

### Bifunctor — map both sides of a Result or tuple

```rust
use karpal_core::Bifunctor;
use karpal_core::hkt::{ResultBF, TupleF};

// Map the error to a structured type and the success to a display string
let result: Result<&str, &str> = Err("connection refused");
let mapped = ResultBF::bimap(result, |e| format!("NetworkError: {}", e), |v| v.len());
assert_eq!(mapped, Err("NetworkError: connection refused".to_string()));

// Transform both halves of a key-value pair
let entry = ("temperature", 98.6_f64);
let display = TupleF::bimap(entry, |k| k.to_uppercase(), |v| format!("{:.1}F", v));
assert_eq!(display, ("TEMPERATURE".to_string(), "98.6F".to_string()));
```

### FunctorFilter — map and filter in one pass

```rust
use karpal_core::FunctorFilter;
use karpal_core::hkt::VecF;

// Parse valid entries, silently skip malformed ones
let raw = vec!["42", "bad", "7", "", "13"];
let valid: Vec<i32> = VecF::filter_map(raw, |s| s.parse().ok());
assert_eq!(valid, vec![42, 7, 13]);
```

### Semigroup / Monoid

```rust
use karpal_core::{Semigroup, Monoid};

// Combine anything associative
assert_eq!(vec![1, 2].combine(vec![3, 4]), vec![1, 2, 3, 4]);
assert_eq!(Some(3i32).combine(Some(4)), Some(7));

// Monoid adds an identity element
assert_eq!(Vec::<i32>::empty().combine(vec![1, 2]), vec![1, 2]);
```

Instances: all numeric types (additive), `String`, `Vec<T>`, `Option<T: Semigroup>`.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Enables `Vec`, `String`, `PredicateF` instances |
| `alloc` | no      | Same instances via `alloc` (for `no_std`) |

## License

MIT OR Apache-2.0
