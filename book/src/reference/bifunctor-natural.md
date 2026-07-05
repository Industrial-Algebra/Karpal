# Bifunctor & NaturalTransformation

Two-parameter functors and structure-preserving transformations.

These two abstractions sit alongside the main Functor hierarchy but address different concerns. `Bifunctor` generalizes mapping over type constructors with *two* type parameters (using `HKT2`), while `NaturalTransformation` provides a way to convert between two single-parameter type constructors without inspecting the contained values.


### Bifunctor

Maps over both type parameters of a two-parameter type constructor.


#### Signature

``` rust
/// Bifunctor: maps over both type parameters of a two-parameter type constructor.
///
/// Laws:
/// - Identity: `bimap(id, id, fab) == fab`
/// - Composition: `bimap(f . g, h . i, fab) == bimap(f, h, bimap(g, i, fab))`
pub trait Bifunctor: HKT2 {
    fn bimap<A, B, C, D>(
        fab: Self::P<A, B>,
        f: impl Fn(A) -> C,
        g: impl Fn(B) -> D,
    ) -> Self::P<C, D>;

    fn first<A, B, C>(fab: Self::P<A, B>, f: impl Fn(A) -> C) -> Self::P<C, B> {
        Self::bimap(fab, f, |b| b)
    }

    fn second<A, B, D>(fab: Self::P<A, B>, g: impl Fn(B) -> D) -> Self::P<A, D> {
        Self::bimap(fab, |a| a, g)
    }
}
```

The `bimap` method applies two functions simultaneously -- one to each type parameter. The `first` and `second` methods are convenience shortcuts that map over only one parameter, leaving the other unchanged. Both have default implementations in terms of `bimap`.

Note that `Bifunctor` extends `HKT2`, the two-parameter higher-kinded type trait. Where `HKT` has `type Of<T>`, `HKT2` has `type P<A, B>`, reflecting the two type parameters.

#### Laws


Identity

Mapping two identity functions over a value must return it unchanged:

``` rust
F::bimap(fab, |a| a, |b| b) == fab
```


Composition

Mapping composed functions must equal mapping in two steps:

``` rust
F::bimap(fab, |a| f(g(a)), |b| h(i(b)))
    == F::bimap(F::bimap(fab, g, i), f, h)
```


#### Instances

| Marker type | `P<A, B>` resolves to | Behavior                                                  | Feature gate    |
|-------------|-----------------------|-----------------------------------------------------------|-----------------|
| `ResultBF`  | `Result<B, A>`        | `f` maps over the `Err` side, `g` maps over the `Ok` side | none (`no_std`) |
| `TupleF`    | `(A, B)`              | `f` maps over the first element, `g` maps over the second | none (`no_std`) |

Note that `ResultBF` places the first type parameter in the `Err` position and the second in the `Ok` position (`P<A, B> = Result<B, A>`). This is consistent with the Bifunctor convention where the second parameter is the "main" one, matching how `ResultF<E>` treats the `Ok` value as the functor target.

#### Examples

``` rust
use karpal_core::bifunctor::Bifunctor;
use karpal_core::hkt::{ResultBF, TupleF};

// bimap over a Result: transform both Ok and Err sides
let r: Result<i32, &str> = Ok(5);
let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
assert_eq!(result, Ok(10));

let r: Result<i32, &str> = Err("hello");
let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
assert_eq!(result, Err(5));

// bimap over a tuple: transform both elements
assert_eq!(TupleF::bimap((1, "hi"), |x| x + 1, |s| s.len()), (2, 2));

// first and second: map over one side only
assert_eq!(TupleF::first((1, "hi"), |x| x * 2), (2, "hi"));
assert_eq!(TupleF::second((1, "hi"), |s| s.len()), (1, 2));

// first on Result maps the Err side
let r: Result<i32, &str> = Err("hi");
assert_eq!(ResultBF::first(r, |s| s.len()), Err(2));

// second on Result maps the Ok side
let r: Result<i32, &str> = Ok(5);
assert_eq!(ResultBF::second(r, |n| n * 3), Ok(15));
```


### NaturalTransformation

A structure-preserving mapping between two type constructors.


#### Signature

``` rust
/// Natural transformation: a mapping between two functors that preserves structure.
///
/// Laws:
/// - Naturality: `fmap_G(f, transform(fa)) == transform(fmap_F(f, fa))`
pub trait NaturalTransformation<F: HKT, G: HKT> {
    fn transform<A>(fa: F::Of<A>) -> G::Of<A>;
}
```

A `NaturalTransformation` converts a value from one type constructor into another without knowing or caring about the contained type `A`. The trait is parameterized by two `HKT` type constructors, `F` (source) and `G` (target), and the implementing struct serves as a named witness for the transformation.

Because the `transform` method is generic over `A`, it cannot inspect or modify the contained values -- it can only restructure the container. This is the key property that the naturality law captures.

#### Laws


Naturality

Mapping a function `f` over the result of `transform` must equal transforming after mapping `f` over the original:

``` rust
G::fmap(NT::transform(fa), f) == NT::transform(F::fmap(fa, f))
```

In other words, it does not matter whether you map first and then transform, or transform first and then map. The diagram commutes.


#### Instances

| Struct            | Source (`F`) | Target (`G`) | Behavior                                             | Feature gate     |
|-------------------|--------------|--------------|------------------------------------------------------|------------------|
| `OptionToVec`     | `OptionF`    | `VecF`       | `None` becomes `vec![]`, `Some(a)` becomes `vec![a]` | `std` or `alloc` |
| `VecHeadToOption` | `VecF`       | `OptionF`    | Takes the first element; empty `Vec` becomes `None`  | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::natural::{NaturalTransformation, OptionToVec, VecHeadToOption};

// OptionToVec: convert Option into a zero-or-one-element Vec
assert_eq!(OptionToVec::transform(Some(42)), vec![42]);
assert_eq!(OptionToVec::transform(None::<i32>), Vec::<i32>::new());

// VecHeadToOption: extract the first element as an Option
assert_eq!(VecHeadToOption::transform(vec![1, 2, 3]), Some(1));
assert_eq!(VecHeadToOption::transform(Vec::<i32>::new()), None);
```

#### Verifying the naturality law

The naturality law can be checked for any function `f`. Here is a concrete example with `OptionToVec`:

``` rust
use karpal_core::functor::Functor;
use karpal_core::hkt::{OptionF, VecF};
use karpal_core::natural::{NaturalTransformation, OptionToVec};

let x: Option<i32> = Some(5);
let f = |a: i32| a + 1;

// Map then transform
let left = OptionToVec::transform(OptionF::fmap(x, f));

// Transform then map
let right = VecF::fmap(OptionToVec::transform(x), f);

assert_eq!(left, right); // both are vec![6]
```


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


