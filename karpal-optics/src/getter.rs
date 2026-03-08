use crate::optic::Optic;

/// A read-only optic that extracts a single focus from a source.
///
/// This is the "getter" component of a lens, without any modification capability.
pub struct Getter<S, A> {
    get: fn(&S) -> A,
}

impl<S, A> Optic for Getter<S, A> {}

impl<S, A> Getter<S, A> {
    pub fn new(get: fn(&S) -> A) -> Self {
        Self { get }
    }

    pub fn get(&self, s: &S) -> A {
        (self.get)(s)
    }

    /// Compose with another getter for deeper access.
    pub fn then<B>(self, inner: Getter<A, B>) -> ComposedGetter<S, B>
    where
        S: 'static,
        A: 'static,
        B: 'static,
    {
        let outer_get = self.get;
        let inner_get = inner.get;
        ComposedGetter {
            get: Box::new(move |s| inner_get(&outer_get(s))),
        }
    }
}

/// A composed getter using boxed closures (from composition or conversion).
pub struct ComposedGetter<S, A> {
    get: Box<dyn Fn(&S) -> A>,
}

impl<S, A> Optic for ComposedGetter<S, A> {}

impl<S, A> ComposedGetter<S, A> {
    pub fn get(&self, s: &S) -> A {
        (self.get)(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[test]
    fn getter_get() {
        let getter = Getter::new(|p: &Point| p.x);
        let p = Point { x: 1.0, y: 2.0 };
        assert!((getter.get(&p) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn getter_composition() {
        #[derive(Clone)]
        struct Line {
            start: Point,
        }
        let line_start = Getter::new(|l: &Line| l.start.clone());
        let point_x = Getter::new(|p: &Point| p.x);
        let composed = line_start.then(point_x);
        let line = Line {
            start: Point { x: 3.0, y: 4.0 },
        };
        assert!((composed.get(&line) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn getter_from_lens() {
        use crate::lens::Lens;
        let lens = Lens::new(|p: &Point| p.x, |p: Point, x| Point { x, ..p });
        let getter = lens.to_getter();
        let p = Point { x: 5.0, y: 6.0 };
        assert!((getter.get(&p) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn getter_from_iso() {
        use crate::iso::Iso;
        let iso = Iso::new(|n: &i32| *n as f64, |f: f64| f as i32);
        let getter = iso.to_getter();
        assert!((getter.get(&42) - 42.0).abs() < f64::EPSILON);
    }
}
