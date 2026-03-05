# karpal-arrow

A complete category and arrow hierarchy for Rust: Semigroupoid, Category, Arrow,
ArrowChoice, ArrowApply, ArrowLoop, ArrowZero, ArrowPlus, and three concrete
implementations.

## What's inside

### Trait hierarchy

```
HKT2
 └→ Semigroupoid          compose(f, g)
     └→ Category           id()
         └→ Arrow           arr(f), first, second, split, fanout
              ├→ ArrowChoice    left, right, splat, fanin
              ├→ ArrowApply     app  (≅ Monad)
              ├→ ArrowLoop      loop_arrow  (D: Default)
              └→ ArrowZero      zero_arrow
                   └→ ArrowPlus  plus(f, g)
```

### Semigroupoid and Category

Composable morphisms with an identity:

```rust
use karpal_arrow::{Semigroupoid, Category, FnA};

let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);

// compose: apply g then f
let fg = FnA::compose(f, g);
assert_eq!(fg(3), 7); // (3 * 2) + 1

// identity
let id = FnA::id::<i32>();
assert_eq!(id(42), 42);
```

### Arrow

Lift pure functions and operate on products:

```rust
use karpal_arrow::{Arrow, Semigroupoid, FnA};

// arr: lift a function into an arrow
let double = FnA::arr(|n: i32| n * 2);
let show = FnA::arr(|n: i32| format!("result: {}", n));
let pipeline = FnA::compose(show, double);
assert_eq!(pipeline(21), "result: 42");

// first: apply to the first component of a pair
let f = FnA::first::<i32, i32, &str>(FnA::arr(|x| x * 2));
assert_eq!(f((5, "hi")), (10, "hi"));

// split (***): two arrows in parallel
let f = FnA::split(
    FnA::arr(|n: i32| n * 2),
    FnA::arr(|s: String| s.len()),
);
assert_eq!(f((5, "hello".into())), (10, 5));

// fanout (&&&): feed one input to two arrows
let bounds = FnA::fanout(
    FnA::arr(|n: i32| n - 5),
    FnA::arr(|n: i32| n + 5),
);
assert_eq!(bounds(100), (95, 105));
```

### ArrowChoice

Route through sum types (`Result<L, R>`):

```rust
use karpal_arrow::{ArrowChoice, Arrow, FnA};

// left: route Ok through the arrow, pass Err through
let f = FnA::left::<i32, i32, &str>(FnA::arr(|x| x * 2));
assert_eq!(f(Ok(5)), Ok(10));
assert_eq!(f(Err("nope")), Err("nope"));

// fanin (|||): merge two arrows, one for each branch
let handle: Box<dyn Fn(Result<i32, String>) -> String> = FnA::fanin(
    FnA::arr(|n: i32| format!("ok: {}", n)),
    FnA::arr(|e: String| format!("err: {}", e)),
);
assert_eq!(handle(Ok(42)), "ok: 42");
assert_eq!(handle(Err("bad".into())), "err: bad");
```

### ArrowApply

Apply arrows from within a computation (equivalent in power to Monad):

```rust
use karpal_arrow::{ArrowApply, FnA};

let app = FnA::app::<i32, i32>();
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
assert_eq!(app((double, 5)), 10);
```

### ArrowLoop

Fixed-point combinator with `D: Default` as the initial feedback seed:

```rust
use karpal_arrow::{ArrowLoop, FnA};

let f = FnA::loop_arrow::<i32, i32, i32>(
    Box::new(|(a, d)| (a + d, d)),
);
assert_eq!(f(5), 5); // 5 + 0 (i32::default())
```

### Concrete implementations

| Type | Representation | Traits |
|------|---------------|--------|
| **FnA** | `Box<dyn Fn(A) -> B>` | Semigroupoid through ArrowLoop (all except Zero/Plus) |
| **KleisliF\<M\>** | `Box<dyn Fn(A) -> M::Of<B>>` | Full hierarchy; ArrowZero/ArrowPlus when `M: Plus` |
| **CokleisliF\<W\>** | `Box<dyn Fn(W::Of<A>) -> B>` | Semigroupoid + Category (via `impl_cokleisli!` macro) |

### KleisliF — arrows for monadic effects

```rust
use karpal_arrow::{Arrow, ArrowPlus, ArrowZero, Semigroupoid, KleisliF};
use karpal_core::hkt::OptionF;

type KOpt = KleisliF<OptionF>;

// Compose monadic functions with automatic short-circuiting
let safe_recip: Box<dyn Fn(f64) -> Option<f64>> =
    Box::new(|x| if x != 0.0 { Some(1.0 / x) } else { None });
let double: Box<dyn Fn(f64) -> Option<f64>> = Box::new(|x| Some(x * 2.0));

let pipeline = KOpt::compose(double, safe_recip);
assert_eq!(pipeline(4.0), Some(0.5));  // 1/4 * 2
assert_eq!(pipeline(0.0), None);        // short-circuits

// ArrowPlus: try first, fall back to second
let primary: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
let fallback: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 10));
let with_fallback = KOpt::plus(primary, fallback);
assert_eq!(with_fallback(5), Some(50));
```

### CokleisliF — arrows from comonadic contexts

```rust
use karpal_arrow::{Semigroupoid, Category, CokleisliF};
use karpal_core::hkt::{NonEmptyVec, NonEmptyVecF};

type CoNev = CokleisliF<NonEmptyVecF>;

// Extract head (identity for NonEmptyVec comonad)
let id = CoNev::id::<i32>();
let nev = NonEmptyVec::new(42, vec![1, 2]);
assert_eq!(id(nev), 42);

// Compose context-aware functions
let sum: Box<dyn Fn(NonEmptyVec<i32>) -> i32> =
    Box::new(|nev| nev.iter().sum());
let show: Box<dyn Fn(NonEmptyVec<i32>) -> String> =
    Box::new(|nev| format!("head={}", nev.head));
let pipeline = CoNev::compose(show, sum);
// extend applies sum to each suffix, then show reads the head
```

CokleisliF requires the `impl_cokleisli!` macro because `W::Of<A>: Clone`
cannot be expressed generically with GATs. Pre-generated instances:
`IdentityF`, `OptionF`, `NonEmptyVecF`. Use `impl_cokleisli_env!` for `EnvF<E>`.

## Design notes

- **`Clone + 'static` bounds**: All type parameters require `Clone + 'static`.
  `'static` for `Box<dyn Fn>` representations; `Clone` for KleisliF's `first`
  (the passthrough value must be cloned inside the monadic bind closure).

- **ArrowLoop and strict evaluation**: Haskell's `loop` relies on laziness to
  tie the knot. Rust is strict, so `loop_arrow` requires `D: Default` and uses
  single-pass evaluation with `D::default()` as the initial feedback seed.

- **Operator naming**: Haskell's symbolic operators are mapped to descriptive names:
  `***` → `split`, `&&&` → `fanout`, `+++` → `splat`, `|||` → `fanin`.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Implies `alloc`; enables FnA, KleisliF, CokleisliF |
| `alloc` | no      | Same instances (for `no_std` with allocator) |

## License

MIT OR Apache-2.0
