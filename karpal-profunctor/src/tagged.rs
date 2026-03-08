use crate::choice::Choice;
use crate::profunctor::{HKT2, Profunctor};

/// Marker type whose `P<A, B> = B`.
///
/// `A` is phantom — the first argument to `dimap` is ignored.
/// This profunctor "tags" a value, supporting construction but not inspection.
/// Notably, `TaggedF` is `Choice` but NOT `Strong`, which enforces at the type
/// level that write-only optics (Review) cannot be used for reading.
pub struct TaggedF;

impl HKT2 for TaggedF {
    type P<A, B> = B;
}

impl Profunctor for TaggedF {
    fn dimap<A: 'static, B: 'static, C, D>(
        _f: impl Fn(C) -> A + 'static,
        g: impl Fn(B) -> D + 'static,
        pab: B,
    ) -> D {
        g(pab)
    }
}

// TaggedF is NOT Strong — this is deliberate.
// Strong requires `first: P<A,B> -> P<(A,C),(B,C)>` which would be `(A,C) -> (B,C)`
// but we only have a `B`, not a way to produce `C`. More importantly,
// NOT implementing Strong enforces that Tagged-based optics (Review) are write-only.

impl Choice for TaggedF {
    fn left<A, B, C>(pab: B) -> Result<B, C>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Ok(pab)
    }

    fn right<A, B, C>(pab: B) -> Result<C, B>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Err(pab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn tagged_dimap_ignores_f() {
        // f is completely ignored
        let result = <TaggedF as Profunctor>::dimap(
            |_: &str| 42i32, // f: ignored
            |b: i32| b * 2,  // g: applied
            10i32,           // pab = B = 10
        );
        assert_eq!(result, 20);
    }

    #[test]
    fn tagged_choice_left() {
        let result = <TaggedF as Choice>::left::<(), i32, &str>(42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn tagged_choice_right() {
        let result = <TaggedF as Choice>::right::<(), i32, &str>(42);
        assert_eq!(result, Err(42));
    }

    #[test]
    fn tagged_phantom_a_verification() {
        // A can be anything, it's never used
        let _: <TaggedF as HKT2>::P<String, i32> = 42;
        let _: <TaggedF as HKT2>::P<Vec<u8>, i32> = 42;
        // Both produce the same type: i32
    }

    proptest! {
        #[test]
        fn tagged_profunctor_identity(x in any::<i32>()) {
            let result = <TaggedF as Profunctor>::dimap(|a: i32| a, |b: i32| b, x);
            prop_assert_eq!(result, x);
        }
    }
}
