# karpal-effect

Monad transformers and effect system for Rust, built on the Karpal HKT encoding.

## What's inside

### Static-bound functor hierarchy

Parallel trait hierarchy with `'static` bounds to support types using
`Box<dyn Fn>` internally (e.g., reader and state transformers):

| Trait | Purpose |
|-------|---------|
| `FunctorSt` | `fmap` with `A: 'static, B: 'static` |
| `ApplicativeSt` | `pure` with `A: 'static` |
| `ChainSt` | `chain` (monadic bind) with `A: 'static, B: 'static` |

Instances: `OptionF`, `ResultF<E>`, `VecF`, `IdentityF`.

### Monad transformers

| Transformer | Representation | Effect |
|-------------|---------------|--------|
| `ExceptTF<E, M>` | `M::Of<Result<A, E>>` | Error handling |
| `WriterTF<W, M>` | `M::Of<(A, W)>` | Log accumulation |
| `ReaderTF<E, M>` | `Box<dyn Fn(E) -> M::Of<A>>` | Shared environment |
| `StateTF<S, M>` | `Box<dyn Fn(S) -> M::Of<(S, A)>>` | Mutable state |

Each transformer provides standalone helper functions:

```rust
use karpal_effect::{ReaderTF, reader_t_pure, reader_t_chain, reader_t_ask};
use karpal_core::hkt::OptionF;

type ReaderOpt<E, A> = <ReaderTF<E, OptionF> as karpal_core::hkt::HKT>::Of<A>;

// Build a reader that looks up a key in the environment
let lookup = reader_t_chain::<&[(&str, &str)], OptionF, &str, String>(
    reader_t_ask::<&[(&str, &str)], OptionF>(),
    |env| reader_t_pure::<_, OptionF, _>(
        env.iter().find(|(k, _)| *k == "host").map(|(_, v)| v.to_string())
            .unwrap_or_default()
    ),
);
```

### MonadTrans

The `MonadTrans` trait lifts computations from an inner monad into a
transformer stack:

```rust
use karpal_effect::MonadTrans;
use karpal_core::hkt::HKT;

// ExceptTF::lift(some_option) wraps Option<A> into Option<Result<A, E>>
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Enables `VecF` instances and transformer implementations |
| `alloc` | no      | Same instances via `alloc` (for `no_std`) |

## License

MIT OR Apache-2.0
