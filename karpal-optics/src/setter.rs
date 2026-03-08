use crate::optic::Optic;

/// A modify-only optic. Always uses boxed closures since setters are
/// typically derived from composition or conversion.
///
/// `S` — source, `T` — modified source, `A` — focus, `B` — replacement.
pub struct Setter<S, T, A, B> {
    #[allow(clippy::type_complexity)]
    modify: Box<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}

/// A simple (monomorphic) setter where `S == T` and `A == B`.
pub type SimpleSetter<S, A> = Setter<S, S, A, A>;

impl<S, T, A, B> Optic for Setter<S, T, A, B> {}

impl<S, T, A, B> Setter<S, T, A, B> {
    pub fn new(modify: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static) -> Self {
        Self {
            modify: Box::new(modify),
        }
    }

    /// Modify the focus using a function.
    pub fn over(&self, s: S, f: impl Fn(A) -> B) -> T {
        (self.modify)(s, &f)
    }

    /// Set the focus to a constant value.
    pub fn set(&self, s: S, b: B) -> T
    where
        B: Clone,
    {
        (self.modify)(s, &|_| b.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    fn point_x_setter() -> SimpleSetter<Point, i32> {
        Setter::new(|p: Point, f: &dyn Fn(i32) -> i32| Point { x: f(p.x), ..p })
    }

    #[test]
    fn setter_over() {
        let setter = point_x_setter();
        let p = Point { x: 1, y: 2 };
        let result = setter.over(p, |x| x + 10);
        assert_eq!(result, Point { x: 11, y: 2 });
    }

    #[test]
    fn setter_set() {
        let setter = point_x_setter();
        let p = Point { x: 1, y: 2 };
        let result = setter.set(p, 99);
        assert_eq!(result, Point { x: 99, y: 2 });
    }

    #[test]
    fn setter_identity_law() {
        // over(s, id) == s
        let setter = point_x_setter();
        let p = Point { x: 5, y: 10 };
        let result = setter.over(p.clone(), |x| x);
        assert_eq!(result, p);
    }

    #[test]
    fn setter_from_lens() {
        use crate::lens::Lens;
        let lens = Lens::new(|p: &Point| p.x, |p: Point, x| Point { x, ..p });
        let setter = lens.to_setter();
        let p = Point { x: 1, y: 2 };
        let result = setter.over(p, |x| x + 10);
        assert_eq!(result, Point { x: 11, y: 2 });
    }

    #[test]
    fn setter_from_prism() {
        use crate::prism::Prism;

        #[derive(Debug, Clone, PartialEq)]
        enum Val {
            Int(i32),
            Str(String),
        }
        let prism = Prism::new(
            |v: Val| match v {
                Val::Int(n) => Ok(n),
                Val::Str(s) => Err(Val::Str(s)),
            },
            Val::Int,
        );
        let setter = prism.to_setter();
        assert_eq!(setter.over(Val::Int(5), |x| x * 2), Val::Int(10));
        assert_eq!(
            setter.over(Val::Str("hi".into()), |x| x * 2),
            Val::Str("hi".into())
        );
    }
}
