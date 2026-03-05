use crate::contravariant::{Contravariant, PredicateF};

/// Decide: the contravariant analogue of Alt.
///
/// Given a way to split `C` into either `A` or `B`, and contravariant
/// functors over `A` and `B`, produce a contravariant functor over `C`.
///
/// Laws:
/// - Associativity: `choose(f, choose(g, a, b), c) == choose(h, a, choose(i, b, c))`
///   (where f/g/h/i are appropriate splitting functions)
pub trait Decide: Contravariant {
    fn choose<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> Result<A, B> + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}

impl Decide for PredicateF {
    fn choose<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> Result<A, B> + 'static,
        fa: Box<dyn Fn(A) -> bool>,
        fb: Box<dyn Fn(B) -> bool>,
    ) -> Box<dyn Fn(C) -> bool> {
        Box::new(move |c| match f(c) {
            Ok(a) => fa(a),
            Err(b) => fb(b),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_choose() {
        let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
        let is_short: Box<dyn Fn(String) -> bool> = Box::new(|s| s.len() < 5);

        // Classify input: integers go left, strings go right
        let classifier =
            PredicateF::choose(|input: Result<i32, String>| input, is_positive, is_short);

        assert!(classifier(Ok(5)));
        assert!(!classifier(Ok(-1)));
        assert!(classifier(Err("hi".to_string())));
        assert!(!classifier(Err("hello world".to_string())));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Associativity
        #[test]
        fn predicate_associativity(x in any::<i8>(), y in any::<i8>(), z in any::<i8>()) {
            let pa: Box<dyn Fn(i8) -> bool> = Box::new(|a| a > 0);
            let pb: Box<dyn Fn(i8) -> bool> = Box::new(|b| b > 0);
            let pc: Box<dyn Fn(i8) -> bool> = Box::new(|c| c > 0);

            let pa2: Box<dyn Fn(i8) -> bool> = Box::new(|a| a > 0);
            let pb2: Box<dyn Fn(i8) -> bool> = Box::new(|b| b > 0);
            let pc2: Box<dyn Fn(i8) -> bool> = Box::new(|c| c > 0);

            // choose(id, choose(id, a, b), c) on a tagged union
            // Tag: 0 = a, 1 = b, 2 = c
            let ab = PredicateF::choose(|v: Result<i8, i8>| v, pa, pb);
            let left = PredicateF::choose(
                |tag: (u8, i8)| {
                    if tag.0 < 2 {
                        Ok(if tag.0 == 0 { Ok(tag.1) } else { Err(tag.1) })
                    } else {
                        Err(tag.1)
                    }
                },
                ab,
                pc,
            );

            let bc = PredicateF::choose(|v: Result<i8, i8>| v, pb2, pc2);
            let right = PredicateF::choose(
                |tag: (u8, i8)| {
                    if tag.0 == 0 {
                        Ok(tag.1)
                    } else {
                        Err(if tag.0 == 1 { Ok(tag.1) } else { Err(tag.1) })
                    }
                },
                pa2,
                bc,
            );

            // Test all three tags
            prop_assert_eq!(left((0, x)), right((0, x)));
            prop_assert_eq!(left((1, y)), right((1, y)));
            prop_assert_eq!(left((2, z)), right((2, z)));
        }
    }
}
