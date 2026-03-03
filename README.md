# Karpal

Higher-Kinded Type (HKT) library for the **Industrial Algebra** ecosystem.

Karpal provides the algebraic foundations — HKT encoding, core typeclasses,
profunctor hierarchy, and profunctor optics — that power downstream crates
like Orlando (optics), Cliffy (FRP), and Amari (math).

## Workspace

| Crate | Description |
|-------|-------------|
| [`karpal-core`](karpal-core/) | HKT encoding, Functor, Semigroup, Monoid |
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
