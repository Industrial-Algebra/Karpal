use crate::fold::Fold;
use crate::optic::Optic;
use crate::review::Review;
use crate::setter::Setter;
use crate::traversal::Traversal;
use karpal_profunctor::choice::Choice;

/// A prism focuses on one variant of a sum type.
///
/// `S` — source type, `T` — modified source type,
/// `A` — focus type (the variant's inner value), `B` — replacement type.
///
/// Where a [`Lens`](crate::Lens) uses [`Strong`](karpal_profunctor::Strong) to
/// decompose products, a Prism uses [`Choice`] to decompose coproducts.
///
/// For simple (monomorphic) prisms, use [`SimplePrism`].
pub struct Prism<S, T, A, B> {
    /// Attempt to match. `Ok(a)` = matched, `Err(t)` = didn't match (pass-through).
    match_: fn(S) -> Result<A, T>,
    /// Construct a `T` from the replacement value.
    build: fn(B) -> T,
}

/// A simple (monomorphic) prism where `S == T` and `A == B`.
pub type SimplePrism<S, A> = Prism<S, S, A, A>;

impl<S, T, A, B> Optic for Prism<S, T, A, B> {}

impl<S, T, A, B> Prism<S, T, A, B> {
    pub fn new(match_: fn(S) -> Result<A, T>, build: fn(B) -> T) -> Self {
        Self { match_, build }
    }

    /// Try to extract the focus. Returns `Some(a)` if the variant matches.
    pub fn preview(&self, s: &S) -> Option<A>
    where
        S: Clone,
    {
        (self.match_)(s.clone()).ok()
    }

    /// Construct a `T` from a replacement value (inject/construct).
    pub fn review(&self, b: B) -> T {
        (self.build)(b)
    }

    /// Replace the focus if the variant matches; otherwise pass through.
    pub fn set(&self, s: S, b: B) -> T {
        match (self.match_)(s) {
            Ok(_) => (self.build)(b),
            Err(t) => t,
        }
    }

    /// Modify the focus if the variant matches; otherwise pass through.
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T {
        match (self.match_)(s) {
            Ok(a) => (self.build)(f(a)),
            Err(t) => t,
        }
    }

    /// Convert to a `Review` (write-only, construction).
    pub fn to_review(&self) -> Review<T, B> {
        Review::new(self.build)
    }

    /// Convert to a `Setter` (modify-only).
    pub fn to_setter(&self) -> Setter<S, T, A, B>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let match_ = self.match_;
        let build = self.build;
        Setter::new(move |s: S, f: &dyn Fn(A) -> B| match match_(s) {
            Ok(a) => build(f(a)),
            Err(t) => t,
        })
    }

    /// Convert to a `Traversal` (0-or-1 element focus).
    pub fn to_traversal(&self) -> Traversal<S, T, A, B>
    where
        S: Clone + 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let match_ = self.match_;
        let build = self.build;
        Traversal::new(
            move |s: &S| match match_(s.clone()) {
                Ok(a) => vec![a],
                Err(_) => vec![],
            },
            move |s: S, f: &dyn Fn(A) -> B| match match_(s) {
                Ok(a) => build(f(a)),
                Err(t) => t,
            },
        )
    }

    /// Convert to a `Fold` (0-or-1 element, read-only).
    pub fn to_fold(&self) -> Fold<S, A>
    where
        S: Clone + 'static,
        T: 'static,
        A: 'static,
    {
        let match_ = self.match_;
        Fold::new(move |s: &S| match match_(s.clone()) {
            Ok(a) => vec![a],
            Err(_) => vec![],
        })
    }

    /// Profunctor encoding: transform a `P<A, B>` into a `P<S, T>` via this prism.
    ///
    /// This connects prisms to the profunctor hierarchy through [`Choice`].
    /// Given any `Choice` profunctor `P` and a value `pab: P<A, B>`,
    /// `transform` produces `P<S, T>` by:
    ///
    /// 1. `right(pab)` lifts to `P<Result<T, A>, Result<T, B>>`
    /// 2. `dimap` pre-composes with `match_` (swapping arms) and post-composes
    ///    with `build` (reassembling)
    ///
    /// The arm-swapping (`Ok→Err`, `Err→Ok` in the pre-composition) is necessary
    /// because `Choice::right` acts on the `Err` branch of `Result`.
    pub fn transform<P: Choice>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let match_ = self.match_;
        let build = self.build;
        let right_pab = P::right::<A, B, T>(pab);
        P::dimap(
            move |s: S| match match_(s) {
                Ok(a) => Err(a), // focus found → Err arm for Choice::right
                Err(t) => Ok(t), // no match → Ok arm passes through
            },
            move |result: Result<T, B>| match result {
                Ok(t) => t,         // passed through unchanged
                Err(b) => build(b), // transformed, rebuild
            },
            right_pab,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_profunctor::FnP;
    use proptest::prelude::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Shape {
        Circle(f64),
        Rectangle(f64, f64),
    }

    fn circle_prism() -> SimplePrism<Shape, f64> {
        Prism::new(
            |s| match s {
                Shape::Circle(r) => Ok(r),
                Shape::Rectangle(w, h) => Err(Shape::Rectangle(w, h)),
            },
            Shape::Circle,
        )
    }

    fn sample_circle() -> Shape {
        Shape::Circle(5.0)
    }

    fn sample_rect() -> Shape {
        Shape::Rectangle(3.0, 4.0)
    }

    // --- Unit tests ---

    #[test]
    fn preview_match() {
        let prism = circle_prism();
        assert_eq!(prism.preview(&sample_circle()), Some(5.0));
    }

    #[test]
    fn preview_no_match() {
        let prism = circle_prism();
        assert_eq!(prism.preview(&sample_rect()), None);
    }

    #[test]
    fn review() {
        let prism = circle_prism();
        assert_eq!(prism.review(10.0), Shape::Circle(10.0));
    }

    #[test]
    fn set_match() {
        let prism = circle_prism();
        assert_eq!(prism.set(sample_circle(), 10.0), Shape::Circle(10.0));
    }

    #[test]
    fn set_no_match() {
        let prism = circle_prism();
        assert_eq!(prism.set(sample_rect(), 10.0), sample_rect());
    }

    #[test]
    fn over_match() {
        let prism = circle_prism();
        assert_eq!(
            prism.over(sample_circle(), |r| r * 2.0),
            Shape::Circle(10.0)
        );
    }

    #[test]
    fn over_no_match() {
        let prism = circle_prism();
        assert_eq!(prism.over(sample_rect(), |r| r * 2.0), sample_rect());
    }

    // --- Prism laws (proptest) ---

    // Use bounded, finite f64 to avoid NaN/infinity
    fn finite_f64() -> impl Strategy<Value = f64> {
        (-1e6f64..1e6f64).prop_filter("finite", |v| v.is_finite())
    }

    // ReviewPreview: preview(review(b)) == Some(b)
    proptest! {
        #[test]
        fn law_review_preview(b in finite_f64()) {
            let prism = circle_prism();
            let s = prism.review(b);
            prop_assert_eq!(prism.preview(&s), Some(b));
        }
    }

    // PreviewReview: if preview(s) == Some(a) then review(a) == s
    proptest! {
        #[test]
        fn law_preview_review(r in finite_f64()) {
            let prism = circle_prism();
            let s = Shape::Circle(r);
            if let Some(a) = prism.preview(&s) {
                prop_assert_eq!(prism.review(a), s);
            }
        }
    }

    // OverIdentity: over(s, id) == s
    proptest! {
        #[test]
        fn law_over_identity_circle(r in finite_f64()) {
            let prism = circle_prism();
            let s = Shape::Circle(r);
            prop_assert_eq!(prism.over(s.clone(), |x| x), s);
        }

        #[test]
        fn law_over_identity_rect(w in finite_f64(), h in finite_f64()) {
            let prism = circle_prism();
            let s = Shape::Rectangle(w, h);
            prop_assert_eq!(prism.over(s.clone(), |x| x), s);
        }
    }

    // --- FnP integration ---

    #[test]
    fn transform_fnp_match() {
        let prism = circle_prism();
        let double: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 2.0);
        let transform_fn = prism.transform::<FnP>(double);
        assert_eq!(transform_fn(sample_circle()), Shape::Circle(10.0));
    }

    #[test]
    fn transform_fnp_no_match() {
        let prism = circle_prism();
        let double: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 2.0);
        let transform_fn = prism.transform::<FnP>(double);
        assert_eq!(transform_fn(sample_rect()), sample_rect());
    }
}
