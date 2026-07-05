# Foldable & Traversable

Summarize and sequence container contents.

## Foldable


### Foldable

A structure that can be folded to a summary value.


#### Signature

``` rust
pub trait Foldable: HKT {
    fn fold_right<A, B>(fa: Self::Of<A>, init: B, f: impl Fn(A, B) -> B) -> B;

    fn fold_map<A, M: Monoid>(fa: Self::Of<A>, f: impl Fn(A) -> M) -> M {
        Self::fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
    }
}
```

`fold_right` is the required method. It processes elements right-to-left, threading an accumulator through each step. `fold_map` is provided as a default: it maps each element into a `Monoid` and combines them.

#### Laws


fold_map consistency

`fold_map(fa, f) == fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))`


Any override of the default `fold_map` must agree with the right fold formulation above.

#### Instances

| Type constructor | Notes                                                                 |
|------------------|-----------------------------------------------------------------------|
| `OptionF`        | Folds over the contained value, if any; returns `init` for `None`.    |
| `ResultF<E>`     | Folds over the `Ok` value; returns `init` for `Err`.                  |
| `VecF`           | Right-folds by reversing and iterating. Requires `alloc`.             |
| `IdentityF`      | Trivially applies `f` to the single contained value.                  |
| `NonEmptyVecF`   | Right-folds the tail, then applies `f` to the head. Requires `alloc`. |

#### Example: summing with fold_map

``` rust
use karpal_std::prelude::*;

// fold_map maps each element into a Monoid and combines them.
// For i32, the Monoid instance uses addition with identity 0.

let sum = VecF::fold_map(vec![1, 2, 3], |a: i32| a);
assert_eq!(sum, 6); // 1 + 2 + 3

let sum = OptionF::fold_map(Some(42), |a: i32| a);
assert_eq!(sum, 42);

let sum = OptionF::fold_map(None::<i32>, |a: i32| a);
assert_eq!(sum, 0); // Monoid::empty() for i32
```

#### Example: fold_right

``` rust
use karpal_std::prelude::*;

// fold_right processes elements right-to-left.
// With subtraction, the associativity matters:
// fold_right([1, 2, 3], 0, |a, b| a - b)
//   = 1 - (2 - (3 - 0))
//   = 1 - (2 - 3)
//   = 1 - (-1)
//   = 2
let result = VecF::fold_right(vec![1, 2, 3], 0, |a, b| a - b);
assert_eq!(result, 2);
```


## Traversable


### Traversable

A Functor + Foldable that can be traversed with an effectful function.


#### Signature

``` rust
pub trait Traversable: Functor + Foldable {
    fn traverse<G, A, B, F>(fa: Self::Of<A>, f: F) -> G::Of<Self::Of<B>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone;
}
```

`traverse` applies an effectful function `f` to every element in the structure, collecting the results inside the effect `G`. If any application of `f` produces a "failure" (e.g. `None` for `OptionF`), the entire traversal short-circuits.

#### Laws


Identity

`traverse::<IdentityF, _, _, _>(fa, pure) == pure(fa)`


Composition

`traverse::<Compose<F, G>, _, _, _>(fa, |a| Compose(F::fmap(f(a), g)))`  
`== Compose(F::fmap(traverse::<F, _, _, _>(fa, f), |fb| traverse::<G, _, _, _>(fb, g)))`


Naturality

`t(traverse::<F, _, _, _>(fa, f)) == traverse::<G, _, _, _>(fa, |a| t(f(a)))`  
for any applicative natural transformation `t: F ~> G`


Karpal verifies the Identity law with property-based tests using `OptionF` as the effect.

#### Instances

| Type constructor | Notes                                                                                                      |
|------------------|------------------------------------------------------------------------------------------------------------|
| `OptionF`        | Traverses the inner value if `Some`; returns `G::pure(None)` for `None`.                                   |
| `ResultF<E>`     | Traverses the `Ok` value; returns `G::pure(Err(e))` for `Err`. Requires `E: Clone`.                        |
| `VecF`           | Traverses each element left-to-right, accumulating via `Applicative::ap`. Requires `alloc` and `B: Clone`. |

#### Example: traverse with Option

``` rust
use karpal_std::prelude::*;

// Parse a list of strings into integers, failing if any parse fails.
fn parse(s: &str) -> Option<i32> {
    s.parse().ok()
}

// All elements parse successfully:
let result = VecF::traverse::<OptionF, _, _, _>(
    vec!["1", "2", "3"],
    parse,
);
assert_eq!(result, Some(vec![1, 2, 3]));

// One element fails, so the whole traversal returns None:
let result = VecF::traverse::<OptionF, _, _, _>(
    vec!["1", "oops", "3"],
    parse,
);
assert_eq!(result, None);
```

#### Example: traverse over Option

``` rust
use karpal_std::prelude::*;

// Traverse an Option with an effectful function:
let result = OptionF::traverse::<OptionF, _, _, _>(
    Some(3),
    |x| Some(x * 2),
);
assert_eq!(result, Some(Some(6)));

// If the inner effect fails:
let result = OptionF::traverse::<OptionF, _, _, _>(
    Some(3),
    |_x| None::<i32>,
);
assert_eq!(result, None);

// Traversing None always succeeds:
let result = OptionF::traverse::<OptionF, i32, i32, _>(
    None,
    |x| Some(x * 2),
);
assert_eq!(result, Some(None));
```


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


