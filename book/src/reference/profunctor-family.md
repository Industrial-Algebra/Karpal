# Profunctor Family

Profunctors: contravariant in the first argument, covariant in the second.

The profunctor family lives in the `karpal-profunctor` crate and provides the abstract machinery behind Karpal's [profunctor optics](optics.md). Where a `Functor` transforms values inside a single type parameter, a `Profunctor` transforms values flowing through a two-parameter type -- think of it as a pipe with an input end and an output end. You can pre-process the input (contravariantly) and post-process the output (covariantly) without opening the pipe.

## Hierarchy

The profunctor hierarchy branches into subclasses, each enabling a different family of optics:

``` rust
HKT2
  |
Profunctor          -- dimap, lmap, rmap            (Iso)
  |         \
Strong     Choice                                   (Lens / Prism)
  |           |
  +-----------+
        |
    Traversing      -- wander                       (Traversal)
```

- **Profunctor** -- the base trait. Provides `dimap` for simultaneous pre- and post-processing. Powers [Iso](optics.md#iso).
- **Strong** -- lifts a profunctor through product types (tuples). Powers [Lens](optics.md#lens).
- **Choice** -- lifts a profunctor through sum types (`Result`). Powers [Prism](optics.md#prism).
- **Traversing** -- extends Strong + Choice to handle multiple foci. Powers [Traversal](optics.md#traversal).

All three traits require the `HKT2` encoding from `karpal-core`:

``` rust
pub trait HKT2 {
    type P<A, B>;
}
```

A type implementing `HKT2` is a two-parameter type constructor. Given types `A` and `B`, it produces a concrete type `P<A, B>`.


### Profunctor

A type that is contravariant in its first argument and covariant in its second.


#### Signature

``` rust
/// A profunctor is contravariant in its first argument and covariant in its second.
pub trait Profunctor: HKT2 {
    fn dimap<A: 'static, B: 'static, C, D>(
        f: impl Fn(C) -> A + 'static,
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, D>;

    fn lmap<A: 'static, B: 'static, C>(
        f: impl Fn(C) -> A + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, B> { ... }

    fn rmap<A: 'static, B: 'static, D>(
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<A, D> { ... }
}
```

`dimap` is the fundamental operation. It takes a function `f: C -> A` that pre-processes the input (contravariant -- note the reversed direction) and a function `g: B -> D` that post-processes the output (covariant), then adapts the profunctor `P<A, B>` into `P<C, D>`.

The convenience methods `lmap` and `rmap` have default implementations in terms of `dimap`:

- `lmap(f, pab)` -- pre-process the input only. Equivalent to `dimap(f, |b| b, pab)`.
- `rmap(g, pab)` -- post-process the output only. Equivalent to `dimap(|a| a, g, pab)`.

#### Laws


Identity

Dimapping with identity functions on both sides changes nothing:

``` rust
P::dimap(|a| a, |b| b, pab) == pab
```


Composition

Dimapping with composed functions is the same as dimapping twice:

``` rust
P::dimap(|a| f(g(a)), |b| h(i(b)), pab)
    == P::dimap(g, h, P::dimap(f, i, pab))
```

Note the order reversal on the contravariant (left) side: `f` then `g` becomes `|a| f(g(a))`, because contravariance reverses composition.


#### Instances

| Marker type  | `P<A, B>` resolves to                | Feature gate    |
|--------------|--------------------------------------|-----------------|
| `FnP`        | `Box<dyn Fn(A) -> B>`                | `alloc`         |
| `ForgetF<R>` | `Box<dyn Fn(A) -> R>` (B is phantom) | `alloc`         |
| `TaggedF`    | `B` (A is phantom)                   | none (`no_std`) |

#### Examples

``` rust
use karpal_profunctor::{Profunctor, FnP};

// A simple doubling function as a profunctor value
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);

// dimap: parse a string to i32 on the input side,
//        format the i32 result to a string on the output side
let f = FnP::dimap(
    |s: &str| s.len() as i32,  // contravariant: &str -> i32
    |n: i32| n.to_string(),     // covariant: i32 -> String
    double,
);
assert_eq!(f("hello"), "10"); // len("hello") = 5, doubled = 10

// lmap: only pre-process the input
let negate: Box<dyn Fn(i32) -> i32> = Box::new(|x| -x);
let neg_len = FnP::lmap(|s: &str| s.len() as i32, negate);
assert_eq!(neg_len("hi"), -2);

// rmap: only post-process the output
let add_one: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
let as_string = FnP::rmap(|n: i32| format!("result: {}", n), add_one);
assert_eq!(as_string(9), "result: 10");
```


### Strong

A profunctor that can be lifted through product types (tuples).


#### Signature

``` rust
pub trait Strong: Profunctor {
    fn first<A, B, C>(pab: Self::P<A, B>) -> Self::P<(A, C), (B, C)>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    fn second<A, B, C>(pab: Self::P<A, B>) -> Self::P<(C, A), (C, B)>
    where
        A: 'static,
        B: 'static,
        C: 'static;
}
```

`first` takes a profunctor `P<A, B>` and lifts it to operate on the first component of a tuple, passing the second component `C` through unchanged. `second` does the mirror image -- it operates on the second component and passes the first through.

#### Laws


First-Dimap Coherence

Lifting through `first` and then dimapping with tuple projections is consistent:

``` rust
P::lmap(|(a, _)| a, P::first(pab))
    == P::rmap(|b| (b, ()), pab)  // up to isomorphism with unit
```


First-First Coherence

Nesting `first` twice is equivalent to `first` once with a tuple reassociation:

``` rust
P::first(P::first(pab))
    == P::dimap(
        |((a, c1), c2)| (a, (c1, c2)),  // reassociate in
        |(b, (c1, c2))| ((b, c1), c2),  // reassociate out
        P::first(pab),
    )
```


#### Instances

| Marker type  | Behavior of `first`                                       | Feature gate |
|--------------|-----------------------------------------------------------|--------------|
| `FnP`        | `|(a, c)| (pab(a), c)`                                    | `alloc`      |
| `ForgetF<R>` | `|(a, _)| pab(a)` -- extracts R, ignores second component | `alloc`      |

`TaggedF` is deliberately **not** `Strong`. This enforces at the type level that write-only optics (like [Review](optics.md#review)) cannot be used for reading -- `Strong` requires producing a `(B, C)` from a `B`, but `TaggedF` has no way to produce the `C`.

#### Connection to Lens

A [Lens](optics.md) is defined as a function that works for *all* profunctors that are `Strong`. The lens `transform` method takes a `P<A, B>` and returns a `P<S, T>` by using `Strong::first` (or `second`) together with `Profunctor::dimap` to focus on a part of a structure:

``` rust
// Conceptually, a lens from S to A (with update types T and B) is:
//   for all P: Strong, P<A, B> -> P<S, T>
//
// Implemented as:
//   dimap(getter_and_context, setter, first(pab))
//
// where:
//   getter_and_context: S -> (A, Context)
//   setter: (B, Context) -> T
```

`Strong::first` lifts the profunctor to work on a tuple `(A, Context)`, and `dimap` adapts the outer structure `S`/`T` to and from that tuple. This is how profunctor optics achieve composability -- lens composition is just function composition of these transforms.

#### Examples

``` rust
use karpal_profunctor::{Strong, FnP};

let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);

// first: apply to the first element of a tuple
let f = FnP::first::<i32, i32, &str>(double);
assert_eq!(f((5, "hi")), (10, "hi"));

// second: apply to the second element of a tuple
let triple: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 3);
let g = FnP::second::<i32, i32, &str>(triple);
assert_eq!(g(("hi", 5)), ("hi", 15));
```


### Choice

A profunctor that can be lifted through sum types (`Result`).


#### Signature

``` rust
pub trait Choice: Profunctor {
    fn left<A, B, C>(pab: Self::P<A, B>) -> Self::P<Result<A, C>, Result<B, C>>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    fn right<A, B, C>(pab: Self::P<A, B>) -> Self::P<Result<C, A>, Result<C, B>>
    where
        A: 'static,
        B: 'static,
        C: 'static;
}
```

Karpal uses `Result<L, R>` as the sum type rather than a custom `Either` -- this is idiomatic Rust and avoids an unnecessary new type. `left` lifts a profunctor to operate on the `Ok` branch of a `Result`, passing the `Err` branch through unchanged. `right` does the mirror image, operating on the `Err` branch.

#### Laws


Left-Dimap Coherence

Lifting through `left` and then extracting the `Ok` branch is consistent:

``` rust
P::lmap(|a| Ok(a), P::left(pab))
    == P::rmap(|b| Ok(b), pab)
```


Left-Left Coherence

Nesting `left` twice is equivalent to `left` once with a `Result` reassociation:

``` rust
P::left(P::left(pab))
    == P::dimap(
        |r| match r {                       // reassociate in
            Ok(Ok(a))  => Ok(a),
            Ok(Err(c)) => Err(Ok(c)),
            Err(d)     => Err(Err(d)),
        },
        |r| match r {                       // reassociate out
            Ok(b)      => Ok(Ok(b)),
            Err(Ok(c)) => Ok(Err(c)),
            Err(Err(d)) => Err(d),
        },
        P::left(pab),
    )
```


#### Instances

| Marker type          | Behavior of `left`                                      | Feature gate    |
|----------------------|---------------------------------------------------------|-----------------|
| `FnP`                | `|r| match r { Ok(a) => Ok(pab(a)), Err(c) => Err(c) }` | `alloc`         |
| `ForgetF<R: Monoid>` | `|r| match r { Ok(a) => pab(a), Err(_) => R::empty() }` | `alloc`         |
| `TaggedF`            | `Ok(pab)` -- wraps the value in `Ok`                    | none (`no_std`) |

`ForgetF`'s `Choice` impl requires `R: Monoid` because the miss case needs a default value (`R::empty()`). Its `Strong` impl has no such restriction -- products always have the focus available.

#### Connection to Prism

A [Prism](optics.md) is defined as a function that works for *all* profunctors that are `Choice`. The prism `transform` method takes a `P<A, B>` and returns a `P<S, T>` by using `Choice::right` together with `Profunctor::dimap` to focus on one variant of a sum type:

``` rust
// Conceptually, a prism from S to A (with update types T and B) is:
//   for all P: Choice, P<A, B> -> P<S, T>
//
// Implemented as:
//   dimap(match_, merge, right(pab))
//
// where:
//   match_: S -> Result<T, A>   (try to extract A, or return T unchanged)
//   merge:  Result<T, B> -> T   (re-inject the modified value)
```

When the `match_` function successfully extracts an `A`, the profunctor processes it into a `B`; otherwise the original `T` passes through untouched. `Choice::right` ensures that only the matched branch is transformed. This is the dual of how `Strong` powers lenses: lenses focus through products, prisms focus through sums.

#### Examples

``` rust
use karpal_profunctor::{Choice, FnP};

let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);

// left: apply to the Ok branch
let f = FnP::left::<i32, i32, &str>(double);
assert_eq!(f(Ok(5)), Ok(10));
assert_eq!(f(Err("nope")), Err("nope"));

// right: apply to the Err branch
let triple: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 3);
let g = FnP::right::<i32, i32, &str>(triple);
assert_eq!(g(Err(5)), Err(15));
assert_eq!(g(Ok("yep")), Ok("yep"));
```


### FnP (Function Profunctor)

Marker type whose `P<A, B>` is `Box<dyn Fn(A) -> B>` -- the canonical profunctor instance.


#### Definition

``` rust
pub struct FnP;

impl HKT2 for FnP {
    type P<A, B> = Box<dyn Fn(A) -> B>;
}
```

`FnP` is a zero-sized marker type. It has no fields and no runtime cost -- it exists solely to carry the `HKT2` type-level association between the marker and the concrete type `Box<dyn Fn(A) -> B>`.

This is the function arrow profunctor (sometimes written `(->)` in Haskell). It is the most natural profunctor: a function from `A` to `B` can be pre-composed with a function `C -> A` (contravariant input) and post-composed with a function `B -> D` (covariant output) to yield a function `C -> D`.

#### Feature gate

`FnP` requires the `alloc` feature because `Box<dyn Fn>` requires heap allocation. It is not available in `no_std` environments without an allocator. The `Profunctor`, `Strong`, and `Choice` traits themselves are `no_std`-compatible -- only the `FnP` instance needs `alloc`.

#### Implemented traits

| Trait        | Method                             | Implementation                                                         |
|--------------|------------------------------------|------------------------------------------------------------------------|
| `Profunctor` | `dimap(f, g, pab)`                 | `Box::new(move |c| g(pab(f(c))))`                                      |
| `Strong`     | `first(pab)`                       | `Box::new(move |(a, c)| (pab(a), c))`                                  |
| `Strong`     | `second(pab)`                      | `Box::new(move |(c, a)| (c, pab(a)))`                                  |
| `Choice`     | `left(pab)`                        | `Box::new(move |r| match r { Ok(a) => Ok(pab(a)), Err(c) => Err(c) })` |
| `Choice`     | `right(pab)`                       | `Box::new(move |r| match r { Ok(c) => Ok(c), Err(a) => Err(pab(a)) })` |
| `Traversing` | `wander(get_all, modify_all, pab)` | `Box::new(move |s| modify_all(s, &*pab))`                              |

#### Role in optics

`FnP` is the profunctor that optics use at the value level. When you call `lens.set()` or `lens.over()`, Karpal internally constructs a `Box<dyn Fn(A) -> B>` and passes it through the lens's `transform` method, which threads it through `Strong::first` and `Profunctor::dimap`. The result is a `Box<dyn Fn(S) -> T>` that performs the focused update on the whole structure. The same mechanism applies to prisms via `Choice`.

Because all the profunctor operations compose (they are just function wrapping), lens composition and prism composition are both achieved by chaining `transform` calls -- no special composition machinery is needed.

#### Example: building a pipeline with dimap

``` rust
use karpal_profunctor::{Profunctor, FnP};

// Start with a base function: parse a number and add 1
let inc: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);

// Adapt it: input is a string (parse it), output is a string (format it)
let pipeline = FnP::dimap(
    |s: &str| s.parse::<i32>().unwrap_or(0),
    |n: i32| format!("result = {}", n),
    inc,
);

assert_eq!(pipeline("41"), "result = 42");
assert_eq!(pipeline("not a number"), "result = 1");
```


### Traversing

A profunctor that can operate over multiple foci simultaneously.


#### Signature

``` rust
pub trait Traversing: Strong + Choice {
    fn wander<S, T, A, B>(
        get_all: impl Fn(&S) -> Vec<A> + 'static,
        modify_all: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<S, T>
    where
        S: 'static, T: 'static, A: 'static, B: 'static;
}
```

`Traversing` extends `Strong + Choice` with the ability to handle multiple foci. The `wander` method takes **two** functions instead of a single polymorphic traversal function, because Rust lacks rank-2 types:

- `get_all` -- extracts all foci (used by read-only profunctors like `ForgetF`)
- `modify_all` -- applies a function to every focus in-place (used by read-write profunctors like `FnP`)

#### Instances

| Marker type          | Strategy                                                        | Feature gate |
|----------------------|-----------------------------------------------------------------|--------------|
| `FnP`                | Uses `modify_all`, ignores `get_all`                            | `alloc`      |
| `ForgetF<R: Monoid>` | Uses `get_all`, maps each through `pab`, combines with `Monoid` | `alloc`      |

#### Connection to Traversal

A [Traversal](optics.md#traversal) is defined as a function that works for *all* profunctors that are `Traversing`. The traversal's `transform` method calls `P::wander(get_all, modify_all, pab)` -- each profunctor instance decides how to interpret the traversal.

#### Example

``` rust
use karpal_profunctor::{Traversing, FnP, ForgetF};

// wander with FnP: modify each element
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
let f = FnP::wander(
    |v: &Vec<i32>| v.clone(),
    |v: Vec<i32>, f: &dyn Fn(i32) -> i32| v.into_iter().map(f).collect(),
    double,
);
assert_eq!(f(vec![1, 2, 3]), vec![2, 4, 6]);

// wander with ForgetF: accumulate with Monoid
let to_str: Box<dyn Fn(i32) -> String> = Box::new(|x| x.to_string());
let g = <ForgetF<String> as Traversing>::wander(
    |v: &Vec<i32>| v.clone(),
    |v: Vec<i32>, f: &dyn Fn(i32) -> String| { let _ = v; let _ = f; String::new() },
    to_str,
);
assert_eq!(g(vec![1, 2, 3]), "123"); // String Monoid concatenates
```


### ForgetF\<R\> (Forget Profunctor)

Marker type whose `P<A, B>` is `Box<dyn Fn(A) -> R>` -- a read-only profunctor that extracts a summary value.


#### Definition

``` rust
pub struct ForgetF<R>(PhantomData<R>);

impl<R: 'static> HKT2 for ForgetF<R> {
    type P<A, B> = Box<dyn Fn(A) -> R>;
}
```

`ForgetF<R>` "forgets" the output type `B` entirely -- the second type parameter is phantom. A `ForgetF<R>::P<A, B>` is just a function from `A` to `R`, regardless of what `B` is. This makes it ideal for read-only operations that extract or summarize data.

#### Implemented traits

| Trait        | Constraint on `R` | Behavior                                                                 |
|--------------|-------------------|--------------------------------------------------------------------------|
| `Profunctor` | `'static`         | `dimap(f, _g, pab) = |c| pab(f(c))` -- `g` is ignored                    |
| `Strong`     | `'static`         | `first(pab) = |(a, _)| pab(a)` -- extracts from first component          |
| `Choice`     | `Monoid`          | `left(pab) = |r| match r { Ok(a) => pab(a), Err(_) => R::empty() }`      |
| `Traversing` | `Monoid`          | Maps each focus through `pab`, combines results via `Semigroup::combine` |

#### Feature gate

Requires `alloc` (uses `Box<dyn Fn>` and `Vec`).

#### Role in optics

`ForgetF` is the profunctor behind read-only optics. When you use a [Traversal](optics.md#traversal) with `ForgetF<R>`, you get a function `S -> R` that extracts and combines data from all foci using a `Monoid`. This is how [Fold](optics.md#fold)'s `fold_map` works conceptually.

``` rust
use karpal_profunctor::ForgetF;

// ForgetF ignores the B parameter completely
let extract: Box<dyn Fn(i32) -> String> = Box::new(|x| format!("got {x}"));

// B can be anything -- it's never used
let _: <ForgetF<String> as HKT2>::P<i32, Vec<u8>> = extract;
```


### TaggedF (Tagged Profunctor)

Marker type whose `P<A, B>` is just `B` -- a write-only profunctor for construction.


#### Definition

``` rust
pub struct TaggedF;

impl HKT2 for TaggedF {
    type P<A, B> = B;
}
```

`TaggedF` "forgets" the input type `A` entirely -- the first type parameter is phantom. A `TaggedF::P<A, B>` is just `B`, regardless of what `A` is. This makes it ideal for construction-only operations.

#### Implemented traits

| Trait        | Behavior                                   |
|--------------|--------------------------------------------|
| `Profunctor` | `dimap(_f, g, b) = g(b)` -- `f` is ignored |
| `Choice`     | `left(b) = Ok(b)`, `right(b) = Err(b)`     |

`TaggedF` is deliberately **not** `Strong` and **not** `Traversing`. This is a deliberate design decision: `Strong::first` would need to produce a `(B, C)` from just a `B`, which is impossible without access to `C`. By not implementing `Strong`, the type system enforces that write-only optics like [Review](optics.md#review) cannot be used for reading.

#### Feature gate

None -- `TaggedF` is `no_std`-compatible with no allocator requirement.

#### Role in optics

`TaggedF` is the profunctor behind write-only optics. A [Review](optics.md#review) conceptually transforms a `TaggedF::P<A, B>` (which is just `B`) into a `TaggedF::P<S, T>` (which is just `T`) -- construction from a value.


## Why `'static` bounds?

You will notice that the type parameters on `dimap`, `first`, `left`, and so on require `'static` bounds. This is a consequence of `FnP`'s implementation: `Box<dyn Fn(A) -> B>` requires that the captured closures (and the types they close over) are `'static`. Without this bound, the compiler cannot guarantee that the boxed closures outlive the scope in which they were created.

In practice, this means profunctor operations work with owned types and `'static` references, but not with short-lived borrows. This is the same trade-off that `Box<dyn Fn>` imposes anywhere in Rust -- the profunctor abstraction does not add any additional restrictions beyond what the underlying representation requires.


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


