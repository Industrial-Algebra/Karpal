#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::comonad::Comonad;
use karpal_core::extend::Extend;
use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

/// Cofree Comonad — the dual of the Free Monad.
///
/// `Cofree<F, A>` pairs a value `A` at each node with a branching structure
/// determined by `F`. The choice of `F` determines the shape:
/// - `OptionF` → a non-empty list (finite stream)
/// - `VecF` → a rose tree
/// - `IdentityF` → an infinite stream
///
/// Each node carries a value (`head`) and subtrees (`tail`).
pub struct Cofree<F: HKT, A> {
    /// The value at this node.
    pub head: A,
    /// The subtrees, with branching structure determined by `F`.
    pub tail: Box<F::Of<Cofree<F, A>>>,
}

impl<F: HKT, A> Cofree<F, A> {
    /// Create a Cofree node with the given head and tail.
    pub fn new(head: A, tail: F::Of<Cofree<F, A>>) -> Self {
        Cofree {
            head,
            tail: Box::new(tail),
        }
    }

    /// Extract the head value.
    pub fn extract(&self) -> A
    where
        A: Clone,
    {
        self.head.clone()
    }
}

impl<F: HKT + Functor, A> Cofree<F, A> {
    /// Map a function over all head values in the tree.
    pub fn fmap<B>(self, f: impl Fn(A) -> B) -> Cofree<F, B> {
        self.fmap_inner(&f)
    }

    fn fmap_inner<B>(self, f: &dyn Fn(A) -> B) -> Cofree<F, B> {
        Cofree {
            head: f(self.head),
            tail: Box::new(F::fmap(*self.tail, |child| child.fmap_inner(f))),
        }
    }

    /// Apply a context-aware function to every position in the tree.
    ///
    /// At each node, `f` receives the entire sub-cofree rooted at that node
    /// and produces the new head value.
    pub fn extend<B>(self, f: impl Fn(&Cofree<F, A>) -> B) -> Cofree<F, B>
    where
        A: Clone,
    {
        self.extend_inner(&f)
    }

    fn extend_inner<B>(self, f: &dyn Fn(&Cofree<F, A>) -> B) -> Cofree<F, B>
    where
        A: Clone,
    {
        let b = f(&self);
        let new_tail = F::fmap(*self.tail, |child| child.extend_inner(f));
        Cofree {
            head: b,
            tail: Box::new(new_tail),
        }
    }

    /// Build a Cofree from a seed value and an unfolding function.
    ///
    /// The function takes a seed and returns `(head, F<Seed>)` — the value
    /// at this node and seeds for the subtrees.
    pub fn unfold<Seed>(seed: Seed, f: impl Fn(&Seed) -> (A, F::Of<Seed>)) -> Self {
        Self::unfold_inner(seed, &f)
    }

    #[allow(clippy::type_complexity)]
    fn unfold_inner<Seed>(seed: Seed, f: &dyn Fn(&Seed) -> (A, F::Of<Seed>)) -> Self {
        let (head, f_seeds) = f(&seed);
        let tail = F::fmap(f_seeds, |child_seed| Self::unfold_inner(child_seed, f));
        Cofree {
            head,
            tail: Box::new(tail),
        }
    }
}

/// HKT marker for `Cofree<F, _>`.
pub struct CofreeF<F: HKT>(PhantomData<F>);

impl<F: HKT> HKT for CofreeF<F> {
    type Of<T> = Cofree<F, T>;
}

impl<F: HKT + Functor> Functor for CofreeF<F> {
    fn fmap<A, B>(fa: Cofree<F, A>, f: impl Fn(A) -> B) -> Cofree<F, B> {
        fa.fmap(f)
    }
}

impl<F: HKT + Functor> Extend for CofreeF<F> {
    fn extend<A, B>(wa: Cofree<F, A>, f: impl Fn(&Cofree<F, A>) -> B) -> Cofree<F, B>
    where
        A: Clone,
    {
        wa.extend_inner(&f)
    }
}

impl<F: HKT + Functor> Comonad for CofreeF<F> {
    fn extract<A: Clone>(wa: &Cofree<F, A>) -> A {
        wa.head.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    fn make_stream(values: &[i32]) -> Cofree<OptionF, i32> {
        match values {
            [] => Cofree::new(0, None),
            [x] => Cofree::new(*x, None),
            [x, rest @ ..] => Cofree::new(*x, Some(make_stream(rest))),
        }
    }

    #[test]
    fn extract_head() {
        let cofree = make_stream(&[1, 2, 3]);
        assert_eq!(cofree.extract(), 1);
    }

    #[test]
    fn fmap_cofree() {
        let cofree = make_stream(&[1, 2, 3]);
        let mapped = cofree.fmap(|x| x * 10);
        assert_eq!(mapped.head, 10);
        let child1 = mapped.tail.as_ref().as_ref().unwrap();
        assert_eq!(child1.head, 20);
        let child2 = child1.tail.as_ref().as_ref().unwrap();
        assert_eq!(child2.head, 30);
    }

    #[test]
    fn unfold_option_stream() {
        // Unfold a countdown: 3, 2, 1, done
        let cofree = Cofree::<OptionF, i32>::unfold(3, |&seed| {
            if seed <= 0 {
                (seed, None)
            } else {
                (seed, Some(seed - 1))
            }
        });
        assert_eq!(cofree.head, 3);
        let c2 = cofree.tail.as_ref().as_ref().unwrap();
        assert_eq!(c2.head, 2);
        let c1 = c2.tail.as_ref().as_ref().unwrap();
        assert_eq!(c1.head, 1);
        let c0 = c1.tail.as_ref().as_ref().unwrap();
        assert_eq!(c0.head, 0);
        assert!(c0.tail.is_none());
    }

    #[test]
    fn unfold_then_extract() {
        let cofree = Cofree::<OptionF, i32>::unfold(42, |&seed| (seed, None));
        assert_eq!(cofree.extract(), 42);
    }

    #[test]
    fn extend_cofree() {
        // Build a stream [1, 2, 3] and extend with a function that sums the head
        // of the current node and the next node (if any)
        let cofree = make_stream(&[1, 2, 3]);
        let extended = cofree.extend(|w| {
            let next = w.tail.as_ref().as_ref().map(|c| c.head).unwrap_or(0);
            w.head + next
        });
        assert_eq!(extended.head, 1 + 2); // 3
        let c1 = extended.tail.as_ref().as_ref().unwrap();
        assert_eq!(c1.head, 2 + 3); // 5
        let c2 = c1.tail.as_ref().as_ref().unwrap();
        assert_eq!(c2.head, 3 + 0); // 3
    }

    #[test]
    fn comonad_trait_works() {
        let cofree = make_stream(&[42, 1]);
        let result = <CofreeF<OptionF> as Comonad>::extract(&cofree);
        assert_eq!(result, 42);
    }

    #[test]
    fn extend_trait_works() {
        let cofree = make_stream(&[10, 20]);
        let extended = <CofreeF<OptionF> as Extend>::extend(cofree, |w| w.head * 2);
        assert_eq!(extended.head, 20);
        let child = extended.tail.as_ref().as_ref().unwrap();
        assert_eq!(child.head, 40);
    }

    #[test]
    fn functor_trait_works() {
        let cofree = make_stream(&[5]);
        let mapped = <CofreeF<OptionF> as Functor>::fmap(cofree, |x| x + 100);
        assert_eq!(mapped.head, 105);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    fn make_singleton(v: i32) -> Cofree<OptionF, i32> {
        Cofree::new(v, None)
    }

    fn make_pair(a: i32, b: i32) -> Cofree<OptionF, i32> {
        Cofree::new(a, Some(Cofree::new(b, None)))
    }

    proptest! {
        // Comonad law: extract(extend(w, f)) == f(w)
        #[test]
        fn extract_extend(x in any::<i32>(), y in any::<i32>()) {
            let w = make_pair(x, y);
            let f = |w: &Cofree<OptionF, i32>| w.head.wrapping_mul(2);
            let expected = f(&w);
            let extended = <CofreeF<OptionF> as Extend>::extend(w, f);
            let result = <CofreeF<OptionF> as Comonad>::extract(&extended);
            prop_assert_eq!(result, expected);
        }

        // Comonad law: extend(w, extract) == w
        #[test]
        fn extend_extract(x in any::<i32>()) {
            let w = make_singleton(x);
            let result = <CofreeF<OptionF> as Extend>::extend(
                w,
                <CofreeF<OptionF> as Comonad>::extract,
            );
            prop_assert_eq!(result.head, x);
        }

        // Functor identity: fmap(id, w) == w
        #[test]
        fn functor_identity(x in any::<i32>(), y in any::<i32>()) {
            let w = make_pair(x, y);
            let result = <CofreeF<OptionF> as Functor>::fmap(w, |a| a);
            prop_assert_eq!(result.head, x);
            prop_assert_eq!(result.tail.as_ref().as_ref().unwrap().head, y);
        }

        // Functor composition: fmap(g . f, w) == fmap(g, fmap(f, w))
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = <CofreeF<OptionF> as Functor>::fmap(make_singleton(x), |a| g(f(a)));
            let right = <CofreeF<OptionF> as Functor>::fmap(
                <CofreeF<OptionF> as Functor>::fmap(make_singleton(x), f),
                g,
            );
            prop_assert_eq!(left.head, right.head);
        }
    }
}
