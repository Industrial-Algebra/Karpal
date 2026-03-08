# karpal-recursion

Recursion schemes for Rust: structured, composable patterns for folding and
unfolding recursive data structures.

## What's inside

### Fixed points

| Type | Description |
|------|-------------|
| `Fix<F>` | Fixed point of a functor — ties the recursive knot with `Rc` for unconditional `Clone` |
| `Mu<F>` | Least fixed point (type alias for `Fix<F>`) |
| `Nu<F, Seed>` | Greatest fixed point — seed + coalgebra for lazy observation |

### Recursion schemes

| Function | Category | Description |
|----------|----------|-------------|
| `cata` | Fold | Catamorphism — fold bottom-up with `F<A> -> A` |
| `ana` | Unfold | Anamorphism — unfold top-down with `A -> F<A>` |
| `hylo` | Refold | Hylomorphism — unfold then fold, no intermediate `Fix` |
| `para` | Fold+ | Paramorphism — fold with access to original subterms |
| `apo` | Unfold+ | Apomorphism — unfold with early termination via `Either` |
| `histo` | Fold++ | Histomorphism — fold with full history via `Cofree` |
| `futu` | Unfold++ | Futumorphism — multi-step unfold via `Free` |
| `zygo` | Composite | Zygomorphism — fold with auxiliary fold in parallel |
| `chrono` | Composite | Chronomorphism — `futu` ; `histo` in a single pass |

### Example: natural numbers with cata and ana

```rust
use karpal_recursion::{Fix, cata, ana};
use karpal_core::hkt::OptionF;

// None = Zero, Some(n) = Succ(n)
// Build "5" by unfolding from a seed
let five: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    5,
);

// Fold to count the layers
let count = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(n) => n + 1,
    },
    five,
);
assert_eq!(count, 5);
```

### Example: Fibonacci with histomorphism

```rust
use karpal_recursion::{Fix, ana, histo};
use karpal_core::hkt::OptionF;

let nat = |n: u32| ana(|s: u32| if s == 0 { None } else { Some(s - 1) }, n);

let fib = histo::<OptionF, u64>(
    |layer| match layer {
        None => 0,                       // fib(0) = 0
        Some(cofree) => {
            let prev = cofree.head;      // fib(n-1)
            match cofree.tail.as_ref() {
                None => 1,               // fib(1) = 1
                Some(gc) => prev + gc.head,  // fib(n-1) + fib(n-2)
            }
        }
    },
    nat(10),
);
assert_eq!(fib, 55);
```

## Design notes

- **`Rc` in Fix** — enables unconditional `Clone` without coinductive proofs.
  Essential for paramorphism, which needs to both keep and consume subterms.
- **Reference algebras** — `histo` and `chrono` take `&F::Of<Cofree<F, A>>`
  to avoid needing `Clone` on `Cofree`.
- **`Either<L, R>`** — local sum type used by apomorphism for early termination.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Implies `alloc` |
| `alloc` | no      | Enables `Fix`, `Nu`, and all schemes (requires heap allocation) |

## License

MIT OR Apache-2.0
