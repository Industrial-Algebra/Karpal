use crate::compose::ComposeF;
use crate::functor::Functor;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::EnvF;
use crate::hkt::{HKT, HKT2, IdentityF};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;
#[cfg(feature = "std")]
use std::rc::Rc;

/// An adjunction F ⊣ U between two type constructors.
///
/// This encodes the fundamental relationship where F is left adjoint to U,
/// meaning there is a natural isomorphism:
///
/// ```text
/// Hom(F(A), B) ≅ Hom(A, U(B))
/// ```
///
/// Expressed via unit/counit:
/// - `unit: A -> U(F(A))` — the universal arrow from A to U
/// - `counit: F(U(B)) -> B` — the universal arrow from F to B
///
/// Laws (triangle identities):
/// - `counit(F::fmap(fa, unit)) == fa` for all fa: F::Of<A>
/// - `U::fmap(unit(a), counit) == a` for all a: U::Of<A>
///
/// Every adjunction F ⊣ U gives rise to:
/// - A monad on `U . F` (via `unit` as pure, derived join)
/// - A comonad on `F . U` (via `counit` as extract, derived duplicate)
///
/// Note: `F` and `U` are bounded by `HKT` rather than `Functor` because
/// some useful right adjoints (like `ReaderF<E>`) cannot implement `Functor`
/// due to `'static` limitations on `Box<dyn Fn>`. The `Clone` bound on
/// `unit` is required because the right adjoint may need to reproduce the
/// value (e.g., inside a reader function).
pub trait Adjunction<F: HKT, U: HKT> {
    /// unit: A -> U(F(A))
    fn unit<A: Clone + 'static>(a: A) -> U::Of<F::Of<A>>;

    /// counit: F(U(B)) -> B
    fn counit<B: 'static>(fub: F::Of<U::Of<B>>) -> B;
}

/// left_adjunct: (F(A) -> B) -> (A -> U(B))
///
/// Derived from unit: `left_adjunct(f, a) = U::fmap(unit(a), f)`
///
/// Requires `U: Functor` — works for adjunctions where the right adjoint
/// has a Functor implementation.
pub fn left_adjunct<Adj, F, U, A, B>(f: impl Fn(F::Of<A>) -> B, a: A) -> U::Of<B>
where
    F: HKT,
    U: Functor,
    A: Clone + 'static,
    Adj: Adjunction<F, U>,
{
    U::fmap(Adj::unit(a), f)
}

/// right_adjunct: (A -> U(B)) -> (F(A) -> B)
///
/// Derived from counit: `right_adjunct(f, fa) = counit(F::fmap(fa, f))`
///
/// Requires `F: Functor` — works for adjunctions where the left adjoint
/// has a Functor implementation.
pub fn right_adjunct<Adj, F, U, A, B>(f: impl Fn(A) -> U::Of<B>, fa: F::Of<A>) -> B
where
    F: Functor,
    U: HKT,
    B: 'static,
    Adj: Adjunction<F, U>,
{
    Adj::counit(F::fmap(fa, f))
}

// ---------------------------------------------------------------------------
// Trivial adjunction: IdentityF ⊣ IdentityF
// ---------------------------------------------------------------------------

/// Witness for the trivial adjunction `IdentityF ⊣ IdentityF`.
///
/// unit and counit are both the identity function.
pub struct IdentityAdj;

impl Adjunction<IdentityF, IdentityF> for IdentityAdj {
    fn unit<A: Clone + 'static>(a: A) -> A {
        a
    }

    fn counit<B: 'static>(b: B) -> B {
        b
    }
}

// ---------------------------------------------------------------------------
// Product/Exponential adjunction: EnvF<E> ⊣ ReaderF<E>
// ---------------------------------------------------------------------------

/// Witness for the curry/uncurry adjunction `EnvF<E> ⊣ ReaderF<E>`.
///
/// This is the product-exponential adjunction:
/// - `unit: A -> (E -> (E, A))` — pairs a value with an environment
/// - `counit: (E, E -> B) -> B` — applies a reader to its environment
///
/// The derived monad `ReaderF<E> . EnvF<E>` is the State monad:
/// `Of<A> = E -> (E, A)`.
///
/// The derived comonad `EnvF<E> . ReaderF<E>` is the Store comonad:
/// `Of<A> = (E, E -> A)`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct CurryAdj<E>(core::marker::PhantomData<E>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E: Clone + 'static> Adjunction<EnvF<E>, crate::hkt::ReaderF<E>> for CurryAdj<E> {
    fn unit<A: Clone + 'static>(a: A) -> Box<dyn Fn(E) -> (E, A)> {
        Box::new(move |e| (e, a.clone()))
    }

    fn counit<B: 'static>(fub: (E, Box<dyn Fn(E) -> B>)) -> B {
        let (e, f) = fub;
        f(e)
    }
}

// ---------------------------------------------------------------------------
// Monad from adjunction: U . F (requires both F and U to be Functor)
// ---------------------------------------------------------------------------

/// Derive monadic `pure` from an adjunction F ⊣ U.
///
/// `pure = unit: A -> U(F(A)) = (U . F)(A)`
pub fn adjunction_pure<Adj, F, U, A>(a: A) -> U::Of<F::Of<A>>
where
    F: HKT,
    U: HKT,
    A: Clone + 'static,
    Adj: Adjunction<F, U>,
{
    Adj::unit(a)
}

#[allow(clippy::type_complexity)]
/// Derive monadic `join` from an adjunction F ⊣ U.
///
/// `join: U(F(U(F(A)))) -> U(F(A))`
///
/// Applies `counit` inside the outer U layer to collapse the inner `F(U(...))`.
///
/// Requires `U: Functor` to map counit over the outer layer.
pub fn adjunction_join<Adj, F, U, A>(ufufa: U::Of<F::Of<U::Of<F::Of<A>>>>) -> U::Of<F::Of<A>>
where
    F: HKT,
    U: Functor,
    A: 'static,
    Adj: Adjunction<F, U>,
    F::Of<A>: 'static,
    U::Of<F::Of<A>>: 'static,
    F::Of<U::Of<F::Of<A>>>: 'static,
{
    U::fmap(ufufa, |fufa: F::Of<U::Of<F::Of<A>>>| Adj::counit(fufa))
}

#[allow(clippy::type_complexity)]
/// Derive monadic `chain` (bind) from an adjunction F ⊣ U.
///
/// `chain m f = join (fmap f m)` where the monad is `U . F`.
///
/// Requires both `U` and `F` to be Functor (for ComposeF::fmap and join).
pub fn adjunction_chain<Adj, F, U, A, B>(
    ufa: U::Of<F::Of<A>>,
    f: impl Fn(A) -> U::Of<F::Of<B>> + 'static,
) -> U::Of<F::Of<B>>
where
    F: Functor,
    U: Functor,
    A: 'static,
    B: 'static,
    Adj: Adjunction<F, U>,
    F::Of<B>: 'static,
    U::Of<F::Of<B>>: 'static,
    F::Of<U::Of<F::Of<B>>>: 'static,
{
    let mapped: U::Of<F::Of<U::Of<F::Of<B>>>> = ComposeF::<U, F>::fmap(ufa, f);
    adjunction_join::<Adj, F, U, B>(mapped)
}

// ---------------------------------------------------------------------------
// Comonad from adjunction: F . U
// ---------------------------------------------------------------------------

/// Derive comonadic `extract` from an adjunction F ⊣ U.
///
/// `extract = counit: F(U(A)) -> A`
pub fn adjunction_extract<Adj, F, U, A>(fua: F::Of<U::Of<A>>) -> A
where
    F: HKT,
    U: HKT,
    A: 'static,
    Adj: Adjunction<F, U>,
{
    Adj::counit(fua)
}

#[allow(clippy::type_complexity)]
/// Derive comonadic `duplicate` from an adjunction F ⊣ U.
///
/// `duplicate: F(U(A)) -> F(U(F(U(A))))`
///
/// Applies `unit` inside the outer F layer to expand `U(A)` into `U(F(U(A)))`.
///
/// Requires `F: Functor` to map unit over the outer layer.
pub fn adjunction_duplicate<Adj, F, U, A>(fua: F::Of<U::Of<A>>) -> F::Of<U::Of<F::Of<U::Of<A>>>>
where
    F: Functor,
    U: HKT,
    A: 'static,
    Adj: Adjunction<F, U>,
    U::Of<A>: Clone + 'static,
{
    F::fmap(fua, |ua: U::Of<A>| Adj::unit(ua))
}

/// Derive comonadic `extend` from an adjunction F ⊣ U.
///
/// `extend f = fmap f . duplicate`
///
/// Requires `F: Functor` (for both duplicate and the outer fmap).
pub fn adjunction_extend<Adj, F, U, A, B>(
    fua: F::Of<U::Of<A>>,
    f: impl Fn(F::Of<U::Of<A>>) -> B + 'static,
) -> F::Of<U::Of<B>>
where
    F: Functor,
    U: Functor,
    A: 'static,
    B: Clone + 'static,
    Adj: Adjunction<F, U>,
    U::Of<A>: Clone + 'static,
    F::Of<U::Of<A>>: 'static,
{
    let duplicated = adjunction_duplicate::<Adj, F, U, A>(fua);
    ComposeF::<F, U>::fmap(duplicated, f)
}

// ---------------------------------------------------------------------------
// CurryAdj-specific helpers (ReaderF can't implement Functor)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// CurryAdj-specific adjunct helpers (using ReaderF's inherent methods)
// ---------------------------------------------------------------------------

/// CurryAdj left_adjunct: `((E, A) -> B) -> (A -> (E -> B))`
///
/// Curries a function that takes a pair into a function returning a reader.
/// Uses `ReaderF::fmap` internally (the Lan workaround for `'static`).
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn curry_left_adjunct<E: Clone + 'static, A: Clone + 'static, B: 'static>(
    f: impl Fn((E, A)) -> B + 'static,
    a: A,
) -> Box<dyn Fn(E) -> B> {
    crate::hkt::ReaderF::<E>::fmap(CurryAdj::<E>::unit(a), f)
}

/// CurryAdj right_adjunct: `(A -> (E -> B)) -> ((E, A) -> B)`
///
/// Uncurries a function returning a reader into one that takes a pair.
/// Uses `EnvF::fmap` (which implements `Functor`) + `counit`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn curry_right_adjunct<E: Clone + 'static, A: 'static, B: 'static>(
    f: impl Fn(A) -> Box<dyn Fn(E) -> B> + 'static,
    pair: (E, A),
) -> B {
    CurryAdj::<E>::counit(<EnvF<E> as Functor>::fmap(pair, f))
}

// ---------------------------------------------------------------------------
// State monad (derived from CurryAdj: ReaderF<E> . EnvF<E>)
// ---------------------------------------------------------------------------

/// State monad `pure` via CurryAdj: `A -> (E -> (E, A))`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_pure<E: Clone + 'static, A: Clone + 'static>(a: A) -> Box<dyn Fn(E) -> (E, A)> {
    CurryAdj::<E>::unit(a)
}

/// State monad `fmap` via CurryAdj: post-compose over the value component.
///
/// `state_fmap(f, sa) = |e| let (e', a) = sa(e) in (e', f(a))`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_fmap<E: Clone + 'static, A: 'static, B: 'static>(
    f: impl Fn(A) -> B + 'static,
    sa: Box<dyn Fn(E) -> (E, A)>,
) -> Box<dyn Fn(E) -> (E, B)> {
    crate::hkt::ReaderF::<E>::fmap(sa, move |(e, a)| (e, f(a)))
}

/// State monad `chain` (bind) via CurryAdj:
/// `(E -> (E, A)) -> (A -> (E -> (E, B))) -> (E -> (E, B))`
///
/// Threads the modified state: `|e| let (e', a) = ma(e) in f(a)(e')`
///
/// Note: this is NOT `ReaderF::chain` — Reader's bind passes the same
/// environment to both, while State threads modified state through.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_chain<E: Clone + 'static, A: 'static, B: 'static>(
    ma: Box<dyn Fn(E) -> (E, A)>,
    f: impl Fn(A) -> Box<dyn Fn(E) -> (E, B)> + 'static,
) -> Box<dyn Fn(E) -> (E, B)> {
    Box::new(move |e| {
        let (e2, a) = ma(e);
        f(a)(e2)
    })
}

/// State monad `get`: `E -> (E, E)`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_get<E: Clone + 'static>() -> Box<dyn Fn(E) -> (E, E)> {
    Box::new(|e: E| {
        let e2 = e.clone();
        (e, e2)
    })
}

/// State monad `put`: `E -> (E -> (E, ()))`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_put<E: Clone + 'static>(e: E) -> Box<dyn Fn(E) -> (E, ())> {
    Box::new(move |_| (e.clone(), ()))
}

/// State monad `modify`: `(E -> E) -> (E -> (E, ()))`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn state_modify<E: Clone + 'static>(f: impl Fn(E) -> E + 'static) -> Box<dyn Fn(E) -> (E, ())> {
    Box::new(move |e| (f(e), ()))
}

// ---------------------------------------------------------------------------
// Store comonad (derived from CurryAdj: EnvF<E> . ReaderF<E>)
// ---------------------------------------------------------------------------

/// Store comonad `extract` via CurryAdj: `(E, E -> A) -> A`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn store_extract<E: Clone + 'static, A: 'static>(store: (E, Box<dyn Fn(E) -> A>)) -> A {
    CurryAdj::<E>::counit(store)
}

/// Store comonad `extend` via CurryAdj: maps over all positions.
///
/// `extend(f, (pos, peek)) = (pos, |e| f((e, peek)))`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn store_extend<E: Clone + 'static, A: 'static, B: 'static>(
    store: (E, Box<dyn Fn(E) -> A>),
    f: impl Fn((E, &dyn Fn(E) -> A)) -> B + 'static,
) -> (E, Box<dyn Fn(E) -> B>) {
    let pos = store.0.clone();
    let peek = store.1;
    let new_peek: Box<dyn Fn(E) -> B> = Box::new(move |e| f((e, peek.as_ref())));
    (pos, new_peek)
}

/// Store comonad `peek`: read at a specific position.
///
/// `peek(pos, (_, f)) = f(pos)`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn store_peek<E: Clone + 'static, A: 'static>(pos: E, store: &(E, Box<dyn Fn(E) -> A>)) -> A {
    (store.1)(pos)
}

/// Store comonad `pos`: get the current position.
///
/// `pos((e, _)) = e`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn store_pos<E: Clone, A>(store: &(E, Box<dyn Fn(E) -> A>)) -> E {
    store.0.clone()
}

// ===========================================================================
// Contravariant Adjunction: Op_R ⊣ Op_R (self-adjoint)
// ===========================================================================

/// A contravariant functor type constructor: `Of<A> = Box<dyn Fn(A) -> R>`.
///
/// This is the "Op" or "Cont" functor, generalizing `PredicateF` (which is `ContF<bool>`).
/// It is contravariant: given `f: B -> A`, we get `ContF<R>::Of<A> -> ContF<R>::Of<B>`
/// by pre-composing with `f`.
///
/// `ContF<R>` is self-adjoint as a contravariant functor, giving rise to the
/// continuation monad `(A -> R) -> R` via `ContF<R> . ContF<R>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct ContF<R>(core::marker::PhantomData<R>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<R: 'static> HKT for ContF<R> {
    type Of<T> = Box<dyn Fn(T) -> R>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<R: 'static> crate::contravariant::Contravariant for ContF<R> {
    fn contramap<A: 'static, B>(fa: Self::Of<A>, f: impl Fn(B) -> A + 'static) -> Self::Of<B> {
        Box::new(move |b| fa(f(b)))
    }
}

/// An adjunction between contravariant functors F and G.
///
/// For contravariant functors, the adjunction is:
///
/// ```text
/// Hom(F(A), B) ≅ Hom(G(B), A)
/// ```
///
/// expressed via unit/counit that reverse variance:
/// - `unit: A -> G(F(A))` — note G(F(A)) is covariant (two contravariants compose to covariant)
/// - `counit: B -> F(G(B))` — same structure
///
/// The primary instance is the self-adjunction `ContF<R> ⊣ ContF<R>`,
/// where `unit(a) = |k| k(a)` and `counit(b) = |k| k(b)` are the same operation.
/// This gives rise to the continuation monad `(A -> R) -> R`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub trait ContravariantAdjunction<F: HKT, G: HKT> {
    /// unit: A -> G(F(A))
    ///
    /// For the `ContF<R>` self-adjunction: `unit(a) = |k| k(a)`
    fn unit<A: Clone + 'static>(a: A) -> G::Of<F::Of<A>>;

    /// counit: B -> F(G(B))
    ///
    /// For the `ContF<R>` self-adjunction: `counit(b) = |k| k(b)`
    fn counit<B: Clone + 'static>(b: B) -> F::Of<G::Of<B>>;
}

/// Witness for the self-adjunction `ContF<R> ⊣ ContF<R>`.
///
/// This is the canonical contravariant adjunction. The composed functor
/// `ContF<R> . ContF<R>` gives `Of<A> = (A -> R) -> R`, the continuation monad.
///
/// - `unit(a) = |k| k(a)` (embed a value into CPS)
/// - `counit(b) = |k| k(b)` (same operation — self-adjoint!)
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct ContAdj<R>(core::marker::PhantomData<R>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<R: 'static> ContravariantAdjunction<ContF<R>, ContF<R>> for ContAdj<R> {
    fn unit<A: Clone + 'static>(a: A) -> Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R> {
        Box::new(move |k: Box<dyn Fn(A) -> R>| k(a.clone()))
    }

    fn counit<B: Clone + 'static>(b: B) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R> {
        Box::new(move |k: Box<dyn Fn(B) -> R>| k(b.clone()))
    }
}

/// Continuation monad `pure`: embed a value into CPS.
///
/// `cont_pure(a) = |k| k(a)`
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn cont_pure<R: 'static, A: Clone + 'static>(a: A) -> Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R> {
    ContAdj::<R>::unit(a)
}

/// Continuation monad `fmap`: post-compose inside CPS.
///
/// `cont_fmap(f, m) = |k| m(|a| k(f(a)))`
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn cont_fmap<R: 'static, A: 'static, B: 'static>(
    f: impl Fn(A) -> B + 'static,
    m: Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R>,
) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R> {
    let f_rc = Rc::new(f);
    Box::new(move |k: Box<dyn Fn(B) -> R>| {
        let f_inner = f_rc.clone();
        let k_rc = Rc::new(k);
        let k_composed: Box<dyn Fn(A) -> R> = Box::new(move |a| k_rc(f_inner(a)));
        m(k_composed)
    })
}

/// Continuation monad `chain` (bind): sequence CPS computations.
///
/// `cont_chain(m, f) = |k| m(|a| f(a)(k))`
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn cont_chain<R: 'static, A: 'static, B: 'static>(
    m: Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R>,
    f: impl Fn(A) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R> + 'static,
) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R> {
    let f_rc = Rc::new(f);
    Box::new(move |k: Box<dyn Fn(B) -> R>| {
        let k_rc = Rc::new(k);
        let k_inner = k_rc.clone();
        let f_inner = f_rc.clone();
        let inner: Box<dyn Fn(A) -> R> = Box::new(move |a| {
            let cont_b = f_inner(a);
            let k_ref = k_inner.clone();
            let k_box: Box<dyn Fn(B) -> R> = Box::new(move |b| k_ref(b));
            cont_b(k_box)
        });
        m(inner)
    })
}

/// Continuation monad `call_cc`: call-with-current-continuation.
///
/// `call_cc(f) = |k| f(|a| |_| k(a))(k)`
///
/// The escape continuation `|a| |_| k(a)` ignores its own continuation
/// and jumps directly to `k`.
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn cont_call_cc<R: 'static, A: Clone + 'static, B: 'static>(
    f: impl Fn(
        Box<dyn Fn(A) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R>>,
    ) -> Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R>
    + 'static,
) -> Box<dyn Fn(Box<dyn Fn(A) -> R>) -> R> {
    Box::new(move |k: Box<dyn Fn(A) -> R>| {
        let k_rc = Rc::new(k);
        let k_for_escape = k_rc.clone();
        let escape: Box<dyn Fn(A) -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R>> =
            Box::new(move |a: A| -> Box<dyn Fn(Box<dyn Fn(B) -> R>) -> R> {
                let k_esc = k_for_escape.clone();
                Box::new(move |_: Box<dyn Fn(B) -> R>| k_esc(a.clone()))
            });
        let k_box: Box<dyn Fn(A) -> R> = Box::new(move |a| k_rc(a));
        f(escape)(k_box)
    })
}

/// Run a continuation computation by supplying the final continuation.
///
/// `cont_run(m, k) = m(k)`
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn cont_run<R, A>(m: &dyn Fn(Box<dyn Fn(A) -> R>) -> R, k: impl Fn(A) -> R + 'static) -> R {
    m(Box::new(k))
}

// ===========================================================================
// Profunctor Adjunction
// ===========================================================================

/// A type-level functor on the category of profunctors.
///
/// Maps a profunctor `P` to another profunctor `Applied<P>`.
/// This serves as an HKT3-like encoding using GATs: the "third parameter"
/// is the profunctor `P` itself.
///
/// Instances include identity (maps P to P) and profunctor transformers
/// like Tambara, Pastro, etc.
pub trait ProfunctorFunctor {
    /// Apply this functor to a profunctor, yielding another profunctor.
    type Applied<P: HKT2>: HKT2;
}

/// An adjunction F ⊣ U in the category of profunctors.
///
/// ```text
/// ProfHom(F(P), Q) ≅ ProfHom(P, U(Q))
/// ```
///
/// Expressed via unit/counit natural transformations between profunctors:
/// - `unit: P<A,B> -> U(F(P))<A,B>` for all profunctors P
/// - `counit: F(U(Q))<A,B> -> Q<A,B>` for all profunctors Q
///
/// The primary instance is the identity adjunction `Id ⊣ Id`.
/// Non-trivial instances (e.g., `Pastro ⊣ Tambara`) require profunctor
/// transformer types.
pub trait ProfunctorAdjunction<F: ProfunctorFunctor, U: ProfunctorFunctor> {
    /// unit: P<A,B> -> U(F(P))<A,B>
    fn unit<P: HKT2, A: 'static, B: 'static>(
        pab: P::P<A, B>,
    ) -> <U::Applied<F::Applied<P>> as HKT2>::P<A, B>;

    /// counit: F(U(Q))<A,B> -> Q<A,B>
    fn counit<Q: HKT2, A: 'static, B: 'static>(
        fuqab: <F::Applied<U::Applied<Q>> as HKT2>::P<A, B>,
    ) -> Q::P<A, B>;
}

/// The identity profunctor functor: maps P to P.
pub struct ProfunctorIdentityF;

impl ProfunctorFunctor for ProfunctorIdentityF {
    type Applied<P: HKT2> = P;
}

/// Witness for the identity profunctor adjunction: Id ⊣ Id.
///
/// Both unit and counit are the identity: `P<A,B> -> P<A,B>`.
pub struct ProfunctorIdentityAdj;

impl ProfunctorAdjunction<ProfunctorIdentityF, ProfunctorIdentityF> for ProfunctorIdentityAdj {
    fn unit<P: HKT2, A: 'static, B: 'static>(pab: P::P<A, B>) -> P::P<A, B> {
        pab
    }

    fn counit<Q: HKT2, A: 'static, B: 'static>(fuqab: Q::P<A, B>) -> Q::P<A, B> {
        fuqab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- IdentityAdj tests ---

    #[test]
    fn identity_adj_unit() {
        let result = IdentityAdj::unit(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn identity_adj_counit() {
        let result = IdentityAdj::counit(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn identity_adj_left_adjunct() {
        let f = |x: i32| x + 1;
        let result = left_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(f, 5);
        assert_eq!(result, 6);
    }

    #[test]
    fn identity_adj_right_adjunct() {
        let f = |x: i32| x * 2;
        let result = right_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(f, 5);
        assert_eq!(result, 10);
    }

    #[test]
    fn identity_adj_monad_pure() {
        let result = adjunction_pure::<IdentityAdj, IdentityF, IdentityF, i32>(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn identity_adj_monad_join() {
        let result = adjunction_join::<IdentityAdj, IdentityF, IdentityF, i32>(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn identity_adj_monad_chain() {
        let result = adjunction_chain::<IdentityAdj, IdentityF, IdentityF, i32, i32>(5, |x| x + 1);
        assert_eq!(result, 6);
    }

    #[test]
    fn identity_adj_comonad_extract() {
        let result = adjunction_extract::<IdentityAdj, IdentityF, IdentityF, i32>(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn identity_adj_comonad_duplicate() {
        let result = adjunction_duplicate::<IdentityAdj, IdentityF, IdentityF, i32>(42);
        assert_eq!(result, 42);
    }

    // --- CurryAdj tests ---

    #[cfg(any(feature = "std", feature = "alloc"))]
    mod curry_adj_tests {
        use super::*;

        #[test]
        fn curry_adj_unit() {
            let reader = CurryAdj::<i32>::unit(42i32);
            assert_eq!(reader(10), (10, 42));
            assert_eq!(reader(0), (0, 42));
        }

        #[test]
        fn curry_adj_counit() {
            let env_reader: (i32, Box<dyn Fn(i32) -> String>) =
                (5, Box::new(|e| format!("env={}", e)));
            let result = CurryAdj::<i32>::counit(env_reader);
            assert_eq!(result, "env=5");
        }

        #[test]
        fn curry_adj_monad_pure() {
            let state_fn = state_pure::<i32, String>("hello".to_string());
            assert_eq!(state_fn(42), (42, "hello".to_string()));
        }

        #[test]
        fn curry_adj_state_chain() {
            // State monad: increment the state and return the old value
            let get_and_inc = state_chain(
                state_get::<i32>(),
                |old: i32| -> Box<dyn Fn(i32) -> (i32, i32)> { Box::new(move |e| (e + 1, old)) },
            );
            assert_eq!(get_and_inc(10), (11, 10));
            assert_eq!(get_and_inc(0), (1, 0));
        }

        #[test]
        fn curry_adj_state_get() {
            let getter = state_get::<i32>();
            assert_eq!(getter(42), (42, 42));
        }

        #[test]
        fn curry_adj_state_put() {
            let putter = state_put(99i32);
            assert_eq!(putter(0), (99, ()));
            assert_eq!(putter(42), (99, ()));
        }

        #[test]
        fn curry_adj_store_extract() {
            let store: (i32, Box<dyn Fn(i32) -> String>) = (5, Box::new(|e| format!("val={}", e)));
            let result = store_extract(store);
            assert_eq!(result, "val=5");
        }

        #[test]
        fn curry_adj_store_extend() {
            let store: (i32, Box<dyn Fn(i32) -> i32>) = (3, Box::new(|e| e * 2));
            let extended = store_extend(store, |(pos, peek)| peek(pos) + 1);
            assert_eq!(extended.0, 3); // position unchanged
            assert_eq!((extended.1)(3), 7); // peek(3) + 1 = 6 + 1
            assert_eq!((extended.1)(5), 11); // peek(5) + 1 = 10 + 1
        }

        #[test]
        fn curry_adj_state_monad_left_identity() {
            // chain(pure(a), f) == f(a)
            let a = 42i32;
            let f = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> { Box::new(move |e| (e, x + 1)) };
            let chained = state_chain(state_pure(a), f);
            let direct = f(a);
            for e in [0, 1, 10, -5] {
                assert_eq!(chained(e), direct(e));
            }
        }

        #[test]
        fn curry_adj_state_monad_right_identity() {
            // chain(m, pure) == m
            let m_fn = |e: i32| (e + 1, e * 2);
            let chained = state_chain(
                Box::new(move |e: i32| (e + 1, e * 2)) as Box<dyn Fn(i32) -> (i32, i32)>,
                |a| state_pure(a),
            );
            for e in [0, 1, 10, -5] {
                assert_eq!(chained(e), m_fn(e));
            }
        }

        // --- ReaderF inherent methods ---

        #[test]
        fn reader_fmap() {
            let reader: Box<dyn Fn(i32) -> i32> = Box::new(|e| e * 2);
            let mapped = crate::hkt::ReaderF::<i32>::fmap(reader, |x| x + 1);
            assert_eq!(mapped(5), 11); // (5 * 2) + 1
        }

        #[test]
        fn reader_pure() {
            let reader = crate::hkt::ReaderF::<i32>::pure(42);
            assert_eq!(reader(0), 42);
            assert_eq!(reader(999), 42);
        }

        #[test]
        fn reader_chain() {
            let reader: Box<dyn Fn(i32) -> i32> = Box::new(|e| e + 1);
            let chained = crate::hkt::ReaderF::<i32>::chain(reader, |a| {
                Box::new(move |e| a * e) as Box<dyn Fn(i32) -> i32>
            });
            // reader(5) = 6, then |e| 6 * e applied to 5 = 30
            assert_eq!(chained(5), 30);
        }

        #[test]
        fn reader_ask() {
            let reader = crate::hkt::ReaderF::<String>::ask();
            assert_eq!(reader("hello".to_string()), "hello".to_string());
        }

        #[test]
        fn reader_local() {
            let reader: Box<dyn Fn(i32) -> i32> = Box::new(|e| e * 2);
            let localized = crate::hkt::ReaderF::<i32>::local(|e| e + 10, reader);
            assert_eq!(localized(5), 30); // (5 + 10) * 2
        }

        // --- CurryAdj adjunct helpers ---

        #[test]
        fn curry_left_adjunct_test() {
            let f = |pair: (i32, i32)| pair.0 + pair.1;
            let reader = curry_left_adjunct(f, 10i32);
            assert_eq!(reader(5), 15); // 5 + 10
            assert_eq!(reader(3), 13); // 3 + 10
        }

        #[test]
        fn curry_right_adjunct_test() {
            let f = |a: i32| -> Box<dyn Fn(i32) -> i32> { Box::new(move |e| e * a) };
            let result = curry_right_adjunct(f, (3i32, 5i32));
            assert_eq!(result, 15); // 3 * 5
        }

        // --- State fmap and modify ---

        #[test]
        fn state_fmap_test() {
            let sa: Box<dyn Fn(i32) -> (i32, i32)> = Box::new(|e| (e + 1, e * 2));
            let mapped = state_fmap(|x| x + 100, sa);
            assert_eq!(mapped(5), (6, 110)); // state=6, value=(5*2)+100
        }

        #[test]
        fn state_modify_test() {
            let modifier = state_modify(|e: i32| e + 10);
            assert_eq!(modifier(5), (15, ()));
        }

        // --- Store peek and pos ---

        #[test]
        fn store_peek_test() {
            let store: (i32, Box<dyn Fn(i32) -> String>) = (3, Box::new(|e| format!("v{}", e)));
            assert_eq!(store_peek(3, &store), "v3");
            assert_eq!(store_peek(7, &store), "v7");
        }

        #[test]
        fn store_pos_test() {
            let store: (i32, Box<dyn Fn(i32) -> i32>) = (42, Box::new(|e| e * 2));
            assert_eq!(store_pos(&store), 42);
        }
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Triangle identity 1 (for IdentityAdj):
        // right_adjunct(unit, fa) == fa
        #[test]
        fn identity_adj_triangle_1(x in any::<i32>()) {
            let result = right_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(
                IdentityAdj::unit, x,
            );
            prop_assert_eq!(result, x);
        }

        // Triangle identity 2 (for IdentityAdj):
        // left_adjunct(counit, a) == a
        #[test]
        fn identity_adj_triangle_2(x in any::<i32>()) {
            let result = left_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(
                IdentityAdj::counit, x,
            );
            prop_assert_eq!(result, x);
        }

        // left_adjunct and right_adjunct are inverses (IdentityAdj)
        #[test]
        fn identity_adj_adjuncts_inverse(x in any::<i16>()) {
            let f = |a: i16| a.wrapping_add(1);
            let la = left_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(f, x);
            prop_assert_eq!(la, f(x));

            let g = |a: i16| a.wrapping_mul(2);
            let ra = right_adjunct::<IdentityAdj, IdentityF, IdentityF, _, _>(g, x);
            prop_assert_eq!(ra, g(x));
        }

        // Monad laws for IdentityAdj (U.F = IdentityF):
        // Left identity: chain(pure(a), f) == f(a)
        #[test]
        fn identity_adj_monad_left_identity(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let pure_x = adjunction_pure::<IdentityAdj, IdentityF, IdentityF, i32>(x);
            let result = adjunction_chain::<IdentityAdj, IdentityF, IdentityF, i32, i32>(
                pure_x, f,
            );
            prop_assert_eq!(result, f(x));
        }

        // Right identity: chain(m, pure) == m
        #[test]
        fn identity_adj_monad_right_identity(x in any::<i32>()) {
            let result = adjunction_chain::<IdentityAdj, IdentityF, IdentityF, i32, i32>(
                x,
                adjunction_pure::<IdentityAdj, IdentityF, IdentityF, i32>,
            );
            prop_assert_eq!(result, x);
        }

        // Comonad laws for IdentityAdj (F.U = IdentityF):
        // extract(duplicate(w)) == w
        #[test]
        fn identity_adj_comonad_extract_duplicate(x in any::<i32>()) {
            let dup = adjunction_duplicate::<IdentityAdj, IdentityF, IdentityF, i32>(x);
            let result = adjunction_extract::<IdentityAdj, IdentityF, IdentityF, i32>(dup);
            prop_assert_eq!(result, x);
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    mod curry_adj_law_tests {
        use super::*;

        proptest! {
            // Triangle identity for CurryAdj:
            // counit(EnvF::fmap((e, a), unit)) == (e, a)
            #[test]
            fn curry_adj_triangle_counit_fmap_unit(e in -100i32..100, a in -100i32..100) {
                let mapped = <EnvF<i32> as crate::functor::Functor>::fmap(
                    (e, a),
                    |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                        CurryAdj::<i32>::unit(x)
                    },
                );
                let result = CurryAdj::<i32>::counit(mapped);
                prop_assert_eq!(result, (e, a));
            }

            // State monad left identity: chain(pure(a), f) == f(a)
            #[test]
            fn curry_adj_state_left_identity(a in -100i32..100, e in -100i32..100) {
                let f = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                    Box::new(move |env| (env, x.wrapping_add(1)))
                };
                let chained = state_chain(state_pure(a), f);
                let expected = f(a);
                prop_assert_eq!(chained(e), expected(e));
            }

            // State monad right identity: chain(m, pure) == m
            #[test]
            fn curry_adj_state_right_identity(e in -100i32..100) {
                let m_fn = move |env: i32| (env.wrapping_add(1), env.wrapping_mul(2));
                let chained = state_chain(
                    Box::new(m_fn) as Box<dyn Fn(i32) -> (i32, i32)>,
                    |a: i32| state_pure(a),
                );
                prop_assert_eq!(chained(e), m_fn(e));
            }

            // State monad associativity: chain(chain(m, f), g) == chain(m, |a| chain(f(a), g))
            #[test]
            fn curry_adj_state_associativity(e in -100i32..100) {
                let _m: Box<dyn Fn(i32) -> (i32, i32)> =
                    Box::new(|env| (env, 10));
                let f = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                    Box::new(move |env| (env.wrapping_add(1), x.wrapping_add(1)))
                };
                let g = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                    Box::new(move |env| (env, x.wrapping_mul(2)))
                };

                let left = state_chain(state_chain(
                    Box::new(|env: i32| (env, 10)) as Box<dyn Fn(i32) -> (i32, i32)>,
                    f,
                ), g);
                let right = state_chain(
                    Box::new(|env: i32| (env, 10)) as Box<dyn Fn(i32) -> (i32, i32)>,
                    move |a: i32| {
                        let inner_f = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                            Box::new(move |env| (env.wrapping_add(1), x.wrapping_add(1)))
                        };
                        let inner_g = |x: i32| -> Box<dyn Fn(i32) -> (i32, i32)> {
                            Box::new(move |env| (env, x.wrapping_mul(2)))
                        };
                        state_chain(inner_f(a), inner_g)
                    },
                );
                prop_assert_eq!(left(e), right(e));
            }

            // Store comonad: extract(store) == peek(pos)
            #[test]
            fn curry_adj_store_extract_law(pos in -100i32..100) {
                let store: (i32, Box<dyn Fn(i32) -> i32>) =
                    (pos, Box::new(|e| e.wrapping_mul(2)));
                let result = store_extract(store);
                prop_assert_eq!(result, pos.wrapping_mul(2));
            }
        }
    }

    // --- ContravariantAdjunction tests ---

    #[cfg(any(feature = "std", feature = "alloc"))]
    mod contravariant_adj_tests {
        use super::*;

        #[test]
        fn cont_adj_unit() {
            let k: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
            let m = ContAdj::<i32>::unit(42);
            assert_eq!(m(k), 43);
        }

        #[test]
        fn cont_adj_counit() {
            let k: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
            let m = ContAdj::<i32>::counit(10);
            assert_eq!(m(k), 20);
        }

        #[test]
        fn cont_adj_self_adjoint() {
            // unit and counit should behave identically (self-adjoint)
            let k: Box<dyn Fn(i32) -> String> = Box::new(|x| format!("{}", x));
            let unit_result = ContAdj::<String>::unit(42);
            let counit_result = ContAdj::<String>::counit(42);
            let k2: Box<dyn Fn(i32) -> String> = Box::new(|x| format!("{}", x));
            assert_eq!(unit_result(k), counit_result(k2));
        }

        #[test]
        fn cont_pure_test() {
            let m = cont_pure::<i32, _>(42);
            assert_eq!(cont_run(&*m, |x| x + 1), 43);
        }

        #[test]
        fn cont_fmap_test() {
            let m = cont_pure::<i32, _>(10);
            let mapped = cont_fmap(|x: i32| x * 3, m);
            assert_eq!(cont_run(&*mapped, |x| x + 1), 31); // (10 * 3) + 1
        }

        #[test]
        fn cont_chain_test() {
            let m = cont_pure::<i32, _>(5);
            let chained = cont_chain(m, |x: i32| cont_pure(x + 10));
            assert_eq!(cont_run(&*chained, |x| x * 2), 30); // (5 + 10) * 2
        }

        #[test]
        fn cont_chain_sequencing() {
            // chain two computations that modify values
            let m1 = cont_pure::<i32, _>(3);
            let m2 = cont_chain(m1, |x| {
                let doubled = x * 2; // 6
                cont_chain(cont_pure(doubled), |y| cont_pure(y + 1)) // 7
            });
            assert_eq!(cont_run(&*m2, |x| x), 7);
        }

        #[test]
        fn cont_call_cc_no_escape() {
            // call_cc where we don't use the escape continuation
            let m = cont_call_cc::<i32, i32, i32>(|_escape| cont_pure(42));
            assert_eq!(cont_run(&*m, |x| x), 42);
        }

        #[test]
        fn cont_call_cc_with_escape() {
            // call_cc where we use the escape continuation to short-circuit
            let m = cont_call_cc::<i32, i32, i32>(|escape| {
                // escape(10) ignores the rest and returns 10
                let escaped = escape(10);
                // This chain would normally add 100, but escape skips it
                cont_chain(escaped, |_| cont_pure(999))
            });
            assert_eq!(cont_run(&*m, |x| x), 10);
        }

        #[test]
        fn contf_contramap() {
            use crate::contravariant::Contravariant;
            let f: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
            let g = ContF::<bool>::contramap(f, |s: &str| s.len() as i32);
            assert!(g("hello"));
            assert!(!g(""));
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    mod contravariant_adj_law_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            // Continuation monad left identity: chain(pure(a), f) == f(a)
            #[test]
            fn cont_monad_left_identity(a in -100i32..100) {
                let f = |x: i32| -> Box<dyn Fn(Box<dyn Fn(i32) -> i32>) -> i32> {
                    cont_pure(x.wrapping_add(1))
                };
                let chained = cont_chain(cont_pure(a), f);
                let expected = f(a);
                // Test with identity continuation
                prop_assert_eq!(
                    cont_run(&*chained, |x| x),
                    cont_run(&*expected, |x| x)
                );
            }

            // Continuation monad right identity: chain(m, pure) == m
            #[test]
            fn cont_monad_right_identity(a in -100i32..100) {
                let m = cont_pure::<i32, _>(a);
                let chained = cont_chain(m, |x: i32| cont_pure(x));
                let m2 = cont_pure::<i32, _>(a);
                prop_assert_eq!(
                    cont_run(&*chained, |x| x),
                    cont_run(&*m2, |x| x)
                );
            }

            // Functor identity: fmap(id, m) ≡ m
            #[test]
            fn cont_functor_identity(a in -100i32..100) {
                let m = cont_pure::<i32, _>(a);
                let mapped = cont_fmap(|x: i32| x, m);
                let m2 = cont_pure::<i32, _>(a);
                prop_assert_eq!(
                    cont_run(&*mapped, |x| x),
                    cont_run(&*m2, |x| x)
                );
            }

            // Functor composition: fmap(g . f) ≡ fmap(g) . fmap(f)
            #[test]
            fn cont_functor_composition(a in -100i32..100) {
                let f = |x: i32| x.wrapping_add(1);
                let g = |x: i32| x.wrapping_mul(2);

                let m1 = cont_pure::<i32, _>(a);
                let left = cont_fmap(move |x| g(f(x)), m1);

                let m2 = cont_pure::<i32, _>(a);
                let right = cont_fmap(g, cont_fmap(f, m2));

                prop_assert_eq!(
                    cont_run(&*left, |x| x),
                    cont_run(&*right, |x| x)
                );
            }
        }
    }

    // --- ProfunctorAdjunction tests ---

    mod profunctor_adj_tests {
        use super::*;
        use crate::hkt::TupleF;

        #[test]
        fn profunctor_identity_adj_unit() {
            let val: (i32, String) = (42, "hello".to_string());
            let result = ProfunctorIdentityAdj::unit::<TupleF, i32, String>(val.clone());
            assert_eq!(result, val);
        }

        #[test]
        fn profunctor_identity_adj_counit() {
            let val: (i32, String) = (42, "hello".to_string());
            let result = ProfunctorIdentityAdj::counit::<TupleF, i32, String>(val.clone());
            assert_eq!(result, val);
        }

        #[test]
        fn profunctor_identity_adj_roundtrip() {
            // counit(unit(p)) == p (triangle identity for identity adjunction)
            let val: (i32, i32) = (1, 2);
            let result =
                ProfunctorIdentityAdj::counit::<TupleF, i32, i32>(ProfunctorIdentityAdj::unit::<
                    TupleF,
                    i32,
                    i32,
                >(val));
            assert_eq!(result, (1, 2));
        }
    }

    mod profunctor_adj_law_tests {
        use super::*;
        use crate::hkt::TupleF;
        use proptest::prelude::*;

        proptest! {
            // unit;counit == id (triangle identity)
            #[test]
            fn profunctor_identity_adj_triangle(a in any::<i32>(), b in any::<i32>()) {
                let val: (i32, i32) = (a, b);
                let result = ProfunctorIdentityAdj::counit::<TupleF, i32, i32>(
                    ProfunctorIdentityAdj::unit::<TupleF, i32, i32>(val),
                );
                prop_assert_eq!(result, (a, b));
            }
        }
    }
}
