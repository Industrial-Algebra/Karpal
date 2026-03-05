use crate::hkt::HKT;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::StoreF;

/// ComonadStore: a comonad with a notion of position and peeking.
///
/// Laws:
/// - `peek(pos(wa), wa) == extract(wa)` (peeking at current position is extract)
pub trait ComonadStore<S>: HKT {
    fn pos<A>(wa: &Self::Of<A>) -> S;
    fn peek<A>(s: S, wa: &Self::Of<A>) -> A;

    /// Extract the focused value (equivalent to `peek(pos(wa), wa)`).
    fn extract<A>(wa: &Self::Of<A>) -> A
    where
        S: Clone,
    {
        Self::peek(Self::pos(wa), wa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<S: Clone + 'static> ComonadStore<S> for StoreF<S> {
    fn pos<A>(wa: &(Box<dyn Fn(S) -> A>, S)) -> S {
        wa.1.clone()
    }

    fn peek<A>(s: S, wa: &(Box<dyn Fn(S) -> A>, S)) -> A {
        (wa.0)(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_pos() {
        let w: (Box<dyn Fn(i32) -> String>, i32) = (Box::new(|s| format!("value_{}", s)), 42);
        assert_eq!(StoreF::<i32>::pos(&w), 42);
    }

    #[test]
    fn store_peek() {
        let w: (Box<dyn Fn(i32) -> String>, i32) = (Box::new(|s| format!("value_{}", s)), 42);
        assert_eq!(StoreF::<i32>::peek(10, &w), "value_10");
    }

    #[test]
    fn store_extract() {
        let w: (Box<dyn Fn(i32) -> String>, i32) = (Box::new(|s| format!("value_{}", s)), 42);
        assert_eq!(StoreF::<i32>::extract(&w), "value_42");
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // peek(pos(wa), wa) == extract(wa)
        #[test]
        fn store_peek_pos_is_extract(s in any::<i16>()) {
            let w: (Box<dyn Fn(i16) -> i32>, i16) =
                (Box::new(|s| s as i32 * 2), s);
            let pos = StoreF::<i16>::pos(&w);
            let left = StoreF::<i16>::peek(pos, &w);
            let right = StoreF::<i16>::extract(&w);
            prop_assert_eq!(left, right);
        }
    }
}
