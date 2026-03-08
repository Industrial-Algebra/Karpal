# karpal-profunctor

A complete profunctor hierarchy for Rust: Profunctor, Strong, Choice, and the
canonical function-arrow instance.

## What's inside

### HKT2

Two-parameter type constructor for profunctors (defined in `karpal-core`,
re-exported here):

```rust
use karpal_profunctor::HKT2;

// trait HKT2 { type P<A, B>; }
```

### Profunctor

Contravariant in the first argument, covariant in the second:

```rust
use karpal_profunctor::{Profunctor, FnP};

let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
let f = FnP::dimap(
    |s: &str| s.len() as i32,
    |n: i32| n.to_string(),
    double,
);
assert_eq!(f("hello"), "10"); // len=5, *2=10, to_string
```

### Strong

Lifts `P<A, B>` into `P<(A, C), (B, C)>` — the key ingredient for lenses.

### Choice

Lifts `P<A, B>` into `P<Result<A, C>, Result<B, C>>` — the key ingredient for prisms.

### Traversing

Extends `Strong + Choice` with `wander` — the key ingredient for traversals.

### ForgetF\<R\>

A profunctor that forgets the output: `P<A, B> = Box<dyn Fn(A) -> R>`.
Implements `Profunctor`, `Strong`, and `Choice + Traversing` when `R: Monoid`.
Used by `Getter`, `Fold`, and read-only optics.

### TaggedF

A profunctor that ignores the input: `P<A, B> = B`.
Implements `Profunctor` and `Choice` but **not** `Strong` (enforces write-only).
Used by `Review`.

### FnP

The canonical profunctor instance: `Box<dyn Fn(A) -> B>`.
Implements `Profunctor`, `Strong`, `Choice`, and `Traversing`. Gated behind the `alloc` feature.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Implies `alloc`; enables `FnP` |
| `alloc` | no      | Enables `FnP` (for `no_std` with allocator) |

## License

MIT OR Apache-2.0
