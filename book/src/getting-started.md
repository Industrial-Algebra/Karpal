# Getting Started

## Installation

Add Karpal to your `Cargo.toml`:

```toml
[dependencies]
karpal-std = "0.7"
```

Or use individual crates:

```toml
[dependencies]
karpal-core = "0.7"
karpal-optics = "0.7"
karpal-proof = "0.7"
```

## Your First Functor

```rust
use karpal_core::{Functor, hkt::OptionF};

let result: Option<i32> = OptionF::fmap(Some(21), |x| x * 2);
assert_eq!(result, Some(42));
```

## Flatten Nested Error Handling with `do_!`

```rust
use karpal_core::{do_, Monad, hkt::OptionF};

fn process(input: &str) -> Option<String> {
    do_! { OptionF;
        id = parse_id(input);
        user = lookup_user(id);
        role = check_permissions(&user);
        Some(format!("{} logged in as {:?}", user, role))
    }
}
```

## Validate a Batch with Traversable

```rust
use karpal_core::{Traversable, hkt::{OptionF, VecF}};

let raw = vec!["10", "20", "30"];
let parsed: Option<Vec<i32>> = VecF::traverse::<OptionF, _, _, _>(raw, |s| s.parse().ok());
assert_eq!(parsed, Some(vec![10, 20, 30]));
```

## Optics — First-Class Field Accessors

```rust
use karpal_optics::Lens;

let reading_lens = Lens::new(|s: &Sensor| s.reading, |s, r| Sensor { reading: r, ..s });
let calibrated = reading_lens.over(sensor, |r| r * 1.02 + 0.5);
```

## Nightly Rust

Karpal requires nightly Rust due to GAT-based HKT encoding:

```sh
rustup default nightly
```
