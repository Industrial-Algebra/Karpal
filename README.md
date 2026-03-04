# Karpal

A comprehensive algebraic structures library for Rust, built on
Higher-Kinded Types (HKTs) via GATs.

Karpal provides HKT encoding, a full functor hierarchy (Functor through Monad,
Alt/Plus/Alternative, Foldable, Traversable, and more), algebraic typeclasses
(Semigroup, Monoid), a profunctor hierarchy (Profunctor, Strong, Choice),
profunctor optics (Lens), and `do_!`/`ado_!` notation macros — all with
`no_std` support and property-based law verification.

## Workspace

| Crate | Description |
|-------|-------------|
| [`karpal-core`](karpal-core/) | HKT encoding, functor hierarchy, Semigroup, Monoid, macros |
| [`karpal-profunctor`](karpal-profunctor/) | Profunctor, Strong, Choice, FnP |
| [`karpal-optics`](karpal-optics/) | Profunctor optics (Lens) |
| [`karpal-std`](karpal-std/) | Standard prelude re-exports (stub) |

`karpal-core` and `karpal-profunctor` are `no_std` compatible with optional
`std`/`alloc` feature gates.

## Quick start

```rust
use karpal_core::{Functor, Semigroup, Monoid};
use karpal_core::hkt::{OptionF, VecF};

// Functor
let doubled = OptionF::fmap(Some(21), |x| x * 2);
assert_eq!(doubled, Some(42));

// Semigroup
let combined = vec![1, 2].combine(vec![3, 4]);
assert_eq!(combined, vec![1, 2, 3, 4]);

// Monoid
let empty = Vec::<i32>::empty();
assert_eq!(empty, vec![]);
```

### Applicative and Monad

```rust
use karpal_core::{Applicative, Chain};
use karpal_core::hkt::OptionF;

// Applicative
let result = OptionF::ap(Some(|x: i32| x + 1), Some(41));
assert_eq!(result, Some(42));

// Chain (flatMap)
let result = OptionF::chain(Some(3), |x| if x > 0 { Some(x * 2) } else { None });
assert_eq!(result, Some(6));
```

### do! / ado! macros

```rust
use karpal_core::{do_, ado_};
use karpal_core::hkt::OptionF;
use karpal_core::Applicative;

// Monadic do-notation
let result = do_! { OptionF;
    x = Some(1);
    y = Some(x + 1);
    OptionF::pure(x + y)
};
assert_eq!(result, Some(3));

// Applicative do-notation
let result = ado_! { OptionF;
    x = Some(10);
    y = Some(20);
    yield x + y
};
assert_eq!(result, Some(30));
```

### Profunctor optics

```rust
use karpal_optics::Lens;
use karpal_profunctor::FnP;

struct Person { name: String, age: u32 }

let age_lens = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let increment: Box<dyn Fn(u32) -> u32> = Box::new(|a| a + 1);
let birthday = age_lens.transform::<FnP>(increment);
let alice = Person { name: "Alice".into(), age: 30 };
let older = birthday(alice);
assert_eq!(older.age, 31);
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
