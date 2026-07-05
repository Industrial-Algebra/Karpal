# Contravariant Family

Contravariant functors and their combinators: the duals of the covariant hierarchy.

Where a covariant `Functor` consumes a function `A -> B` to transform `F<A>` into `F<B>`, a `Contravariant` functor consumes a function going the *other way* -- `B -> A` -- to transform `F<A>` into `F<B>`. The canonical example is a predicate: if you have a predicate on integers and a function that extracts an integer from a string, you can build a predicate on strings.

The contravariant family splits into two branches that mirror the covariant hierarchy:

- **Product side:** `Contravariant` → `Divide` → `Divisible`
- **Sum side:** `Contravariant` → `Decide` → `Conclude`

All contravariant types in Karpal are **alloc-gated** -- they require the `std` or `alloc` feature because they use `Box<dyn Fn>` internally.

## Duality with the Covariant Hierarchy

Each contravariant trait is the dual of a corresponding covariant trait. The relationship is systematic: where the covariant side *produces* values, the contravariant side *consumes* them.

| Contravariant   | Covariant dual | Role                                              |
|-----------------|----------------|---------------------------------------------------|
| `Contravariant` | `Functor`      | Adapt input type via a function                   |
| `Divide`        | `Apply`        | Split input into parts, handle each independently |
| `Divisible`     | `Applicative`  | Identity for splitting (accepts anything)         |
| `Decide`        | `Alt`          | Route input to one of two handlers                |
| `Conclude`      | `Plus`         | Identity for routing (uninhabited input)          |


### Contravariant

A functor that maps over inputs rather than outputs.


#### Signature

``` rust
/// Contravariant functor: lifts a function `B -> A` into `F<A> -> F<B>`.
pub trait Contravariant: HKT {
    fn contramap<A: 'static, B>(
        fa: Self::Of<A>,
        f: impl Fn(B) -> A + 'static,
    ) -> Self::Of<B>;
}
```

Given a value of type `F<A>` and a function `B -> A`, `contramap` produces a value of type `F<B>`. The function goes in the *opposite* direction compared to `Functor::fmap`. The `'static` bounds are required because `PredicateF` stores the function inside a `Box<dyn Fn>`.

#### Laws


Identity

Contramapping the identity function changes nothing:

``` rust
Contravariant::contramap(fa, |x| x) == fa
```


Composition

Contramapping a composed function is the same as contramapping each function in sequence (note the reversed order):

``` rust
contramap(f . g, fa) == contramap(g, contramap(f, fa))
```


#### Instances

| Type constructor | `Of<T>`                  | Behavior                                                  | Feature gate     |
|------------------|--------------------------|-----------------------------------------------------------|------------------|
| `PredicateF`     | `Box<dyn Fn(T) -> bool>` | Pre-composes the adaptation function before the predicate | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::contravariant::{Contravariant, PredicateF};

// A predicate on integers
let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);

// Adapt it to work on strings by extracting the length
let str_len_positive = PredicateF::contramap(is_positive, |s: &str| s.len() as i32);

assert!(str_len_positive("hello"));  // len 5 > 0
assert!(!str_len_positive(""));      // len 0, not > 0
```


### Divide

The contravariant analogue of Apply -- split an input into parts and handle each independently.


#### Signature

``` rust
/// Divide: the contravariant analogue of Apply.
///
/// Given a way to split `C` into `(A, B)`, and contravariant functors over
/// `A` and `B`, produce a contravariant functor over `C`.
pub trait Divide: Contravariant {
    fn divide<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> (A, B) + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}
```

Where `Apply` combines two containers of *outputs*, `Divide` combines two consumers of *inputs*. The splitting function `f` decomposes the input `C` into a pair `(A, B)`, then each part is handled by its respective consumer.

For `PredicateF`, `divide` produces a predicate that splits the input and returns `true` only if **both** sub-predicates accept their respective parts.

#### Laws


Associativity

Nesting `divide` on the left or right produces equivalent results, as long as the splitting functions decompose the input consistently:

``` rust
divide(f, divide(g, a, b), c) == divide(h, a, divide(i, b, c))
```

Where `f`, `g`, `h`, and `i` are appropriate splitting functions that distribute the components equivalently.


#### Instances

| Type constructor | Behavior of `divide`                            | Feature gate     |
|------------------|-------------------------------------------------|------------------|
| `PredicateF`     | Splits the input, then returns `fa(a) && fb(b)` | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divide::Divide;

let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
let is_even: Box<dyn Fn(i32) -> bool> = Box::new(|x| x % 2 == 0);

// Split a tuple into its components, check both predicates
let both: Box<dyn Fn((i32, i32)) -> bool> =
    PredicateF::divide(|pair: (i32, i32)| pair, is_positive, is_even);

assert!(both((3, 4)));   // 3 > 0 AND 4 is even
assert!(!both((-1, 4))); // -1 is not > 0
assert!(!both((3, 3)));  // 3 is not even
```


### Divisible

The contravariant analogue of Applicative -- adds an identity element for Divide.


#### Signature

``` rust
/// Divisible: the contravariant analogue of Applicative.
///
/// Adds a `conquer` operation (the identity for `divide`), analogous to `pure`.
pub trait Divisible: Divide {
    fn conquer<A: 'static>() -> Self::Of<A>;
}
```

The `conquer` method produces a consumer that accepts any input and always succeeds. It is the identity element for `divide` -- dividing against a `conquer()` value has no effect on the result.

For `PredicateF`, `conquer` returns a predicate that is always `true`.

#### Laws


Left Identity

Dividing with `conquer()` on the left is equivalent to contramapping the second projection:

``` rust
divide(f, conquer(), fa) == contramap(snd . f, fa)
```


Right Identity

Dividing with `conquer()` on the right is equivalent to contramapping the first projection:

``` rust
divide(f, fa, conquer()) == contramap(fst . f, fa)
```


#### Instances

| Type constructor | Behavior of `conquer`                          | Feature gate     |
|------------------|------------------------------------------------|------------------|
| `PredicateF`     | Returns `Box::new(|_| true)` -- always accepts | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divisible::Divisible;

// conquer() produces a predicate that accepts everything
let p: Box<dyn Fn(i32) -> bool> = PredicateF::conquer();
assert!(p(42));
assert!(p(-1));
assert!(p(0));
```

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::divide::Divide;
use karpal_core::divisible::Divisible;

// Left identity: divide with conquer() on the left has no effect
let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
let result = PredicateF::divide(
    |a: i32| ((), a),
    PredicateF::conquer::<()>(),
    fa,
);
assert!(result(5));   // equivalent to the original predicate
assert!(!result(-3));
```


### Decide

The contravariant analogue of Alt -- route an input to one of two handlers.


#### Signature

``` rust
/// Decide: the contravariant analogue of Alt.
///
/// Given a way to split `C` into either `A` or `B`, and contravariant
/// functors over `A` and `B`, produce a contravariant functor over `C`.
pub trait Decide: Contravariant {
    fn choose<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> Result<A, B> + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}
```

Where `Divide` handles the product case (split into *both* parts), `Decide` handles the sum case (route to *one* handler). The classification function `f` returns `Result<A, B>`, which serves as Karpal's encoding of `Either`: `Ok(a)` routes to `fa`, and `Err(b)` routes to `fb`.

For `PredicateF`, `choose` classifies the input and delegates to whichever predicate matches.

#### Laws


Associativity

Nesting `choose` on the left or right produces equivalent results, as long as the routing functions classify consistently:

``` rust
choose(f, choose(g, a, b), c) == choose(h, a, choose(i, b, c))
```

Where `f`, `g`, `h`, and `i` are appropriate routing functions that distribute the cases equivalently.


#### Instances

| Type constructor | Behavior of `choose`                                                 | Feature gate     |
|------------------|----------------------------------------------------------------------|------------------|
| `PredicateF`     | Classifies input via `f`, then applies `fa` on `Ok` or `fb` on `Err` | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::decide::Decide;

let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
let is_short: Box<dyn Fn(String) -> bool> = Box::new(|s| s.len() < 5);

// Classify input: integers go to Ok, strings go to Err
let classifier = PredicateF::choose(
    |input: Result<i32, String>| input,
    is_positive,
    is_short,
);

assert!(classifier(Ok(5)));                          // 5 > 0
assert!(!classifier(Ok(-1)));                        // -1 not > 0
assert!(classifier(Err("hi".to_string())));          // len 2 < 5
assert!(!classifier(Err("hello world".to_string()))); // len 11, not < 5
```


### Conclude

The contravariant analogue of Plus -- the identity element for Decide.


#### Signature

``` rust
/// Conclude: the contravariant analogue of Plus.
///
/// Adds a `conclude` operation (the identity for `choose`).
/// `conclude` takes a function `A -> Infallible`, witnessing that `A` is
/// uninhabited -- so the resulting predicate is vacuously true.
pub trait Conclude: Decide {
    fn conclude<A: 'static>(
        f: impl Fn(A) -> core::convert::Infallible + 'static,
    ) -> Self::Of<A>;
}
```

The `conclude` method takes a function from `A` to `Infallible`. If such a function exists, it witnesses that `A` is uninhabited -- no value of type `A` can ever be constructed. The resulting consumer is vacuously valid: it will never be called with a real input.

Rust uses `core::convert::Infallible` as its bottom type (the equivalent of Haskell's `Void`). For inhabited types, the function body typically uses `unreachable!()` since it can never actually execute in well-typed code.

For `PredicateF`, `conclude` returns a predicate that is always `true`.

#### Laws


Left Identity

Choosing with `conclude(absurd)` on the left is equivalent to contramapping the right projection:

``` rust
choose(f, conclude(absurd), fa) == contramap(from_right . f, fa)
```


Right Identity

Choosing with `conclude(absurd)` on the right is equivalent to contramapping the left projection:

``` rust
choose(f, fa, conclude(absurd)) == contramap(from_left . f, fa)
```


#### Instances

| Type constructor | Behavior of `conclude`                            | Feature gate     |
|------------------|---------------------------------------------------|------------------|
| `PredicateF`     | Returns `Box::new(|_| true)` -- vacuously accepts | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::conclude::Conclude;

// conclude with an unreachable function -- the predicate is vacuously true
let p: Box<dyn Fn(i32) -> bool> = PredicateF::conclude(|_: i32| unreachable!());
assert!(p(42));
assert!(p(-1));
```

``` rust
use karpal_core::contravariant::PredicateF;
use karpal_core::decide::Decide;
use karpal_core::conclude::Conclude;

// Right identity: choosing with conclude on the right has no effect
let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
let result = PredicateF::choose(
    |a: i32| -> Result<i32, core::convert::Infallible> { Ok(a) },
    fa,
    PredicateF::conclude(|i: core::convert::Infallible| -> core::convert::Infallible { i }),
);
assert!(result(5));   // equivalent to the original predicate
assert!(!result(-3));
```


## Combining Both Branches

In practice, `Divide` and `Decide` complement each other. `Divide` handles product types (structs, tuples) by splitting into fields, while `Decide` handles sum types (enums) by routing to the matching variant. Together they let you build validators and predicates for complex data structures compositionally:

``` rust
use karpal_core::contravariant::{Contravariant, PredicateF};
use karpal_core::divide::Divide;
use karpal_core::decide::Decide;

// Field-level predicates
let name_valid: Box<dyn Fn(String) -> bool> = Box::new(|s| !s.is_empty());
let age_valid: Box<dyn Fn(i32) -> bool> = Box::new(|a| a >= 0 && a <= 150);

// Combine with Divide: validate a (name, age) pair
let person_valid: Box<dyn Fn((String, i32)) -> bool> =
    PredicateF::divide(|p: (String, i32)| p, name_valid, age_valid);

assert!(person_valid(("Alice".to_string(), 30)));
assert!(!person_valid(("".to_string(), 30)));       // empty name
assert!(!person_valid(("Alice".to_string(), -1)));   // negative age

// Sum-type routing with Decide: handle either a string or an integer
let str_check: Box<dyn Fn(String) -> bool> = Box::new(|s| s.len() < 10);
let int_check: Box<dyn Fn(i32) -> bool> = Box::new(|n| n > 0);

let either_check = PredicateF::choose(
    |input: Result<String, i32>| input,
    str_check,
    int_check,
);

assert!(either_check(Ok("short".to_string())));
assert!(!either_check(Err(-5)));
```


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


