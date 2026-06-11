//! Enriched categories: categories whose hom-objects carry algebraic structure
//! from a monoidal base category V.
//!
//! - Enriched over **Set**: ordinary category (hom-sets)
//! - Enriched over **Monoid**: effect-tracking categories
//! - Enriched over **Lattice**: subtyping lattice categories

/// A category enriched over a monoidal category V.
///
/// Instead of hom-sets, an enriched category has hom-objects `Hom<A, B>`
/// that live in V. Composition is a morphism in V:
/// `comp: Hom<A, B> ⊗ Hom<B, C> → Hom<A, C>`
///
/// Identity is a morphism in V:
/// `id_A: I → Hom<A, A>`
///
/// # Type parameters
///
/// - `V` — the monoidal base category (not expressed as a trait bound —
///   the enrichment structure is provided by the implementor)
/// - `Hom<A, B>` — the hom-object in V between objects A and B
pub trait EnrichedCategory<V> {
    /// The hom-object in V from A to B.
    type Hom<A, B>;

    /// Composition morphism in V: `Hom<A, B> ⊗ Hom<B, C> → Hom<A, C>`
    fn compose<A: 'static, B: 'static, C: 'static>(
        f: Self::Hom<A, B>,
        g: Self::Hom<B, C>,
    ) -> Self::Hom<A, C>;

    /// Identity morphism in V: `I → Hom<A, A>`
    fn id<A: 'static>() -> Self::Hom<A, A>;
}

// ---------------------------------------------------------------------------
// Enriched over Set (ordinary categories)
// ---------------------------------------------------------------------------

/// Marker: enrichment over Set (the cartesian monoidal category).
pub struct SetEnrichment;

/// An ordinary category, enriched over Set.
/// `Hom<A,B>` is `Box<dyn Fn(A) -> B>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct SetCategory;

#[cfg(any(feature = "std", feature = "alloc"))]
impl EnrichedCategory<SetEnrichment> for SetCategory {
    type Hom<A, B> = Box<dyn Fn(A) -> B>;

    fn compose<A: 'static, B: 'static, C: 'static>(
        f: Self::Hom<A, B>,
        g: Self::Hom<B, C>,
    ) -> Self::Hom<A, C> {
        Box::new(move |a| g(f(a)))
    }

    fn id<A: 'static>() -> Self::Hom<A, A> {
        Box::new(|a| a)
    }
}

// ---------------------------------------------------------------------------
// Enriched over Monoid (effect-tracking categories)
// ---------------------------------------------------------------------------

/// Marker: enrichment over the monoidal category of monoids.
pub struct MonoidEnrichment;

/// A category enriched over monoids.
/// `Hom<A,B>` carries a monoid structure under composition.
pub struct MonoidCategory;

impl EnrichedCategory<MonoidEnrichment> for MonoidCategory {
    type Hom<A, B> = ();

    fn compose<A: 'static, B: 'static, C: 'static>(
        _f: Self::Hom<A, B>,
        _g: Self::Hom<B, C>,
    ) -> Self::Hom<A, C> {
    }

    fn id<A: 'static>() -> Self::Hom<A, A> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_enriched_compose_chains_functions() {
        let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
        let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let gf = SetCategory::compose(f, g);
        assert_eq!(gf(5), 12); // (5+1)*2
    }

    #[test]
    fn set_enriched_id_is_identity() {
        let id = SetCategory::id::<i32>();
        assert_eq!(id(42), 42);
    }
}
