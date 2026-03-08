use alloc::vec::Vec;

use crate::choice::Choice;
use crate::strong::Strong;

/// A `Traversing` profunctor can lift a `P<A, B>` to operate over multiple foci.
///
/// This is the key ingredient for profunctor-encoded traversals.
///
/// `wander` takes both a `get_all` (for read-only profunctors like `ForgetF`)
/// and a `modify_all` (for read-write profunctors like `FnP`), since Rust
/// lacks rank-2 types that would allow a single polymorphic traversal function.
pub trait Traversing: Strong + Choice {
    fn wander<S, T, A, B>(
        get_all: impl Fn(&S) -> Vec<A> + 'static,
        modify_all: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static;
}
