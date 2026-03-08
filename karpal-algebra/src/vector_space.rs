use crate::field::Field;
use crate::module::Module;

/// A `Module` over a `Field` — a vector space.
pub trait VectorSpace<F: Field>: Module<F> {}

impl VectorSpace<f32> for f32 {}
impl VectorSpace<f64> for f64 {}

impl<F: Field + crate::abelian::AbelianGroup> VectorSpace<F> for (F, F) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Module;
    use crate::semiring::Semiring;
    use karpal_core::Semigroup;

    #[test]
    fn f64_is_vector_space() {
        fn use_vs<V: VectorSpace<f64>>(v: V, s: f64) -> V {
            v.scale(s)
        }
        assert!((use_vs(3.0f64, 2.0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn tuple_is_vector_space() {
        fn add_scaled<V: VectorSpace<f64> + Semigroup>(a: V, b: V, s: f64) -> V {
            a.combine(b.scale(s))
        }
        let result = add_scaled((1.0f64, 0.0), (0.0, 1.0f64), 2.0);
        assert!((result.0 - 1.0).abs() < 1e-10);
        assert!((result.1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn tuple_linear_combination() {
        let e1 = (1.0f64, 0.0);
        let e2 = (0.0f64, 1.0);
        let v = e1.scale(3.0).combine(e2.scale(4.0));
        assert!((v.0 - 3.0).abs() < 1e-10);
        assert!((v.1 - 4.0).abs() < 1e-10);
    }

    #[test]
    fn scalar_field_is_one_dimensional() {
        // Every field is a vector space over itself
        let v: f64 = 5.0;
        let scaled = v.scale(f64::one());
        assert!((scaled - 5.0).abs() < 1e-10);
    }
}
