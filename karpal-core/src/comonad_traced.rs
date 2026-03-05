use crate::hkt::HKT;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::TracedF;
use crate::monoid::Monoid;

/// ComonadTraced: a comonad with a monoidal trace/accumulator.
///
/// Laws:
/// - `trace(M::empty(), wa) == extract(wa)` (tracing with identity is extract)
pub trait ComonadTraced<M: Monoid>: HKT {
    fn trace<A>(m: M, wa: &Self::Of<A>) -> A;

    /// Extract the focused value (equivalent to `trace(M::empty(), wa)`).
    fn extract<A>(wa: &Self::Of<A>) -> A {
        Self::trace(M::empty(), wa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<M: Monoid + Clone + 'static> ComonadTraced<M> for TracedF<M> {
    fn trace<A>(m: M, wa: &Box<dyn Fn(M) -> A>) -> A {
        wa(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traced_trace() {
        let w: Box<dyn Fn(i32) -> String> = Box::new(|m| format!("traced_{}", m));
        assert_eq!(TracedF::<i32>::trace(5, &w), "traced_5");
    }

    #[test]
    fn traced_extract() {
        let w: Box<dyn Fn(i32) -> String> = Box::new(|m| format!("traced_{}", m));
        // i32::empty() == 0
        assert_eq!(TracedF::<i32>::extract(&w), "traced_0");
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // trace(M::empty(), wa) == extract(wa)
        #[test]
        fn traced_identity_trace(offset in any::<i16>()) {
            let w: Box<dyn Fn(i32) -> i32> = Box::new(move |m| m + offset as i32);
            let left = TracedF::<i32>::trace(i32::empty(), &w);
            let right = TracedF::<i32>::extract(&w);
            prop_assert_eq!(left, right);
        }
    }
}
