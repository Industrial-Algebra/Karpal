use core::marker::PhantomData;

/// Witness that justification `Via` validates rewriting `Lhs` to `Rhs`.
///
/// Types implementing this trait declare which algebraic identities
/// are valid rewrite steps.
pub trait Justifies<Lhs, Rhs> {}

/// Witness that type-level expression `Lhs` equals `Rhs`,
/// justified by rule `Via`.
///
/// `Rewrite` is a zero-sized type — it carries no data, only
/// type-level evidence that an algebraic identity has been invoked.
///
/// # Example
///
/// ```
/// use karpal_proof::rewrite::*;
///
/// // Create a rewrite justified by associativity
/// let step: Rewrite<AssocLeft, AssocRight, ByAssociativity> =
///     Rewrite::witness();
///
/// // Reverse it
/// let _back: Rewrite<AssocRight, AssocLeft, BySymmetry<ByAssociativity>> =
///     step.sym();
/// ```
pub struct Rewrite<Lhs, Rhs, Via> {
    _phantom: PhantomData<(Lhs, Rhs, Via)>,
}

impl<Lhs, Rhs, Via> Rewrite<Lhs, Rhs, Via> {
    /// Construct a rewrite witness.
    ///
    /// Only compiles if `Via: Justifies<Lhs, Rhs>`.
    pub fn witness() -> Self
    where
        Via: Justifies<Lhs, Rhs>,
    {
        Rewrite {
            _phantom: PhantomData,
        }
    }

    /// Reverse: if `Lhs = Rhs` then `Rhs = Lhs`.
    pub fn sym(self) -> Rewrite<Rhs, Lhs, BySymmetry<Via>> {
        Rewrite {
            _phantom: PhantomData,
        }
    }

    /// Chain: if `Lhs = Mid` (via self) and `Mid = Rhs2` (via next),
    /// then `Lhs = Rhs2`.
    ///
    /// The `Rhs` of the first rewrite must be the same type as the
    /// `Lhs` (i.e., `Mid`) of the second. This is enforced by sharing
    /// the type parameter `Rhs`/`Mid`.
    pub fn then<Rhs2, V2>(
        self,
        _next: Rewrite<Rhs, Rhs2, V2>,
    ) -> Rewrite<Lhs, Rhs2, ByTransitivity<Via, V2, Rhs>> {
        Rewrite {
            _phantom: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Justification types (all ZSTs)
// ---------------------------------------------------------------------------

/// Justified by associativity: `a ∘ (b ∘ c) = (a ∘ b) ∘ c`.
pub struct ByAssociativity;

/// Justified by commutativity: `a ∘ b = b ∘ a`.
pub struct ByCommutativity;

/// Justified by identity law: `a ∘ e = a` or `e ∘ a = a`.
pub struct ByIdentity;

/// Justified by inverse law: `a ∘ a⁻¹ = e`.
pub struct ByInverse;

/// Justified by distribution: `a * (b + c) = a*b + a*c`.
pub struct ByDistribution;

/// Justified by zero annihilation: `0 * a = 0`.
pub struct ByAnnihilation;

/// Chain two justifications: if `Lhs = Mid` via `V1` and `Mid = Rhs` via `V2`.
/// `Mid` is carried as a type parameter to satisfy Rust's coherence rules.
pub struct ByTransitivity<V1, V2, Mid = ()>(PhantomData<(V1, V2, Mid)>);

/// Reverse a justification: if `Lhs = Rhs` via `V`, then `Rhs = Lhs`.
pub struct BySymmetry<V>(PhantomData<V>);

// ---------------------------------------------------------------------------
// Expression marker types for common algebraic patterns
// ---------------------------------------------------------------------------

/// `(a ∘ b) ∘ c` — left-associated.
pub struct AssocLeft;
/// `a ∘ (b ∘ c)` — right-associated.
pub struct AssocRight;

/// `a ∘ b`.
pub struct CombineAB;
/// `b ∘ a`.
pub struct CombineBA;

/// `a ∘ e` or `e ∘ a`.
pub struct WithIdentity;
/// Just `a`.
pub struct JustA;

/// `a ∘ a⁻¹`.
pub struct WithInverse;
/// The identity element `e`.
pub struct Identity;

/// `a * (b + c)` — undistributed.
pub struct Undistributed;
/// `a*b + a*c` — distributed.
pub struct Distributed;

/// `0 * a`.
pub struct ZeroTimes;
/// `0`.
pub struct Zero;

// ---------------------------------------------------------------------------
// Justifies implementations
// ---------------------------------------------------------------------------

// Associativity: (a∘b)∘c = a∘(b∘c) and reverse
impl Justifies<AssocLeft, AssocRight> for ByAssociativity {}
impl Justifies<AssocRight, AssocLeft> for ByAssociativity {}

// Commutativity: a∘b = b∘a and reverse
impl Justifies<CombineAB, CombineBA> for ByCommutativity {}
impl Justifies<CombineBA, CombineAB> for ByCommutativity {}

// Identity: a∘e = a and reverse
impl Justifies<WithIdentity, JustA> for ByIdentity {}
impl Justifies<JustA, WithIdentity> for ByIdentity {}

// Inverse: a∘a⁻¹ = e and reverse
impl Justifies<WithInverse, Identity> for ByInverse {}
impl Justifies<Identity, WithInverse> for ByInverse {}

// Distribution: a*(b+c) = a*b + a*c and reverse
impl Justifies<Undistributed, Distributed> for ByDistribution {}
impl Justifies<Distributed, Undistributed> for ByDistribution {}

// Zero annihilation: 0*a = 0 and reverse
impl Justifies<ZeroTimes, Zero> for ByAnnihilation {}
impl Justifies<Zero, ZeroTimes> for ByAnnihilation {}

// Symmetry: if V justifies Lhs→Rhs, BySymmetry<V> justifies Rhs→Lhs
impl<V, Lhs, Rhs> Justifies<Rhs, Lhs> for BySymmetry<V> where V: Justifies<Lhs, Rhs> {}

// Transitivity: if V1 justifies Lhs→Mid and V2 justifies Mid→Rhs,
// ByTransitivity<V1, V2, Mid> justifies Lhs→Rhs
impl<V1, V2, Lhs, Mid, Rhs> Justifies<Lhs, Rhs> for ByTransitivity<V1, V2, Mid>
where
    V1: Justifies<Lhs, Mid>,
    V2: Justifies<Mid, Rhs>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associativity_rewrite() {
        let _: Rewrite<AssocLeft, AssocRight, ByAssociativity> = Rewrite::witness();
        let _: Rewrite<AssocRight, AssocLeft, ByAssociativity> = Rewrite::witness();
    }

    #[test]
    fn commutativity_rewrite() {
        let _: Rewrite<CombineAB, CombineBA, ByCommutativity> = Rewrite::witness();
    }

    #[test]
    fn identity_rewrite() {
        let _: Rewrite<WithIdentity, JustA, ByIdentity> = Rewrite::witness();
        let _: Rewrite<JustA, WithIdentity, ByIdentity> = Rewrite::witness();
    }

    #[test]
    fn inverse_rewrite() {
        let _: Rewrite<WithInverse, Identity, ByInverse> = Rewrite::witness();
    }

    #[test]
    fn distribution_rewrite() {
        let _: Rewrite<Undistributed, Distributed, ByDistribution> = Rewrite::witness();
    }

    #[test]
    fn annihilation_rewrite() {
        let _: Rewrite<ZeroTimes, Zero, ByAnnihilation> = Rewrite::witness();
    }

    #[test]
    fn symmetry() {
        let step: Rewrite<AssocLeft, AssocRight, ByAssociativity> = Rewrite::witness();
        let _back: Rewrite<AssocRight, AssocLeft, BySymmetry<ByAssociativity>> = step.sym();
    }

    #[test]
    fn transitivity_chain() {
        // (a∘b)∘c → a∘(b∘c) → (b∘c)∘a  [via assoc then commutativity]
        // We model this as: AssocLeft → AssocRight → CombineBA
        // But we need CombineBA to match the second step...
        // Instead demonstrate: WithIdentity → JustA → WithIdentity
        let step1: Rewrite<WithIdentity, JustA, ByIdentity> = Rewrite::witness();
        let step2: Rewrite<JustA, WithIdentity, ByIdentity> = Rewrite::witness();
        let _chained: Rewrite<
            WithIdentity,
            WithIdentity,
            ByTransitivity<ByIdentity, ByIdentity, JustA>,
        > = step1.then(step2);
    }

    #[test]
    fn three_step_chain() {
        // WithInverse → Identity → WithInverse → Identity
        let s1: Rewrite<WithInverse, Identity, ByInverse> = Rewrite::witness();
        let s2: Rewrite<Identity, WithInverse, ByInverse> = Rewrite::witness();
        let s3: Rewrite<WithInverse, Identity, ByInverse> = Rewrite::witness();
        let _: Rewrite<
            WithInverse,
            Identity,
            ByTransitivity<ByTransitivity<ByInverse, ByInverse, Identity>, ByInverse, WithInverse>,
        > = s1.then(s2).then(s3);
    }
}
