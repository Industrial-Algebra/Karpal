/// Type aliases documenting the various algebra and coalgebra forms
/// used in recursion schemes.
///
/// These are provided for documentation and pedagogy. The actual scheme
/// functions accept `impl Fn(...)` parameters directly, which is more
/// ergonomic than working with trait objects.
///
/// # Algebra family (folds)
///
/// - **Algebra**: `F<A> -> A` — basic fold (catamorphism)
/// - **RAlgebra**: `F<(Fix<F>, A)> -> A` — fold with original subterms (paramorphism)
/// - **CVAlgebra**: `F<Cofree<F, A>> -> A` — fold with history (histomorphism)
///
/// # Coalgebra family (unfolds)
///
/// - **Coalgebra**: `A -> F<A>` — basic unfold (anamorphism)
/// - **RCoalgebra**: `A -> F<Either<Fix<F>, A>>` — unfold with early stop (apomorphism)
/// - **CVCoalgebra**: `A -> F<Free<F, A>>` — multi-step unfold (futumorphism)
///
/// # Composite
///
/// - **Chronomorphism**: CVCoalgebra + CVAlgebra (futu ; histo)
/// - **Zygomorphism**: Algebra + auxiliary Algebra (fold with helper)
use karpal_core::hkt::HKT;

use crate::either::Either;
use crate::fix::Fix;

#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

/// `F<A> -> A` — an F-algebra, used in catamorphism.
pub type Algebra<F, A> = Box<dyn Fn(<F as HKT>::Of<A>) -> A>;

/// `A -> F<A>` — an F-coalgebra, used in anamorphism.
pub type Coalgebra<F, A> = Box<dyn Fn(A) -> <F as HKT>::Of<A>>;

/// `F<(Fix<F>, A)> -> A` — an R-algebra, used in paramorphism.
pub type RAlgebra<F, A> = Box<dyn Fn(<F as HKT>::Of<(Fix<F>, A)>) -> A>;

/// `A -> F<Either<Fix<F>, A>>` — an R-coalgebra, used in apomorphism.
pub type RCoalgebra<F, A> = Box<dyn Fn(A) -> <F as HKT>::Of<Either<Fix<F>, A>>>;
