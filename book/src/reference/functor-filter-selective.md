# FunctorFilter & Selective

Filtering and conditional execution within functorial contexts.

## FunctorFilter


### FunctorFilter


A `Functor` that can filter elements during mapping. `filter_map` applies a function that may return `None` to discard elements, combining mapping and filtering in a single pass.

#### Signature

``` rust
pub trait FunctorFilter: Functor {
    fn filter_map<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Option<B>) -> Self::Of<B>;

    fn filter<A: Clone>(fa: Self::Of<A>, pred: impl Fn(&A) -> bool) -> Self::Of<A> {
        Self::filter_map(fa, |a| if pred(&a) { Some(a) } else { None })
    }
}
```

#### Methods

| Method              | Description                                                                                                                         |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------|
| `filter_map(fa, f)` | Apply `f` to each element; keep only those where `f` returns `Some`. This is the required method that implementations must provide. |
| `filter(fa, pred)`  | Keep only elements for which `pred` returns `true`. Default implementation delegates to `filter_map`. Requires `A: Clone`.          |

#### Laws


- **Identity:** `filter_map(fa, Some) == fa` — mapping with `Some` (which never discards) is a no-op.
- **Composition:** `filter_map(filter_map(fa, f), g) == filter_map(fa, |a| f(a).and_then(g))` — two successive filter-maps can be fused into one.


#### Instances

| Type constructor | `Of<A>`     | Notes                                                                      |
|------------------|-------------|----------------------------------------------------------------------------|
| `OptionF`        | `Option<A>` | Delegates to `Option::and_then`. Available in `no_std`.                    |
| `VecF`           | `Vec<A>`    | Uses `Iterator::filter_map` internally. Requires `alloc` or `std` feature. |

`ResultF<E>` does not implement `FunctorFilter` because filtering a `Result` would require a default error value (`E: Default`), which is too restrictive.

#### Example

``` rust
use karpal_std::prelude::*;

// filter_map: keep only positive values, doubled
let nums = vec![1, -2, 3, -4, 5];
let result = VecF::filter_map(nums, |x| {
    if x > 0 { Some(x * 2) } else { None }
});
assert_eq!(result, vec![2, 6, 10]);

// filter: keep only even numbers
let nums = vec![1, 2, 3, 4, 5, 6];
let evens = VecF::filter(nums, |x| x % 2 == 0);
assert_eq!(evens, vec![2, 4, 6]);

// With OptionF: filter_map acts like and_then
let value = OptionF::filter_map(Some(10), |x| {
    if x > 5 { Some(x * 3) } else { None }
});
assert_eq!(value, Some(30));

let rejected = OptionF::filter_map(Some(2), |x| {
    if x > 5 { Some(x * 3) } else { None }
});
assert_eq!(rejected, None);
```


## Selective


### Selective


An `Applicative` that can conditionally apply effects. `Selective` sits between `Applicative` and `Monad` in expressive power: it can branch on a value inside the functor without requiring full monadic bind. The branching is encoded using `Result<A, B>` where `Ok(a)` means "needs the function applied" and `Err(b)` means "already resolved."

#### Signature

``` rust
pub trait Selective: Applicative {
    fn select<A, B, F>(fab: Self::Of<Result<A, B>>, ff: Self::Of<F>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}
```

#### Methods

| Method            | Description                                                                                                                               |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
| `select(fab, ff)` | If `fab` contains `Ok(a)`, apply the function inside `ff` to produce `B`. If `fab` contains `Err(b)`, return `b` directly, ignoring `ff`. |

#### Laws


- **Identity:** `select(fmap(Err, x), _) == x` — when every value is already resolved (wrapped in `Err`), the function argument is never used and the original values pass through unchanged.


#### Instances

| Type constructor | `Of<A>`     | Notes                                                                                                        |
|------------------|-------------|--------------------------------------------------------------------------------------------------------------|
| `OptionF`        | `Option<A>` | `None` propagates. `Some(Ok(a))` applies the function if present. `Some(Err(b))` returns `Some(b)` directly. |

#### Branching semantics

The `Result` inside the first argument encodes a choice:

| `fab`          | `ff`      | Result                                         |
|----------------|-----------|------------------------------------------------|
| `Some(Ok(a))`  | `Some(f)` | `Some(f(a))` — function is applied             |
| `Some(Ok(a))`  | `None`    | `None` — function needed but absent            |
| `Some(Err(b))` | *(any)*   | `Some(b)` — already resolved, function ignored |
| `None`         | *(any)*   | `None` — no value to branch on                 |

#### Example

``` rust
use karpal_std::prelude::*;

// Ok branch: the function is applied
let result = OptionF::select(
    Some(Ok(3i32)),
    Some(|x: i32| x * 2),
);
assert_eq!(result, Some(6));

// Err branch: already resolved, function is ignored
let result = OptionF::select(
    Some(Err(42i32)),
    Some(|_x: i32| 0),
);
assert_eq!(result, Some(42));

// None propagation: no value means no result
let result = OptionF::select(
    None::<Result<i32, i32>>,
    Some(|x: i32| x * 2),
);
assert_eq!(result, None);

// Ok branch but no function available
let result = OptionF::select(
    Some(Ok(3i32)),
    None::<fn(i32) -> i32>,
);
assert_eq!(result, None);
```

#### When to use Selective

`Selective` is useful when you need conditional logic inside a functorial pipeline but do not need the full power of `Monad`. Because the branching is encoded in the type (`Result<A, B>`) rather than in arbitrary closures, selective computations can be analyzed statically — making them suitable for scenarios like build systems or task schedulers where you want to inspect the structure of a computation before running it.


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


