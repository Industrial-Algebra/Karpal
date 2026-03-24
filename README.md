# Karpal

A comprehensive algebraic structures library for Rust, built on
Higher-Kinded Types (HKTs) via GATs.

Karpal provides HKT encoding, a full functor hierarchy (Functor through Monad,
Alt/Plus/Alternative, Foldable, Traversable, and more), algebraic typeclasses
(Semigroup, Monoid, Group, Ring, Field), profunctor optics (Lens, Prism,
Traversal, Fold, and more), a category/arrow hierarchy, free constructions
(Free Monad, Cofree Comonad, Coyoneda, Day Convolution), recursion schemes
(cata, ana, hylo, histo, and more), adjunctions and advanced category theory
(ends, coends, dinatural transformations), monad transformers (ExceptT,
WriterT, ReaderT, StateT), and `do_!`/`ado_!` notation macros — all with
`no_std` support and property-based law verification.

## Workspace

| Crate | Description |
|-------|-------------|
| [`karpal-core`](karpal-core/) | HKT encoding, functor hierarchy, Semigroup, Monoid, newtype wrappers, macros |
| [`karpal-profunctor`](karpal-profunctor/) | Profunctor, Strong, Choice, Traversing, FnP |
| [`karpal-arrow`](karpal-arrow/) | Category/Arrow hierarchy, FnA, KleisliF, CokleisliF |
| [`karpal-optics`](karpal-optics/) | Profunctor optics (Iso, Lens, Prism, Traversal, Fold, Getter, Setter, Review) |
| [`karpal-free`](karpal-free/) | Free constructions (Coyoneda, Yoneda, Free, Cofree, Freer, Day, FreeAp, FreeAlt) |
| [`karpal-recursion`](karpal-recursion/) | Recursion schemes (Fix, cata, ana, hylo, para, apo, histo, futu, zygo, chrono) |
| [`karpal-algebra`](karpal-algebra/) | Abstract algebra (Group, Semiring, Ring, Field, Lattice, Module, VectorSpace) |
| [`karpal-effect`](karpal-effect/) | Monad transformers (ExceptT, WriterT, ReaderT, StateT) and static-bound functor hierarchy |
| [`karpal-proof`](karpal-proof/) | Algebraic law witnesses, rewrite witnesses, refinement types, and derive-based law verification |
| [`karpal-verify`](karpal-verify/) | External prover bridge: proof obligations, SMT-LIB2 export, structured Lean 4 integration, runners/reporting, and explicit trust model |
| [`karpal-std`](karpal-std/) | Standard prelude re-exports |

`karpal-core`, `karpal-profunctor`, `karpal-arrow`, `karpal-free`,
`karpal-recursion`, `karpal-algebra`, `karpal-effect`, `karpal-proof`,
and `karpal-verify` are `no_std` compatible with optional `std`/`alloc`
feature gates.

## Why Karpal?

Rust has `Option::map`, `Result::and_then`, and `Iterator::collect`. They work
great — but they're ad-hoc. Every container re-invents the same patterns with
slightly different names, and there's no way to write a function that's generic
over "any container that supports mapping" or "any container that supports
sequencing effects."

Karpal gives those patterns names and laws, so you can abstract over them.

### Flatten nested error handling with `do_!`

Deeply nested `.and_then()` chains are a common Rust pain point:

```rust
// Without do_! — rightward drift with every step
fn process(input: &str) -> Option<String> {
    parse_id(input).and_then(|id| {
        lookup_user(id).and_then(|user| {
            check_permissions(&user).and_then(|role| {
                Some(format!("{} logged in as {:?}", user.name, role))
            })
        })
    })
}

// With do_! — reads top-to-bottom, each step can use previous bindings
fn process(input: &str) -> Option<String> {
    do_! { OptionF;
        id = parse_id(input);
        user = lookup_user(id);
        role = check_permissions(&user);
        Some(format!("{} logged in as {:?}", user.name, role))
    }
}
```

### Validate an entire batch — or fail fast

`Traversable` turns a `Vec<Option<T>>` into an `Option<Vec<T>>`: either
every element succeeds, or the whole thing fails. No manual loop, no
early-return boilerplate:

```rust
use karpal_core::Traversable;
use karpal_core::hkt::{OptionF, VecF};

let raw = vec!["10", "20", "30"];
let parsed: Option<Vec<i32>> = VecF::traverse::<OptionF, _, _, _>(
    raw,
    |s| s.parse::<i32>().ok(),
);
assert_eq!(parsed, Some(vec![10, 20, 30]));

// One bad value → entire result is None
let raw = vec!["10", "nope", "30"];
let parsed: Option<Vec<i32>> = VecF::traverse::<OptionF, _, _, _>(
    raw,
    |s| s.parse::<i32>().ok(),
);
assert_eq!(parsed, None);
```

### Combine independent results with `ado_!`

When computations don't depend on each other, `ado_!` makes it clear
they're independent — unlike `do_!`/`and_then` which always imply
sequencing:

```rust
use karpal_core::{ado_, Applicative};
use karpal_core::hkt::OptionF;

fn load_config(env: &[(&str, &str)]) -> Option<String> {
    let find = |key: &str| env.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string());
    ado_! { OptionF;
        host = find("DB_HOST");
        port = find("DB_PORT");
        name = find("DB_NAME");
        yield format!("postgres://{}:{}/{}", host, port, name)
    }
}

let env = vec![("DB_HOST", "localhost"), ("DB_PORT", "5432"), ("DB_NAME", "app")];
assert_eq!(load_config(&env), Some("postgres://localhost:5432/app".into()));

// Missing any key → None, no partial results
let env = vec![("DB_HOST", "localhost")];
assert_eq!(load_config(&env), None);
```

### Fallback chains with `Alt`

Try a sequence of strategies, taking the first success:

```rust
use karpal_core::Alt;
use karpal_core::hkt::OptionF;

fn resolve_setting(overrides: &str, config: &str, default: i32) -> Option<i32> {
    let from_override = overrides.parse().ok();
    let from_config = config.parse().ok();
    let from_default = Some(default);

    OptionF::alt(OptionF::alt(from_override, from_config), from_default)
}

assert_eq!(resolve_setting("42", "10", 0), Some(42));  // override wins
assert_eq!(resolve_setting("bad", "10", 0), Some(10)); // fallback to config
assert_eq!(resolve_setting("bad", "bad", 0), Some(0)); // fallback to default
```

### Aggregate with `Foldable` and `Monoid`

Summarize a collection into any `Monoid` — the fold knows nothing about
the specific type, just that it has an associative `combine` and an `empty`:

```rust
use karpal_core::{Foldable, Monoid, Semigroup};
use karpal_core::hkt::VecF;

struct Stats { count: i32, total: i32 }

impl Semigroup for Stats {
    fn combine(self, other: Self) -> Self {
        Stats { count: self.count + other.count, total: self.total + other.total }
    }
}
impl Monoid for Stats {
    fn empty() -> Self { Stats { count: 0, total: 0 } }
}

let orders = vec![150, 89, 210, 45];
let stats: Stats = VecF::fold_map(orders, |price| Stats { count: 1, total: price });
assert_eq!(stats.count, 4);
assert_eq!(stats.total, 494);
```

### Profunctor optics — first-class field accessors

A `Lens` + `transform` produces a reusable, first-class function that
modifies a field deep inside a struct. You can store it, pass it around,
or compose it with other functions — something you can't do with
a plain `struct.field = value`:

```rust
use karpal_optics::Lens;
use karpal_profunctor::FnP;

struct Sensor { id: u32, reading: f64, location: String }

let reading_lens = Lens::new(
    |s: &Sensor| s.reading,
    |s, r| Sensor { reading: r, ..s },
);

// Build a reusable calibration function — it's just a Box<dyn Fn>
let calibrate: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 1.02 + 0.5);
let calibrate_sensor = reading_lens.transform::<FnP>(calibrate);

// Apply it to any sensor — the function carries the "which field" knowledge
let raw = Sensor { id: 1, reading: 98.6, location: "Lab A".into() };
let calibrated = calibrate_sensor(raw);
assert!((calibrated.reading - 101.072).abs() < 0.001);

// The same lens also gives you direct get/set/over
let s = Sensor { id: 2, reading: 50.0, location: "Lab B".into() };
assert_eq!(reading_lens.get(&s), 50.0);
let zeroed = reading_lens.set(s, 0.0);
assert_eq!(zeroed.reading, 0.0);
```

### Prism — pattern-matching as a value

A `Prism` focuses on one variant of an enum, the dual of `Lens` for sum types:

```rust
use karpal_optics::Prism;

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

let circle = Prism::new(
    |s| match s {
        Shape::Circle(r) => Ok(r),
        Shape::Rectangle(w, h) => Err(Shape::Rectangle(w, h)),
    },
    Shape::Circle,
);

// preview — extract focus if variant matches
assert_eq!(circle.preview(&Shape::Circle(5.0)), Some(5.0));
assert_eq!(circle.preview(&Shape::Rectangle(3.0, 4.0)), None);

// over — modify focus only if matched, pass through otherwise
let doubled = circle.over(Shape::Circle(5.0), |r| r * 2.0);
assert_eq!(doubled, Shape::Circle(10.0));

// review — construct the variant
assert_eq!(circle.review(7.0), Shape::Circle(7.0));
```

### Compose computations with Arrows

Arrows generalize functions into composable computation pipelines.
`FnA` wraps plain functions; `KleisliF` wraps effectful functions like
`A -> Option<B>`:

```rust
use karpal_arrow::{Arrow, ArrowChoice, Semigroupoid, FnA};

// Build a processing pipeline from small, composable pieces
let parse_int = FnA::arr(|s: String| s.parse::<i32>().unwrap_or(0));
let double = FnA::arr(|n: i32| n * 2);
let to_string = FnA::arr(|n: i32| format!("result: {}", n));

// Compose: parse → double → format
let pipeline = FnA::compose(to_string, FnA::compose(double, parse_int));
assert_eq!(pipeline("21".into()), "result: 42");

// fanout: feed one input to two arrows, collect both results
let bounds = FnA::fanout(
    FnA::arr(|n: i32| n - 5),
    FnA::arr(|n: i32| n + 5),
);
assert_eq!(bounds(100), (95, 105));

// ArrowChoice: route through sum types
let handle: Box<dyn Fn(Result<i32, String>) -> String> = FnA::fanin(
    FnA::arr(|n: i32| format!("ok: {}", n)),
    FnA::arr(|e: String| format!("err: {}", e)),
);
assert_eq!(handle(Ok(42)), "ok: 42");
assert_eq!(handle(Err("bad".into())), "err: bad");
```

Kleisli arrows lift monadic functions into the same Arrow interface,
so you get composition with short-circuiting for free:

```rust
use karpal_arrow::{Arrow, ArrowZero, ArrowPlus, Semigroupoid, KleisliF};
use karpal_core::hkt::OptionF;

type KOpt = KleisliF<OptionF>;

let safe_div = |d: i32| -> Box<dyn Fn(i32) -> Option<i32>> {
    Box::new(move |n| if d != 0 { Some(n / d) } else { None })
};
let add_one: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|n| Some(n + 1));

// Compose: divide by 2, then add 1 — short-circuits on None
let pipeline = KOpt::compose(add_one, safe_div(2));
assert_eq!(pipeline(10), Some(6));  // 10/2 + 1
assert_eq!(KOpt::compose(Box::new(|n| Some(n + 1)), safe_div(0))(10), None);

// ArrowPlus: try first arrow, fall back to second
let try_parse: Box<dyn Fn(String) -> Option<i32>> =
    Box::new(|s| s.parse().ok());
let fallback: Box<dyn Fn(String) -> Option<i32>> =
    Box::new(|_| Some(0));
let with_fallback = KOpt::plus(try_parse, fallback);
assert_eq!(with_fallback("42".into()), Some(42));
assert_eq!(with_fallback("bad".into()), Some(0));
```

### Recursion schemes — structured folds and unfolds

Build and tear down recursive data without writing explicit recursion:

```rust
use karpal_recursion::{Fix, cata, ana, histo};
use karpal_core::hkt::OptionF;

// Build natural number 5 by unfolding
let five: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    5,
);

// Fold it back to count layers
let count = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(n) => n + 1,
    },
    five.clone(),
);
assert_eq!(count, 5);

// Histomorphism — fold with access to full history (Fibonacci)
let fib = histo::<OptionF, u64>(
    |layer| match layer {
        None => 0,
        Some(cofree) => {
            let prev = cofree.head;
            match cofree.tail.as_ref() {
                None => 1,
                Some(gc) => prev + gc.head,
            }
        }
    },
    five,
);
assert_eq!(fib, 5);  // fib(5) = 5
```

### Abstract algebra — groups, rings, fields, and vector spaces

```rust
use karpal_algebra::{Group, Ring, Field, Semiring, Module, VectorSpace};
use karpal_core::{Semigroup, Monoid, Product};
use karpal_core::hkt::VecF;
use karpal_core::Foldable;

// Group: every element has an inverse
assert_eq!(5i32.invert(), -5);
assert_eq!(10i32.combine_inverse(3), 7);

// Ring: two operations with additive inverse
assert_eq!(3i32.add(4), 7);
assert_eq!(3i32.mul(4), 12);

// Newtype wrappers select alternative Monoid instances
let product = VecF::fold_map(vec![1, 2, 3, 4], |x| Product(x));
assert_eq!(product, Product(24));

// 2D vector space
let e1 = (1.0f64, 0.0);
let e2 = (0.0f64, 1.0);
let v = e1.scale(3.0).combine(e2.scale(4.0));
assert!((v.0 - 3.0).abs() < 1e-10);
assert!((v.1 - 4.0).abs() < 1e-10);
```

### Adjunctions — state and store from universal constructions

Adjunctions capture deep relationships between functors. The currying
adjunction `EnvF<E> ⊣ ReaderF<E>` gives rise to the State monad and
Store comonad as derived constructions:

```rust
use karpal_core::adjunction::{CurryAdj, state_pure, state_chain, state_get, state_modify};
use karpal_core::hkt::OptionF;

// State monad from the currying adjunction
let counter = state_chain(
    state_get::<i32>(),
    |n| state_chain(
        state_modify(move |s: i32| s + 1),
        move |_| state_pure::<i32, _>(n),  // return old value, increment state
    ),
);
let (result, final_state) = counter(10);
assert_eq!(result, 10);       // returned the old value
assert_eq!(final_state, 11);  // state was incremented
```

### Monad transformers — compose effects

Stack monadic effects with transformers. Each transformer adds one
effect to an existing monad:

```rust
use karpal_effect::{ExceptTF, MonadTrans, FunctorSt, ChainSt};
use karpal_core::hkt::OptionF;

// ExceptT adds error handling on top of Option
type ExceptOpt<A> = Option<Result<A, String>>;

// Lift an Option into the ExceptT stack
let lifted: ExceptOpt<i32> = ExceptTF::<String, OptionF>::lift(Some(42));
assert_eq!(lifted, Some(Ok(42)));

// Map over the inner value through both layers
let doubled = ExceptTF::<String, OptionF>::fmap_st(
    Some(Ok(21)),
    |x| x * 2,
);
assert_eq!(doubled, Some(Ok(42)));
```

### Lens composition — focus deep into nested structs

Chain lenses with `.then()` to focus multiple levels deep:

```rust
use karpal_optics::Lens;

struct Company { name: String, ceo: Person }
struct Person  { name: String, age: u32 }

let ceo = Lens::new(
    |c: &Company| c.ceo.clone(),
    |c, ceo| Company { ceo, ..c },
);
let age = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

// Compose: Company → ceo → age
let ceo_age = ceo.then(age);

let co = Company {
    name: "Acme".into(),
    ceo: Person { name: "Alice".into(), age: 30 },
};

assert_eq!(ceo_age.get(&co), 30);
let updated = ceo_age.over(co, |a| a + 1);
assert_eq!(updated.ceo.age, 31);
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
