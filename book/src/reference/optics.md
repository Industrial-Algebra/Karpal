# Optics

Profunctor optics: first-class field accessors and pattern matchers.

Optics let you focus on parts of a data structure -- reading, writing, and transforming nested fields or enum variants -- without breaking encapsulation. Karpal provides a full hierarchy of optic types, each constrained by a different profunctor class:

| Optic                   | Focus                   | Profunctor constraint | Read | Write |
|-------------------------|-------------------------|-----------------------|------|-------|
| [Iso](#iso)             | Exactly 1 (isomorphism) | Profunctor            | yes  | yes   |
| [Lens](#lens)           | Exactly 1 (field)       | Strong                | yes  | yes   |
| [Prism](#prism)         | 0 or 1 (variant)        | Choice                | yes  | yes   |
| [Traversal](#traversal) | 0 to many               | Traversing            | yes  | yes   |
| [Getter](#getter)       | Exactly 1 (read-only)   | --                    | yes  | no    |
| [Review](#review)       | Construction only       | --                    | no   | yes   |
| [Setter](#setter)       | Modify only             | --                    | no   | yes   |
| [Fold](#fold)           | 0 to many (read-only)   | --                    | yes  | no    |

Optics form a subtyping hierarchy -- every Iso can be used as a Lens or Prism, every Lens can be used as a Getter, Setter, Traversal, or Fold, and so on. Karpal provides explicit `to_*` conversion methods for these relationships.

All optic types live in the `karpal-optics` crate and implement the [`Optic`](#optic-trait) marker trait.


### Optic

Marker trait for the optic family.


#### Signature

``` rust
/// Marker trait for all optics.
///
/// This trait exists to unify the optic family under a single taxonomy.
/// Concrete optic types (Lens, Prism, etc.) implement this trait.
pub trait Optic {}
```

`Optic` carries no methods. It exists solely to classify types as optics, which is useful for trait bounds and documentation. All concrete optic types implement `Optic`: `Iso`, `Lens`, `ComposedLens`, `Prism`, `Getter`, `ComposedGetter`, `Review`, `Setter`, `Traversal`, `ComposedTraversal`, `Fold`, and `ComposedFold`.


### Iso

An isomorphism: a lossless, reversible conversion between two representations.


#### Struct definition

``` rust
pub struct Iso<S, T, A, B> {
    forward: fn(&S) -> A,
    backward: fn(B) -> T,
}

pub type SimpleIso<S, A> = Iso<S, S, A, A>;
```

An `Iso` witnesses that `S` and `A` carry the same information. It is the strongest optic -- it requires only `Profunctor` (no `Strong` or `Choice`), and can be converted to any other optic type.

#### Methods

``` rust
impl<S, T, A, B> Iso<S, T, A, B> {
    pub fn new(forward: fn(&S) -> A, backward: fn(B) -> T) -> Self;
    pub fn get(&self, s: &S) -> A;
    pub fn review(&self, b: B) -> T;
    pub fn set(&self, _s: S, b: B) -> T;

    /// Profunctor encoding -- only requires Profunctor (weakest constraint).
    pub fn transform<P: Profunctor>(&self, pab: P::P<A, B>) -> P::P<S, T>;

    // Conversions
    pub fn to_getter(&self) -> Getter<S, A>;
    pub fn to_review(&self) -> Review<T, B>;
    pub fn to_fold(&self) -> Fold<S, A>;
}

impl<S: Clone, T, A, B> Iso<S, T, A, B> {
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T;
    pub fn to_lens(&self) -> ComposedLens<S, T, A, B>;   // boxed (captures backward)
    pub fn to_setter(&self) -> Setter<S, T, A, B>;
    pub fn to_traversal(&self) -> Traversal<S, T, A, B>;
}
```

#### Laws


Roundtrip (forward-backward)

``` rust
iso.review(iso.get(&s)) == s
```


Roundtrip (backward-forward)

``` rust
iso.get(&iso.review(b)) == b
```


#### Example

``` rust
use karpal_optics::{Iso, SimpleIso};

// Celsius <-> Fahrenheit
let temp: SimpleIso<f64, f64> = Iso::new(
    |c: &f64| c * 9.0 / 5.0 + 32.0,  // forward: C -> F
    |f: f64| (f - 32.0) * 5.0 / 9.0,   // backward: F -> C
);

assert!((temp.get(&100.0) - 212.0).abs() < 1e-10);
assert!((temp.review(32.0) - 0.0).abs() < 1e-10);

// Modify in the "other" representation
let result = temp.over(0.0, |f| f + 18.0); // add 18F to 0C
assert!((result - 10.0).abs() < 1e-10);    // = 10C
```


### Lens

A first-class getter/setter pair for focusing on a field inside a product type.


#### Struct definition

``` rust
/// A van Laarhoven-style lens encoded with getter/setter function pointers.
///
/// `S` -- source type, `T` -- modified source type,
/// `A` -- focus type, `B` -- replacement type.
pub struct Lens<S, T, A, B> {
    getter: fn(&S) -> A,
    setter: fn(S, B) -> T,
}

/// A simple (monomorphic) lens where `S == T` and `A == B`.
pub type SimpleLens<S, A> = Lens<S, S, A, A>;
```

The four type parameters support **polymorphic update**: you can replace a field of type `A` with a value of type `B`, changing the source from `S` to `T`. In practice, most lenses are *simple* (monomorphic), where `S == T` and `A == B`. The `SimpleLens` type alias covers this common case.

#### Methods

``` rust
impl<S, T, A, B> Lens<S, T, A, B> {
    /// Create a new lens from a getter and setter.
    pub fn new(getter: fn(&S) -> A, setter: fn(S, B) -> T) -> Self;

    /// Extract the focus from the source.
    pub fn get(&self, s: &S) -> A;

    /// Replace the focus, producing a new source.
    pub fn set(&self, s: S, b: B) -> T;

    /// Chain another lens to focus deeper, producing a ComposedLens.
    /// Requires all type parameters to be `'static`.
    pub fn then<X, Y>(self, inner: Lens<A, B, X, Y>) -> ComposedLens<S, T, X, Y>
    where
        S: 'static, T: 'static, A: 'static, B: 'static,
        X: 'static, Y: 'static;
}

impl<S: Clone, T, A, B> Lens<S, T, A, B> {
    /// Modify the focus by applying a function. Requires `S: Clone`.
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T;

    /// Profunctor encoding: transform a `P<A, B>` into a `P<S, T>`.
    /// Requires `S: Clone` and `Strong` profunctor `P`.
    /// All type parameters must be `'static`.
    pub fn transform<P: Strong>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static, T: 'static, A: 'static, B: 'static;

    // Conversions (all type params must be 'static)
    pub fn to_getter(&self) -> Getter<S, A>;
    pub fn to_setter(&self) -> Setter<S, T, A, B>;
    pub fn to_traversal(&self) -> Traversal<S, T, A, B>;
    pub fn to_fold(&self) -> Fold<S, A>;
}
```

#### How `transform` works (Strong)

The `transform` method connects a concrete lens to the profunctor hierarchy through the [`Strong`](profunctor-family.md) trait. Given any `Strong` profunctor `P` and a value `pab: P<A, B>`, it produces `P<S, T>` by:

1.  `P::first(pab)` lifts to `P<(A, S), (B, S)>`
2.  `P::dimap` pre-composes with `|s| (get(s), s)` and post-composes with `|(b, s)| set(s, b)`

``` rust
pub fn transform<P: Strong>(&self, pab: P::P<A, B>) -> P::P<S, T>
where
    S: 'static, T: 'static, A: 'static, B: 'static,
{
    let getter = self.getter;
    let setter = self.setter;
    let first_pab = P::first::<A, B, S>(pab);
    P::dimap(
        move |s: S| {
            let a = getter(&s);
            (a, s)
        },
        move |(b, s)| setter(s, b),
        first_pab,
    )
}
```

#### Laws

A well-behaved lens must satisfy three laws:


GetSet

Setting a value you just got changes nothing:

``` rust
lens.set(s.clone(), lens.get(&s)) == s
```


SetGet

Getting after setting yields the value you set:

``` rust
lens.get(&lens.set(s, b)) == b
```


SetSet

Setting twice is the same as setting once with the second value:

``` rust
lens.set(lens.set(s.clone(), b1), b2) == lens.set(s, b2)
```


#### Example

``` rust
use karpal_optics::{Lens, SimpleLens};

#[derive(Debug, Clone, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

let age_lens: SimpleLens<Person, u32> = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let alice = Person { name: "Alice".into(), age: 30 };

// get
assert_eq!(age_lens.get(&alice), 30);

// set
let updated = age_lens.set(alice.clone(), 31);
assert_eq!(updated.age, 31);

// over -- modify the focus with a function
let updated = age_lens.over(alice.clone(), |a| a + 1);
assert_eq!(updated.age, 31);
```

#### Profunctor usage with FnP

``` rust
use karpal_optics::{Lens, SimpleLens};
use karpal_profunctor::FnP;

let age_lens: SimpleLens<Person, u32> = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let increment: Box<dyn Fn(u32) -> u32> = Box::new(|age| age + 1);
let transform_fn = age_lens.transform::<FnP>(increment);

let result = transform_fn(Person { name: "Alice".into(), age: 30 });
assert_eq!(result.age, 31);
```


### ComposedLens

A lens built by chaining two or more lenses for deep field access.


#### Struct definition

``` rust
/// A composed lens built from two lenses chained together.
///
/// Unlike `Lens`, which stores `fn` pointers, a composed lens stores
/// boxed closures because closure composition cannot produce `fn` pointers.
pub struct ComposedLens<S, T, X, Y> {
    getter: Box<dyn Fn(&S) -> X>,
    setter: Box<dyn Fn(S, Y) -> T>,
}

/// A simple (monomorphic) composed lens where `S == T` and `X == Y`.
pub type SimpleComposedLens<S, X> = ComposedLens<S, S, X, X>;
```

`ComposedLens` is produced by calling `Lens::then()` or `ComposedLens::then()`. It stores `Box<dyn Fn>` closures instead of `fn` pointers because closure composition captures the outer lens's getter and setter, which cannot be represented as bare function pointers.

#### Methods

``` rust
impl<S, T, X, Y> ComposedLens<S, T, X, Y> {
    /// Extract the deeply-nested focus from the source.
    pub fn get(&self, s: &S) -> X;

    /// Replace the deeply-nested focus, producing a new source.
    pub fn set(&self, s: S, y: Y) -> T;
}

impl<S: Clone, T, X, Y> ComposedLens<S, T, X, Y> {
    /// Modify the deeply-nested focus by applying a function. Requires `S: Clone`.
    pub fn over(&self, s: S, f: impl FnOnce(X) -> Y) -> T;

    /// Chain another lens to focus even deeper.
    /// All type parameters must be `'static`.
    pub fn then<U, V>(self, inner: Lens<X, Y, U, V>) -> ComposedLens<S, T, U, V>
    where
        S: 'static, T: 'static, X: 'static, Y: 'static,
        U: 'static, V: 'static;
}
```

#### No `transform` on ComposedLens

`ComposedLens` does **not** provide a `transform` method. For profunctor-level composition, use nested `Lens::transform` calls on the original lenses instead:

``` rust
// Instead of composed_lens.transform::<P>(pab), write:
let result = outer.transform::<P>(inner.transform::<P>(pab));
```

This avoids the need for `Rc`/`Arc` to share closures at the profunctor level and preserves the clean semantics of the profunctor encoding.

#### Example

``` rust
use karpal_optics::{Lens, SimpleLens};

#[derive(Debug, Clone, PartialEq)]
struct Company {
    name: String,
    ceo: Person,
}

let ceo_lens: SimpleLens<Company, Person> = Lens::new(
    |c: &Company| c.ceo.clone(),
    |c, ceo| Company { ceo, ..c },
);

let age_lens: SimpleLens<Person, u32> = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

// Compose: Company -> ceo -> age
let ceo_age = ceo_lens.then(age_lens);

let acme = Company {
    name: "Acme".into(),
    ceo: Person { name: "Alice".into(), age: 30 },
};

assert_eq!(ceo_age.get(&acme), 30);

let updated = ceo_age.set(acme.clone(), 31);
assert_eq!(updated.ceo.age, 31);

let updated = ceo_age.over(acme, |age| age + 1);
assert_eq!(updated.ceo.age, 31);
```


### Prism

A first-class pattern matcher for focusing on one variant of a sum type.


#### Struct definition

``` rust
/// A prism focuses on one variant of a sum type.
///
/// `S` -- source type, `T` -- modified source type,
/// `A` -- focus type (the variant's inner value), `B` -- replacement type.
///
/// Where a Lens uses Strong to decompose products, a Prism uses Choice
/// to decompose coproducts.
pub struct Prism<S, T, A, B> {
    /// Attempt to match. `Ok(a)` = matched, `Err(t)` = didn't match (pass-through).
    match_: fn(S) -> Result<A, T>,
    /// Construct a `T` from the replacement value.
    build: fn(B) -> T,
}

/// A simple (monomorphic) prism where `S == T` and `A == B`.
pub type SimplePrism<S, A> = Prism<S, S, A, A>;
```

A `Prism` is the dual of a `Lens`. Where a lens focuses on a field that is always present (product types), a prism focuses on a variant that may or may not be present (sum types). The `match_` function returns `Ok(a)` if the variant matches and `Err(t)` if it does not, allowing the original value to pass through unchanged.

#### Methods

``` rust
impl<S, T, A, B> Prism<S, T, A, B> {
    /// Create a new prism from a match function and a build function.
    pub fn new(match_: fn(S) -> Result<A, T>, build: fn(B) -> T) -> Self;

    /// Try to extract the focus. Returns `Some(a)` if the variant matches.
    /// Requires `S: Clone`.
    pub fn preview(&self, s: &S) -> Option<A>
    where
        S: Clone;

    /// Construct a `T` from a replacement value (inject/construct).
    pub fn review(&self, b: B) -> T;

    /// Replace the focus if the variant matches; otherwise pass through.
    pub fn set(&self, s: S, b: B) -> T;

    /// Modify the focus if the variant matches; otherwise pass through.
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T;

    /// Profunctor encoding: transform a `P<A, B>` into a `P<S, T>`.
    /// Requires `Choice` profunctor `P`.
    /// All type parameters must be `'static`.
    pub fn transform<P: Choice>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static, T: 'static, A: 'static, B: 'static;

    // Conversions
    pub fn to_review(&self) -> Review<T, B>;
    pub fn to_setter(&self) -> Setter<S, T, A, B>;
    pub fn to_traversal(&self) -> Traversal<S, T, A, B>; // S: Clone
    pub fn to_fold(&self) -> Fold<S, A>;                  // S: Clone
}
```

#### How `transform` works (Choice)

The `transform` method connects a concrete prism to the profunctor hierarchy through the [`Choice`](profunctor-family.md) trait. Given any `Choice` profunctor `P` and a value `pab: P<A, B>`, it produces `P<S, T>` by:

1.  `P::right(pab)` lifts to `P<Result<T, A>, Result<T, B>>`
2.  `P::dimap` pre-composes with `match_` (swapping `Ok`/`Err` arms) and post-composes with `build` (reassembling)

The arm-swapping (`Ok` to `Err`, `Err` to `Ok` in pre-composition) is necessary because `Choice::right` acts on the `Err` branch of `Result`.

``` rust
pub fn transform<P: Choice>(&self, pab: P::P<A, B>) -> P::P<S, T>
where
    S: 'static, T: 'static, A: 'static, B: 'static,
{
    let match_ = self.match_;
    let build = self.build;
    let right_pab = P::right::<A, B, T>(pab);
    P::dimap(
        move |s: S| match match_(s) {
            Ok(a) => Err(a),  // focus found -- Err arm for Choice::right
            Err(t) => Ok(t),  // no match -- Ok arm passes through
        },
        move |result: Result<T, B>| match result {
            Ok(t) => t,          // passed through unchanged
            Err(b) => build(b),  // transformed, rebuild
        },
        right_pab,
    )
}
```

#### Laws

A well-behaved prism must satisfy two laws:


PreviewReview

If a preview succeeds, reviewing the result reconstructs the original:

``` rust
if let Some(a) = prism.preview(&s) {
    assert_eq!(prism.review(a), s);
}
```


ReviewPreview

Previewing a value built with review always succeeds and returns the original value:

``` rust
assert_eq!(prism.preview(&prism.review(b)), Some(b));
```


#### Example

``` rust
use karpal_optics::{Prism, SimplePrism};

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

let circle: SimplePrism<Shape, f64> = Prism::new(
    |s| match s {
        Shape::Circle(r) => Ok(r),
        Shape::Rectangle(w, h) => Err(Shape::Rectangle(w, h)),
    },
    Shape::Circle,
);

// preview -- extract if the variant matches
assert_eq!(circle.preview(&Shape::Circle(5.0)), Some(5.0));
assert_eq!(circle.preview(&Shape::Rectangle(3.0, 4.0)), None);

// review -- construct the variant
assert_eq!(circle.review(10.0), Shape::Circle(10.0));

// set -- replace the focus if matched
assert_eq!(circle.set(Shape::Circle(5.0), 10.0), Shape::Circle(10.0));
assert_eq!(
    circle.set(Shape::Rectangle(3.0, 4.0), 10.0),
    Shape::Rectangle(3.0, 4.0),
);

// over -- modify the focus if matched
assert_eq!(
    circle.over(Shape::Circle(5.0), |r| r * 2.0),
    Shape::Circle(10.0),
);
```

#### Profunctor usage with FnP

``` rust
use karpal_optics::{Prism, SimplePrism};
use karpal_profunctor::FnP;

let circle: SimplePrism<Shape, f64> = Prism::new(
    |s| match s {
        Shape::Circle(r) => Ok(r),
        Shape::Rectangle(w, h) => Err(Shape::Rectangle(w, h)),
    },
    Shape::Circle,
);

let double: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 2.0);
let transform_fn = circle.transform::<FnP>(double);

// Matching variant is transformed
assert_eq!(transform_fn(Shape::Circle(5.0)), Shape::Circle(10.0));

// Non-matching variant passes through unchanged
assert_eq!(
    transform_fn(Shape::Rectangle(3.0, 4.0)),
    Shape::Rectangle(3.0, 4.0),
);
```


### Getter

A read-only optic that extracts a single value from a source.


#### Struct definition

``` rust
pub struct Getter<S, A> {
    get: fn(&S) -> A,
}

/// Composed variant (from .then() or conversions).
pub struct ComposedGetter<S, A> {
    get: Box<dyn Fn(&S) -> A>,
}
```

A `Getter` is the read-only component of a `Lens`. It can extract a focus but cannot modify it. Getters are typically obtained via `Lens::to_getter()` or `Iso::to_getter()`.

#### Methods

``` rust
impl<S, A> Getter<S, A> {
    pub fn new(get: fn(&S) -> A) -> Self;
    pub fn get(&self, s: &S) -> A;
    pub fn then<B>(self, inner: Getter<A, B>) -> ComposedGetter<S, B>;
}
```

#### Example

``` rust
use karpal_optics::{Lens, SimpleLens};

let age_lens: SimpleLens<Person, u32> = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let getter = age_lens.to_getter();
assert_eq!(getter.get(&alice), 30);
```


### Review

A write-only optic that constructs a target from a value.


#### Struct definition

``` rust
pub struct Review<T, B> {
    build: fn(B) -> T,
}
```

A `Review` is the construction component of a `Prism` or `Iso`. It can build a target but cannot inspect one. At the profunctor level, `Review` corresponds to `TaggedF`, which is `Choice` but deliberately not `Strong` -- this enforces the write-only constraint at the type level.

#### Methods

``` rust
impl<T, B> Review<T, B> {
    pub fn new(build: fn(B) -> T) -> Self;
    pub fn review(&self, b: B) -> T;
}
```

#### Example

``` rust
use karpal_optics::{Prism, SimplePrism};

let circle: SimplePrism<Shape, f64> = Prism::new(/* ... */);

let review = circle.to_review();
assert_eq!(review.review(5.0), Shape::Circle(5.0));
```


### Setter

A modify-only optic that can transform foci but not read them independently.


#### Struct definition

``` rust
pub struct Setter<S, T, A, B> {
    modify: Box<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}

pub type SimpleSetter<S, A> = Setter<S, S, A, A>;
```

A `Setter` always uses boxed closures since it is typically derived from composition or conversion (e.g. `Lens::to_setter()` or `Prism::to_setter()`). The `modify` closure takes the source and a transformation function, and returns the modified source.

#### Methods

``` rust
impl<S, T, A, B> Setter<S, T, A, B> {
    pub fn new(modify: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static) -> Self;
    pub fn over(&self, s: S, f: impl Fn(A) -> B) -> T;
    pub fn set(&self, s: S, b: B) -> T where B: Clone;
}
```

#### Laws


Identity

Modifying with identity changes nothing:

``` rust
setter.over(s, |x| x) == s
```


#### Example

``` rust
use karpal_optics::{Lens, SimpleLens};

let age_lens: SimpleLens<Person, u32> = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let setter = age_lens.to_setter();
let updated = setter.over(alice, |age| age + 1);
assert_eq!(updated.age, 31);

let updated = setter.set(alice, 99);
assert_eq!(updated.age, 99);
```


### Traversal

A multi-focus optic that can get and modify zero or more foci.


#### Struct definition

``` rust
pub struct Traversal<S, T, A, B> {
    get_all: Rc<dyn Fn(&S) -> Vec<A>>,
    modify_all: Rc<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}

pub type SimpleTraversal<S, A> = Traversal<S, S, A, A>;

/// Composed variant (from .then()).
pub struct ComposedTraversal<S, T, A, B> {
    get_all: Box<dyn Fn(&S) -> Vec<A>>,
    modify_all: Box<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}
```

A `Traversal` generalizes both `Lens` (exactly one focus) and `Prism` (zero or one focus) to zero or more foci. It stores `Rc<dyn Fn>` closures to allow sharing between `get_all`, `modify_all`, and the `transform` method.

#### Methods

``` rust
impl<S, T, A, B> Traversal<S, T, A, B> {
    pub fn new(
        get_all: impl Fn(&S) -> Vec<A> + 'static,
        modify_all: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static,
    ) -> Self;

    pub fn get_all(&self, s: &S) -> Vec<A>;
    pub fn over(&self, s: S, f: impl Fn(A) -> B) -> T;
    pub fn set(&self, s: S, b: B) -> T where B: Clone;

    /// Profunctor encoding via Traversing::wander.
    pub fn transform<P: Traversing>(&self, pab: P::P<A, B>) -> P::P<S, T>;

    pub fn to_fold(&self) -> Fold<S, A>;
    pub fn then<X, Y>(self, inner: Traversal<A, B, X, Y>) -> ComposedTraversal<S, T, X, Y>;
}
```

#### How `transform` works (Traversing)

The `transform` method connects a traversal to the profunctor hierarchy through [`Traversing`](profunctor-family.md#traversing). It calls `P::wander(get_all, modify_all, pab)` -- each profunctor instance decides how to interpret the traversal:

- `FnP` uses `modify_all` to apply the function to every focus
- `ForgetF<R: Monoid>` uses `get_all` to extract every focus, maps each through the profunctor, and combines results with `Monoid`

#### Laws


Identity

``` rust
trav.over(s, |x| x) == s
```


Composition

``` rust
trav.over(trav.over(s, f), g) == trav.over(s, |x| g(f(x)))
```


#### Example

``` rust
use karpal_optics::{Traversal, SimpleTraversal};
use karpal_profunctor::{FnP, ForgetF};

// Traverse all elements of a Vec
let each: SimpleTraversal<Vec<i32>, i32> = Traversal::new(
    |v: &Vec<i32>| v.clone(),
    |v: Vec<i32>, f: &dyn Fn(i32) -> i32| v.into_iter().map(f).collect(),
);

assert_eq!(each.get_all(&vec![1, 2, 3]), vec![1, 2, 3]);
assert_eq!(each.over(vec![1, 2, 3], |x| x * 10), vec![10, 20, 30]);
assert_eq!(each.set(vec![1, 2, 3], 0), vec![0, 0, 0]);

// Profunctor usage: FnP applies function to all elements
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
let f = each.transform::<FnP>(double);
assert_eq!(f(vec![1, 2, 3]), vec![2, 4, 6]);

// Profunctor usage: ForgetF<String> concatenates results
let to_str: Box<dyn Fn(i32) -> String> = Box::new(|x| x.to_string());
let g = each.transform::<ForgetF<String>>(to_str);
assert_eq!(g(vec![1, 2, 3]), "123");
```


### Fold

A read-only multi-focus optic with powerful aggregation methods.


#### Struct definition

``` rust
pub struct Fold<S, A> {
    fold_fn: Box<dyn Fn(&S) -> Vec<A>>,
}

/// Composed variant (from .then()).
pub struct ComposedFold<S, A> {
    fold_fn: Box<dyn Fn(&S) -> Vec<A>>,
}
```

A `Fold` is the read-only counterpart of a `Traversal`. It extracts zero or more foci from a source, with convenience methods for aggregation via `Monoid`. Folds are typically obtained from `Lens::to_fold()`, `Prism::to_fold()`, or `Traversal::to_fold()`.

#### Methods

``` rust
impl<S, A> Fold<S, A> {
    pub fn new(fold_fn: impl Fn(&S) -> Vec<A> + 'static) -> Self;
    pub fn get_all(&self, s: &S) -> Vec<A>;
    pub fn fold_map<R: Monoid>(&self, s: &S, f: impl Fn(A) -> R) -> R;
    pub fn any(&self, s: &S, f: impl Fn(&A) -> bool) -> bool;
    pub fn all(&self, s: &S, f: impl Fn(&A) -> bool) -> bool;
    pub fn find(&self, s: &S, f: impl Fn(&A) -> bool) -> Option<A>;
    pub fn length(&self, s: &S) -> usize;
    pub fn then<B>(self, inner: Fold<A, B>) -> ComposedFold<S, B>;
}
```

The `fold_map` method maps each focus to a `Monoid` value and combines them. This is the core aggregation operation -- `any`, `all`, `find`, and `length` are convenience wrappers.

#### Example

``` rust
use karpal_optics::Fold;

let fold = Fold::new(|v: &Vec<i32>| v.clone());

assert_eq!(fold.get_all(&vec![1, 2, 3]), vec![1, 2, 3]);

// fold_map: sum all elements (i32 Monoid is additive)
let sum: i32 = fold.fold_map(&vec![1, 2, 3], |x| x);
assert_eq!(sum, 6);

// fold_map: concatenate string representations
let s: String = fold.fold_map(&vec![1, 2, 3], |x| x.to_string());
assert_eq!(s, "123");

// Predicate queries
assert!(fold.any(&vec![1, 2, 3], |x| *x > 2));
assert!(fold.all(&vec![1, 2, 3], |x| *x > 0));
assert_eq!(fold.find(&vec![1, 2, 3], |x| *x > 1), Some(2));
assert_eq!(fold.length(&vec![1, 2, 3]), 3);
```


## Optic Conversions

Optics form a subtyping hierarchy. A stronger optic can always be used where a weaker one is expected. Karpal provides explicit `to_*` methods for these conversions:

``` rust
  Iso
  / \
Lens  Prism
 |  \  / |
 | Traversal
 |    |    |
 | +--+---+
 | |      |
Getter  Setter  Review
   \    |
    \   |
     Fold
```

| From        | Available conversions                                                       |
|-------------|-----------------------------------------------------------------------------|
| `Iso`       | `to_lens`, `to_getter`, `to_review`, `to_setter`, `to_traversal`, `to_fold` |
| `Lens`      | `to_getter`, `to_setter`, `to_traversal`, `to_fold`                         |
| `Prism`     | `to_review`, `to_setter`, `to_traversal`, `to_fold`                         |
| `Traversal` | `to_fold`                                                                   |

Note: `Iso::to_lens()` returns a `ComposedLens` (not `Lens`) because the conversion captures the iso's `backward` function in a closure, which cannot be represented as a bare `fn` pointer.

## Composing Lenses with `.then()`

Lenses compose naturally via the `.then()` method. Each call to `then` produces a `ComposedLens` that focuses one level deeper. You can chain as many lenses as needed for deep access into nested structures.

``` rust
use karpal_optics::{Lens, SimpleLens};

#[derive(Debug, Clone, PartialEq)]
struct Address {
    street: String,
    city: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Employee {
    name: String,
    addr: Address,
}

#[derive(Debug, Clone, PartialEq)]
struct Org {
    title: String,
    lead: Employee,
}

let lead_lens: SimpleLens<Org, Employee> = Lens::new(
    |o: &Org| o.lead.clone(),
    |o, lead| Org { lead, ..o },
);

let addr_lens: SimpleLens<Employee, Address> = Lens::new(
    |e: &Employee| e.addr.clone(),
    |e, addr| Employee { addr, ..e },
);

let city_lens: SimpleLens<Address, String> = Lens::new(
    |a: &Address| a.city.clone(),
    |a, city| Address { city, ..a },
);

// Three-deep composition: Org -> lead -> addr -> city
let org_city = lead_lens.then(addr_lens).then(city_lens);

let org = Org {
    title: "R&D".into(),
    lead: Employee {
        name: "Alice".into(),
        addr: Address {
            street: "123 Main St".into(),
            city: "Springfield".into(),
        },
    },
};

// Read a deeply nested field
assert_eq!(org_city.get(&org), "Springfield");

// Update a deeply nested field
let updated = org_city.set(org.clone(), "Shelbyville".into());
assert_eq!(updated.lead.addr.city, "Shelbyville");
assert_eq!(updated.lead.addr.street, "123 Main St");

// Modify a deeply nested field with a function
let updated = org_city.over(org, |c| c.to_uppercase());
assert_eq!(updated.lead.addr.city, "SPRINGFIELD");
```

For profunctor-level composition (where you need `transform`), nest the original lens transforms instead of using the composed lens:

``` rust
use karpal_profunctor::FnP;

let ceo_lens: SimpleLens<Company, Person> = /* ... */;
let age_lens: SimpleLens<Person, u32> = /* ... */;

let increment: Box<dyn Fn(u32) -> u32> = Box::new(|age| age + 1);

// Nested transform: equivalent to composed_lens.over(company, |age| age + 1)
let transform_fn = ceo_lens.transform::<FnP>(age_lens.transform::<FnP>(increment));
```


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


