use crate::optic::Optic;

/// A write-only optic that can construct a target from a value.
///
/// This is the "review" (construction) component of a prism,
/// without any matching capability.
pub struct Review<T, B> {
    build: fn(B) -> T,
}

impl<T, B> Optic for Review<T, B> {}

impl<T, B> Review<T, B> {
    pub fn new(build: fn(B) -> T) -> Self {
        Self { build }
    }

    pub fn review(&self, b: B) -> T {
        (self.build)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Shape {
        Circle(f64),
    }

    #[test]
    fn review_build() {
        let review = Review::new(Shape::Circle);
        assert_eq!(review.review(5.0), Shape::Circle(5.0));
    }

    #[test]
    fn review_from_prism() {
        use crate::prism::Prism;
        let prism = Prism::new(
            |s: Shape| match s {
                Shape::Circle(r) => Ok(r),
            },
            Shape::Circle,
        );
        let review = prism.to_review();
        assert_eq!(review.review(10.0), Shape::Circle(10.0));
    }

    #[test]
    fn review_from_iso() {
        use crate::iso::Iso;
        let iso = Iso::new(|n: &i32| *n as f64, |f: f64| f as i32);
        let review = iso.to_review();
        assert_eq!(review.review(3.14), 3);
    }
}
