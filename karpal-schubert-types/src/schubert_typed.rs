use crate::schubert_type::SchubertType;

/// Associate a Schubert class with a Rust type.
///
/// Implementors declare which Schubert class in which Grassmannian
/// represents their type. This is the entry point for the Schubert
/// intersection type system: two types are compatible when their
/// Schubert classes intersect nontrivially.
pub trait SchubertTyped {
    /// The Schubert class associated with this type.
    fn schubert_type() -> SchubertType;
}

// Blanket impl for reference types — delegates to the inner type.
impl<T: SchubertTyped> SchubertTyped for &T {
    fn schubert_type() -> SchubertType {
        T::schubert_type()
    }
}
