# karpal-free

Free constructions for the [Industrial Algebra](https://github.com/Industrial-Algebra) ecosystem.

## What's included

| Type | Description |
|------|-------------|
| `Coyoneda<F, A>` | Free functor — makes any type constructor into a Functor by deferring `fmap` |
| `Yoneda<F, A>` | Yoneda lemma as a data type — O(1) map fusion via CPS |
| `Free<F, A>` | Free monad — `Pure(A) \| Roll(F<Free<F, A>>)` |
| `Cofree<F, A>` | Free comonad — annotated recursive structure with `head` + `tail` |
| `Freer<F, A>` | Freer monad — no `F: Functor` requirement (uses Coyoneda internally) |
| `Lan<G, H, A, B>` | Left Kan extension |
| `Ran` | Right Kan extension (trait-based) |
| `Codensity<F, A>` | CPS monad — no `F` bounds for `pure`/`fmap`/`chain` |
| `Density<W, A>` | CPS comonad — no `W` bounds for `extract`/`fmap` |
| `Day<F, G, A, B, C>` | Day convolution — combines two functors via a binary function |
| `FreeAp<F, A>` | Free applicative — supports static analysis via `count_effects` |
| `FreeAlt<F, A>` | Free alternative — `Vec<FreeAp<F, A>>` |

## Usage

```rust
use karpal_free::{Coyoneda, Yoneda, Free, Cofree};
use karpal_core::hkt::OptionF;

// Coyoneda: fmap without Functor
let co = Coyoneda::<OptionF, i32>::lift(Some(1))
    .fmap(|x| x + 1)
    .fmap(|x| x * 10);
assert_eq!(co.lower(), Some(20));

// Yoneda: O(1) map fusion
let yo = Yoneda::<OptionF, i32>::lift(Some(1))
    .fmap(|x| x + 1)
    .fmap(|x| x * 10);
assert_eq!(yo.lower(), Some(20));
```

### Free monad

```rust
use karpal_free::Free;
use karpal_core::hkt::OptionF;

// Pure values
let pure: Free<OptionF, i32> = Free::pure(42);

// Roll wraps one layer of the functor
let rolled: Free<OptionF, i32> = Free::roll(Some(Free::pure(42)));
```

## Features

- `std` (default) — enables `alloc` and standard library support
- `alloc` — enables heap-allocated types (`Box<dyn Fn>`)
- With `default-features = false`, the crate is `no_std` compatible (but most types require `alloc`)

## License

MIT OR Apache-2.0
