use crate::hkt::HKT2;

/// An End for profunctor P: represents `∫_A P(A, A)`, i.e., `forall A. P(A, A)`.
///
/// In category theory, the end of a profunctor `P: C^op × C -> Set` is the
/// universal wedge — a value that can produce `P(A, A)` for ANY type A.
///
/// In Rust, universal quantification over types is expressed via a generic
/// method. Concrete end values are structs implementing this trait.
///
/// # Example
///
/// The end of the function profunctor `FnP` (where `P(A, B) = Box<dyn Fn(A) -> B>`)
/// is the polymorphic identity function `forall A. A -> A`. This instance
/// lives in `karpal-profunctor` where `FnP` is defined.
///
/// # Note
///
/// The `End` trait is not dyn-compatible (object-safe) because `run` has a
/// generic type parameter. It is useful via static dispatch.
pub trait End<P: HKT2> {
    /// Extract the diagonal component for any type A.
    fn run<A: 'static>(&self) -> P::P<A, A>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hkt::TupleF;

    // A concrete end for TupleF that produces (T, T) via Clone.
    // This only works for a specific T — the universal "for all A" is
    // witnessed by the fact that the implementor can choose any T.
    struct DiagonalEnd<T: Clone>(T);

    impl<T: Clone + 'static> End<TupleF> for DiagonalEnd<T> {
        fn run<A: 'static>(&self) -> (A, A) {
            // This can only succeed when A = T. In practice, ends over
            // TupleF are degenerate — included for testing the trait.
            // The truly useful End is FnEnd in karpal-profunctor.
            //
            // Use Any downcasting to demonstrate the concept:
            use core::any::Any;
            let boxed: Box<dyn Any> = Box::new(self.0.clone());
            let val = *boxed.downcast::<A>().expect("End<TupleF> type mismatch");
            let val2 = {
                let boxed: Box<dyn Any> = Box::new(self.0.clone());
                *boxed.downcast::<A>().expect("End<TupleF> type mismatch")
            };
            (val, val2)
        }
    }

    #[test]
    fn diagonal_end_tuple() {
        let end = DiagonalEnd(42i32);
        let (a, b): (i32, i32) = end.run();
        assert_eq!(a, 42);
        assert_eq!(b, 42);
    }
}
