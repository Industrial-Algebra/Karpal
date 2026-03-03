# karpal-core

Core algebraic structures for the Karpal ecosystem.

## What's inside

### HKT encoding

GAT-based Higher-Kinded Type encoding with zero dependencies:

```rust
use karpal_core::hkt::{HKT, OptionF, ResultF, VecF};

// OptionF::Of<T> = Option<T>
// ResultF<E>::Of<T> = Result<T, E>
// VecF::Of<T> = Vec<T>
```

### Functor

```rust
use karpal_core::{Functor, hkt::OptionF};

let result = OptionF::fmap(Some(21), |x| x * 2);
assert_eq!(result, Some(42));
```

Instances: `OptionF`, `ResultF<E>`, `VecF`.

### Semigroup

```rust
use karpal_core::Semigroup;

assert_eq!(3i32.combine(4), 7);
assert_eq!("hello ".to_string().combine("world".to_string()), "hello world");
assert_eq!(Some(3).combine(Some(4)), Some(7));
```

Instances: all numeric types (additive), `String`, `Vec<T>`, `Option<T: Semigroup>`.

### Monoid

```rust
use karpal_core::Monoid;

assert_eq!(i32::empty(), 0);
assert_eq!(String::empty(), "");
assert_eq!(Vec::<i32>::empty(), vec![]);
```

Instances match Semigroup instances.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Enables `Vec`, `String` instances |
| `alloc` | no      | Same instances via `alloc` (for `no_std`) |

## License

MIT OR Apache-2.0
