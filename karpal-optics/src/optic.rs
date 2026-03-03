/// Marker trait for all optics.
///
/// This trait exists to unify the optic family under a single taxonomy.
/// Concrete optic types (Lens, Prism, etc.) implement this trait.
pub trait Optic {}
