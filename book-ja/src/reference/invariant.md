# Invariant

Invariant functors: mapping that requires both directions.

An `Invariant` functor generalizes both covariant (`Functor`) and contravariant (`Contravariant`) functors. Where a `Functor` only needs a forward function `A -> B` to transform its contents, and a `Contravariant` only needs a backward function `B -> A`, an `Invariant` functor requires *both* directions. This makes it the most general of the three -- any type that is either covariant or contravariant is automatically invariant as well.


### Invariant

A functor that maps with both a covariant and contravariant function.


#### Signature

``` rust
/// Invariant functor: maps with both a covariant and contravariant function.
///
/// Every covariant Functor is trivially Invariant (ignoring `g`).
/// Every Contravariant is also Invariant (ignoring `f`).
///
/// Laws:
/// - Identity: `invmap(fa, id, id) == fa`
/// - Composition: `invmap(fa, g1 . f1, f2 . g2) == invmap(invmap(fa, f1, f2), g1, g2)`
pub trait Invariant: HKT {
    fn invmap<A, B>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B,
        g: impl Fn(B) -> A,
    ) -> Self::Of<B>;
}
```

The `invmap` method takes a value in the functor (`fa`), a forward function `f: A -> B`, and a backward function `g: B -> A`, and produces a new value of type `Self::Of<B>`. The forward function `f` is used to transform values going out, and the backward function `g` is available for types that need to transform values going in.

#### Laws


Identity

Mapping with two identity functions changes nothing:

``` rust
F::invmap(fa, |a| a, |a| a) == fa
```

If neither direction transforms the value, the structure is unchanged.


Composition

Composing two `invmap` calls is the same as composing the functions and calling `invmap` once:

``` rust
F::invmap(fa, |a| g1(f1(a)), |a| f2(g2(a)))
    == F::invmap(F::invmap(fa, f1, f2), g1, g2)
```

The forward functions compose left-to-right (`g1 . f1`), while the backward functions compose right-to-left (`f2 . g2`). This mirrors how covariant and contravariant mappings compose in opposite directions.


#### Instances

| Type constructor | Behavior of `invmap`                                                                          | Feature gate     |
|------------------|-----------------------------------------------------------------------------------------------|------------------|
| `OptionF`        | Maps the inner value with `f` (ignores `g`); `None` stays `None`                              | none (`no_std`)  |
| `ResultF<E>`     | Maps the `Ok` value with `f` (ignores `g`); `Err` is unchanged                                | none (`no_std`)  |
| `VecF`           | Maps each element with `f` (ignores `g`)                                                      | `std` or `alloc` |
| `IdentityF`      | Applies `f` directly to the value (ignores `g`)                                               | none (`no_std`)  |
| `NonEmptyVecF`   | Maps the head and tail elements with `f` (ignores `g`)                                        | `std` or `alloc` |
| `EnvF<E>`        | Maps the second element of the tuple with `f` (ignores `g`); the environment `E` is unchanged | none (`no_std`)  |

All of the instances listed above are covariant functors, so they only use the forward function `f` and ignore the backward function `g`. A truly invariant type -- one that is neither covariant nor contravariant -- would need both functions. Such types arise in practice with bidirectional codecs, serializers/deserializers, and isomorphisms.

#### Examples

``` rust
use karpal_core::hkt::{OptionF, VecF, IdentityF, EnvF, ResultF};
use karpal_core::invariant::Invariant;

// Option: maps Some values, passes through None
let doubled = OptionF::invmap(Some(3), |x| x * 2, |x| x / 2);
assert_eq!(doubled, Some(6));

let nothing = OptionF::invmap(None::<i32>, |x| x * 2, |x| x / 2);
assert_eq!(nothing, None);

// Result: maps Ok values, leaves Err unchanged
let ok = ResultF::<&str>::invmap(Ok(5), |x| x + 1, |x| x - 1);
assert_eq!(ok, Ok(6));

// Vec: maps each element
let scaled = VecF::invmap(vec![1, 2, 3], |x| x * 2, |x| x / 2);
assert_eq!(scaled, vec![2, 4, 6]);

// Identity: applies the function directly
let result = IdentityF::invmap(42, |x| x + 1, |x| x - 1);
assert_eq!(result, 43);

// Env: maps the value, keeps the environment
let env = EnvF::<&str>::invmap(("hello", 42), |x| x + 1, |x| x - 1);
assert_eq!(env, ("hello", 43));
```

#### Relationship to Functor and Contravariant

`Invariant` sits at the top of the variance hierarchy. Every `Functor` (covariant functor) is trivially `Invariant`: just ignore the backward function `g` and use `f` alone. Likewise, every `Contravariant` functor is trivially `Invariant`: just ignore the forward function `f` and use `g` alone.

``` rust
// A Functor can implement Invariant by ignoring g:
//   fn invmap(fa, f, _g) { F::fmap(fa, f) }
//
// A Contravariant can implement Invariant by ignoring f:
//   fn invmap(fa, _f, g) { C::contramap(fa, g) }
```

This means `Invariant` captures the most general notion of "mappability" for a type constructor. It is useful when you need to abstract over types that may be covariant, contravariant, or neither -- for example, when building generic codec or serialization frameworks where values flow in both directions.

In Karpal, all provided instances happen to be covariant (they are all `Functor`s), so they ignore `g`. However, the `Invariant` trait is available for user-defined types that genuinely require both directions.


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


