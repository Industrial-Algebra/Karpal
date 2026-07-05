# Getting Started

This guide walks you through adding Karpal to your Rust project, understanding its HKT encoding, and using the core abstractions: Functor, Monad, and the ergonomic `do_!` and `ado_!` macros.

## 1. Installation

The easiest way to use Karpal is through the `karpal-std` crate, which re-exports everything from the other workspace crates in a single prelude.

Add it to your `Cargo.toml`:

``` rust
[dependencies]
karpal-std = "0.1"
```

Then import the prelude at the top of any module that uses Karpal types and traits:

``` rust
use karpal_std::prelude::*;
```

This single import brings in all type constructors (`OptionF`, `VecF`, `ResultF`, etc.), all traits (`Functor`, `Applicative`, `Monad`, `Foldable`, etc.), and the `do_!` and `ado_!` macros.

### Toolchain requirements

Karpal requires **nightly Rust** because it uses edition 2024 features. The repository includes a `rust-toolchain.toml` that pins the exact nightly version, so if you are working within the Karpal workspace, Cargo and rustup will select the correct toolchain automatically.

If you are consuming Karpal as a dependency in your own project, make sure your project also uses a nightly toolchain. You can create a `rust-toolchain.toml` in your project root:

``` rust
[toolchain]
channel = "nightly"
```

## 2. Your First HKT

Higher-Kinded Types (HKTs) let you abstract over *type constructors* — not just concrete types like `Option<i32>`, but the `Option` constructor itself. Rust does not natively support HKTs, but Karpal encodes them using Generic Associated Types (GATs), which have been stable since Rust 1.65.

The core trait is:

``` rust
trait HKT {
    type Of<T>;
}
```

A type that implements `HKT` is a **type constructor** — a marker type that, given a parameter `T`, produces a concrete type. Karpal provides several built-in constructors:

| Marker type  | `Of<T>` resolves to |
|--------------|---------------------|
| `OptionF`    | `Option<T>`         |
| `VecF`       | `Vec<T>`            |
| `ResultF<E>` | `Result<T, E>`      |

So `<OptionF as HKT>::Of<i32>` is simply `Option<i32>`. Nothing new at the value level — the magic is at the *type* level. You can now write functions that are generic over the *shape* of the container, not just its contents:

``` rust
use karpal_std::prelude::*;

/// Wraps a value in any container that supports `Applicative::pure`.
fn wrap<F: Applicative>(value: i32) -> F::Of<i32> {
    F::pure(value)
}

let opt: Option<i32> = wrap::<OptionF>(42);   // Some(42)
let vec: Vec<i32>    = wrap::<VecF>(42);      // vec![42]
```

The caller chooses the container by supplying a type constructor as a generic parameter. The function body stays the same regardless of which container is selected.

## 3. Your First Functor

A **Functor** is any type constructor that supports mapping a function over its contents. If you have used `Option::map` or `Iterator::map`, you already know the idea — Karpal just gives it a uniform interface.

``` rust
use karpal_std::prelude::*;

let result = OptionF::fmap(Some(2), |x| x * 3);
assert_eq!(result, Some(6));

let result = VecF::fmap(vec![1, 2, 3], |x| x + 10);
assert_eq!(result, vec![11, 12, 13]);
```

This looks similar to calling `.map()` directly, and at the concrete level it behaves identically. The difference is that `Functor::fmap` is a trait method on the *type constructor*, which means you can write functions that work with any functor:

``` rust
use karpal_std::prelude::*;

fn double_inner<F: Functor>(fa: F::Of<i32>) -> F::Of<i32> {
    F::fmap(fa, |x| x * 2)
}

// Works with Option
assert_eq!(double_inner::<OptionF>(Some(5)), Some(10));
assert_eq!(double_inner::<OptionF>(None), None);

// Works with Vec
assert_eq!(double_inner::<VecF>(vec![1, 2, 3]), vec![2, 4, 6]);
```

One function, multiple container types, zero code duplication.

### Functor laws

Every `Functor` implementation must satisfy two laws. Karpal verifies these with property-based tests, but they are worth knowing informally:

- **Identity:** mapping the identity function changes nothing. `F::fmap(fa, |x| x) == fa`
- **Composition:** mapping `f` then `g` is the same as mapping `|x| g(f(x))`. `F::fmap(F::fmap(fa, f), g) == F::fmap(fa, |x| g(f(x)))`

These laws guarantee that `fmap` only transforms values — it never adds, removes, or reorders elements in the container.

## 4. Monadic Notation with `do_!`

Monadic computations in Rust quickly turn into deeply nested `.and_then()` chains. Each step that depends on the previous value adds another level of indentation:

``` rust
// The nesting problem: every step pushes the code further right
fn fetch_dashboard(user_id: &str) -> Option<Dashboard> {
    lookup_user(user_id).and_then(|user| {
        load_preferences(&user).and_then(|prefs| {
            fetch_activity(&user).and_then(|activity| {
                build_dashboard(&user, &prefs, &activity)
            })
        })
    })
}
```

With three steps this is manageable; with six or seven it becomes painful to read. The `do_!` macro flattens this into a top-to-bottom sequence of bindings:

``` rust
use karpal_std::prelude::*;

fn fetch_dashboard(user_id: &str) -> Option<Dashboard> {
    do_! { OptionF;
        user     = lookup_user(user_id);
        prefs    = load_preferences(&user);
        activity = fetch_activity(&user);
        build_dashboard(&user, &prefs, &activity)
    }
}
```

Each `name = expr` line binds the unwrapped value from the monadic expression on the right. If any step returns `None` (or `Err` for `ResultF`), the entire block short-circuits immediately. The final expression (without a binding) is the return value of the block.

### Syntax reference

``` rust
do_! { TypeConstructor;
    binding1 = monadic_expr1;
    binding2 = monadic_expr2;
    // ... more bindings ...
    final_monadic_expr
}
```

- The first token is the type constructor (`OptionF`, `VecF`, `ResultF<E>`, etc.), followed by a semicolon.
- Each binding uses `=`, not `<-`. Rust edition 2024 reserves `<-` as a token, so the arrow syntax is not available.
- The final line must be an expression of type `F::Of<T>` — it is the value returned by the whole `do_!` block.
- Bindings can reference earlier bindings — each step has access to all names bound above it.

### A concrete example

``` rust
use karpal_std::prelude::*;

fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

let result = do_! { OptionF;
    x = safe_divide(100.0, 4.0);   // Some(25.0)
    y = safe_divide(x, 5.0);       // Some(5.0)
    z = safe_divide(y, 2.0);       // Some(2.5)
    Some(z + 1.0)                   // Some(3.5)
};

assert_eq!(result, Some(3.5));
```

## 5. Applicative Notation with `ado_!`

When your computations are *independent* — none of them need the result of a previous step — you do not need the full power of `do_!`. The `ado_!` macro expresses this pattern and makes the independence explicit:

``` rust
use karpal_std::prelude::*;

fn load_host() -> Option<&'static str> { Some("localhost") }
fn load_port() -> Option<u16>           { Some(8080) }
fn load_workers() -> Option<usize>      { Some(4) }

let config = ado_! { OptionF;
    host    = load_host();
    port    = load_port();
    workers = load_workers();
    yield format!("{}:{} ({} workers)", host, port, workers)
};

assert_eq!(config, Some("localhost:8080 (4 workers)".to_string()));
```

The `yield` line combines all the bound values into a final result. Unlike `do_!`, the bindings in `ado_!` cannot reference each other — they are all evaluated independently, and the results are combined at the end.

### Syntax reference

``` rust
ado_! { TypeConstructor;
    binding1 = applicative_expr1;
    binding2 = applicative_expr2;
    // ... more bindings ...
    yield combining_expression
}
```

- Same first-token convention as `do_!`: the type constructor, then a semicolon.
- Each binding uses `=`. Bindings are independent and must not reference each other.
- The `yield` line combines all bound values into the final result. The expression after `yield` is a *pure* function of the bound names — it is automatically lifted into the applicative context.
- If any binding evaluates to `None` (or `Err`), the whole block short-circuits.

### When to use `ado_!` vs `do_!`

| Use this | When                                               |
|----------|----------------------------------------------------|
| `do_!`   | Later steps depend on earlier results (sequential) |
| `ado_!`  | All steps are independent (parallel-safe)          |

In practice, `ado_!` documents intent: it tells the reader that the computations have no data dependencies. For types where order does not matter (like `Option`), the runtime behavior is identical, but the semantic clarity is valuable.

## 6. Proof and External Verification

Once you are comfortable with Karpal's core abstractions, the next layer is reasoning about laws explicitly.

`karpal-proof` gives you Rust-native evidence types like `Proven<P, T>`, refinement wrappers like `NonEmpty<T>` and `Positive<T>`, and derive helpers that generate algebraic law tests.

`karpal-verify` takes the next step outward: it lets you model proof obligations, export them to SMT-LIB2 or Lean 4, write artifacts, execute verification runs, and collect JSON / Markdown reports suitable for CI. Imported certificates remain explicit and do not silently become Rust proof witnesses.

``` rust
use karpal_std::prelude::*;

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);
let report = verify_bundle(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");
assert_eq!(report.obligation_count(), 3);
```

See the [Proof & Verification reference](reference/proof-verification.md) for the full workflow and trust model, the [Verification CI Workflow](reference/verification-ci.md) guide for artifact/report orchestration, and the [Verification Schemas](reference/verification-schemas.md) page for serialized compatibility details.

## 7. Next Steps

Now that you can install Karpal, map over containers generically, flatten monadic chains, and understand where proofs fit into the ecosystem, here is where to go next:

- [**Architecture**](architecture.md) — understand the full functor hierarchy, from `Functor` through `Monad`, and the Alt/Alternative branch. See how the traits relate and which type constructors implement each one.
- [**Functor Family reference**](reference/functor-family.md) — detailed documentation for `Functor`, `Apply`, `Applicative`, `Chain`, and `Monad`, including all method signatures and implementation notes.
- [**Macros reference**](reference/macros.md) — the full syntax and edge cases for `do_!` and `ado_!`, including usage with `ResultF` and `VecF`.
- [**Optics**](reference/optics.md) — profunctor-based Lens and Prism for composable, first-class field access and pattern matching.
- [**Proof & Verification reference**](reference/proof-verification.md) — law witnesses, derive-based checks, Lean/SMT obligation export, project-aware Lean execution, diagnostics mapping, trust boundaries, and CI-oriented verification reports.
- [**Verification CI Workflow**](reference/verification-ci.md) — artifact layout, report writing, backend policies, Lean manifest/sidecar generation, and CI integration guidance for `karpal-verify`.
- [**Verification Schemas**](reference/verification-schemas.md) — schema-versioned report, manifest, and diagnostics formats plus compatibility guidance for consumers.
- [**Config Pipeline example**](examples/config-pipeline.md) — a realistic end-to-end example combining Functor, Applicative, and monadic chaining to build a configuration loader.
- [**Data Transformation example**](examples/data-transformation.md) — using Foldable, Traversable, and FunctorFilter to process collections generically.
- [**Verification Workflow example**](examples/verification-workflow.md) — a full `karpal-verify` walkthrough from obligation bundle to CI report files and explicit certificate import.
- [**Verified Domain API example**](examples/verified-domain-api.md) — how `karpal-proof` `Proven<...>`-based APIs and `karpal-verify` `Certified<...>` imports fit together at a domain boundary.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


