# karpal-free

Free constructions for the [Industrial Algebra](https://github.com/Industrial-Algebra) ecosystem.

## What's included

- **Coyoneda** — the free functor. Makes any type constructor into a `Functor` by deferring `fmap` as function composition. Only requires `F: Functor` when lowering back.
- **Yoneda** — the Yoneda lemma as a data type. Wraps `F<A>` in CPS form `forall B. (A -> B) -> F<B>`, enabling O(1) map fusion.

## Usage

```rust
use karpal_free::{Coyoneda, Yoneda};
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

## Features

- `std` (default) — enables `alloc` and standard library support
- `alloc` — enables heap-allocated types (`Box<dyn Fn>`)
- With `default-features = false`, the crate is `no_std` compatible (but Coyoneda/Yoneda require `alloc`)

## License

MIT OR Apache-2.0
