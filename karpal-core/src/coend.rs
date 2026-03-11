use crate::hkt::HKT2;

/// A Coend for profunctor P: represents `∫^A P(A, A)`, i.e., `exists A. P(A, A)`.
///
/// In category theory, the coend of a profunctor `P: C^op × C -> Set` is the
/// universal cowedge — a value `P(A, A)` for SOME (existentially hidden) type A.
///
/// Since Rust lacks first-class existential types, the type parameter A is
/// exposed (following the same pragmatic approach as `Lan<G, H, A, B>` in
/// karpal-free). Users should treat A as opaque when consuming coend values.
///
/// # Construction
///
/// ```rust
/// use karpal_core::coend::Coend;
/// use karpal_core::hkt::TupleF;
///
/// // A coend value: exists some A such that we have (A, A)
/// let c: Coend<TupleF, i32> = Coend::new((42, 42));
/// assert_eq!(c.value, (42, 42));
/// ```
pub struct Coend<P: HKT2, A> {
    /// The diagonal value `P(A, A)` for the existentially quantified A.
    pub value: P::P<A, A>,
}

impl<P: HKT2, A> Coend<P, A> {
    /// Construct a coend value from a diagonal element.
    pub fn new(value: P::P<A, A>) -> Self {
        Coend { value }
    }

    /// Eliminate the coend by applying a function to the diagonal value.
    ///
    /// Given a function `P(A, A) -> R`, extract a result. This is the
    /// concrete elimination form — in category theory, the eliminator
    /// would be a dinatural transformation, but here the type parameter
    /// A is exposed rather than existentially hidden.
    pub fn elim<R>(self, f: impl FnOnce(P::P<A, A>) -> R) -> R {
        f(self.value)
    }
}

impl<P: HKT2, A: Clone> Clone for Coend<P, A>
where
    P::P<A, A>: Clone,
{
    fn clone(&self) -> Self {
        Coend {
            value: self.value.clone(),
        }
    }
}

impl<P: HKT2, A: PartialEq> PartialEq for Coend<P, A>
where
    P::P<A, A>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<P: HKT2, A: Eq> Eq for Coend<P, A> where P::P<A, A>: Eq {}

impl<P: HKT2, A: core::fmt::Debug> core::fmt::Debug for Coend<P, A>
where
    P::P<A, A>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Coend").field("value", &self.value).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hkt::{ResultBF, TupleF};

    #[test]
    fn coend_tuple_new() {
        let c: Coend<TupleF, i32> = Coend::new((1, 2));
        assert_eq!(c.value, (1, 2));
    }

    #[test]
    fn coend_tuple_elim() {
        let c: Coend<TupleF, i32> = Coend::new((3, 4));
        let sum = c.elim(|(a, b)| a + b);
        assert_eq!(sum, 7);
    }

    #[test]
    fn coend_result_new() {
        let c: Coend<ResultBF, i32> = Coend::new(Ok(42));
        assert_eq!(c.value, Ok(42));
    }

    #[test]
    fn coend_clone() {
        let c: Coend<TupleF, i32> = Coend::new((1, 2));
        let c2 = c.clone();
        assert_eq!(c, c2);
    }

    #[test]
    fn coend_debug() {
        let c: Coend<TupleF, i32> = Coend::new((1, 2));
        let s = format!("{:?}", c);
        assert!(s.contains("Coend"));
        assert!(s.contains("(1, 2)"));
    }
}
