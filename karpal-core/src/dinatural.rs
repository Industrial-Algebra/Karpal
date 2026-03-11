use crate::hkt::HKT2;

/// A dinatural transformation between two profunctors P and Q.
///
/// Components: `α_A: P<A, A> -> Q<A, A>` for each type A,
/// satisfying the dinaturality (hexagon) condition:
///
/// ```text
/// For any f: A -> B:
///   Q::dimap(id, f, α_A(P::dimap(f, id, p))) == Q::dimap(f, id, α_B(P::dimap(id, f, p)))
/// ```
///
/// This follows the same pattern as `NaturalTransformation<F, G>`:
/// a marker type witnesses the transformation between two profunctors.
pub trait DinaturalTransformation<P: HKT2, Q: HKT2> {
    fn transform<A: 'static>(paa: P::P<A, A>) -> Q::P<A, A>;
}

/// Identity dinatural transformation: P -> P (the identity at each component).
pub struct DinaturalId;

impl<P: HKT2> DinaturalTransformation<P, P> for DinaturalId {
    fn transform<A: 'static>(paa: P::P<A, A>) -> P::P<A, A> {
        paa
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hkt::{ResultBF, TupleF};

    #[test]
    fn dinatural_id_tuple() {
        let val: (i32, i32) = (1, 2);
        let result =
            <DinaturalId as DinaturalTransformation<TupleF, TupleF>>::transform::<i32>(val);
        assert_eq!(result, (1, 2));
    }

    #[test]
    fn dinatural_id_result() {
        let val: Result<i32, i32> = Ok(42);
        let result =
            <DinaturalId as DinaturalTransformation<ResultBF, ResultBF>>::transform::<i32>(val);
        assert_eq!(result, Ok(42));

        let val_err: Result<i32, i32> = Err(7);
        let result_err =
            <DinaturalId as DinaturalTransformation<ResultBF, ResultBF>>::transform::<i32>(val_err);
        assert_eq!(result_err, Err(7));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::hkt::TupleF;
    use proptest::prelude::*;

    proptest! {
        // Identity dinatural transformation preserves values
        #[test]
        fn dinatural_id_preserves(a in any::<i32>(), b in any::<i32>()) {
            let val: (i32, i32) = (a, b);
            let result = <DinaturalId as DinaturalTransformation<TupleF, TupleF>>::transform::<i32>(val);
            prop_assert_eq!(result, (a, b));
        }
    }
}
