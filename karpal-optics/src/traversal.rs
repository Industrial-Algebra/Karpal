use crate::optic::Optic;
use karpal_profunctor::Traversing;
use std::rc::Rc;

/// A multi-focus optic that can get/modify zero or more foci.
///
/// `S` — source, `T` — modified source, `A` — focus, `B` — replacement.
pub struct Traversal<S, T, A, B> {
    #[allow(clippy::type_complexity)]
    get_all: Rc<dyn Fn(&S) -> Vec<A>>,
    #[allow(clippy::type_complexity)]
    modify_all: Rc<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}

/// A simple (monomorphic) traversal where `S == T` and `A == B`.
pub type SimpleTraversal<S, A> = Traversal<S, S, A, A>;

impl<S, T, A, B> Optic for Traversal<S, T, A, B> {}

impl<S, T, A, B> Traversal<S, T, A, B> {
    pub fn new(
        get_all: impl Fn(&S) -> Vec<A> + 'static,
        modify_all: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static,
    ) -> Self {
        Self {
            get_all: Rc::new(get_all),
            modify_all: Rc::new(modify_all),
        }
    }

    pub fn get_all(&self, s: &S) -> Vec<A> {
        (self.get_all)(s)
    }

    pub fn over(&self, s: S, f: impl Fn(A) -> B) -> T {
        (self.modify_all)(s, &f)
    }

    pub fn set(&self, s: S, b: B) -> T
    where
        B: Clone,
    {
        (self.modify_all)(s, &|_| b.clone())
    }

    /// Profunctor encoding via `Traversing::wander`.
    pub fn transform<P: Traversing>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let get_all = Rc::clone(&self.get_all);
        let modify_all = Rc::clone(&self.modify_all);
        P::wander(move |s| get_all(s), move |s, f| modify_all(s, f), pab)
    }

    /// Convert to a `Fold` (read-only).
    pub fn to_fold(&self) -> crate::fold::Fold<S, A>
    where
        S: 'static,
        A: 'static,
    {
        let get_all = Rc::clone(&self.get_all);
        crate::fold::Fold::new(move |s| get_all(s))
    }

    /// Compose with another traversal for deeper multi-focus access.
    pub fn then<X, Y>(self, inner: Traversal<A, B, X, Y>) -> ComposedTraversal<S, T, X, Y>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
        X: 'static,
        Y: 'static,
    {
        let outer_get_all = self.get_all;
        let inner_get_all = Rc::clone(&inner.get_all);
        let outer_modify_all = self.modify_all;
        let inner_modify_all = inner.modify_all;
        ComposedTraversal {
            get_all: Box::new(move |s| {
                outer_get_all(s)
                    .into_iter()
                    .flat_map(|a| inner_get_all(&a))
                    .collect()
            }),
            modify_all: Box::new(move |s, f| outer_modify_all(s, &|a| (inner_modify_all)(a, f))),
        }
    }
}

/// A composed traversal using boxed closures.
pub struct ComposedTraversal<S, T, A, B> {
    #[allow(clippy::type_complexity)]
    get_all: Box<dyn Fn(&S) -> Vec<A>>,
    #[allow(clippy::type_complexity)]
    modify_all: Box<dyn Fn(S, &dyn Fn(A) -> B) -> T>,
}

impl<S, T, A, B> Optic for ComposedTraversal<S, T, A, B> {}

impl<S, T, A, B> ComposedTraversal<S, T, A, B> {
    pub fn get_all(&self, s: &S) -> Vec<A> {
        (self.get_all)(s)
    }

    pub fn over(&self, s: S, f: impl Fn(A) -> B) -> T {
        (self.modify_all)(s, &f)
    }

    pub fn set(&self, s: S, b: B) -> T
    where
        B: Clone,
    {
        (self.modify_all)(s, &|_| b.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_profunctor::{FnP, ForgetF};

    fn vec_each_traversal() -> SimpleTraversal<Vec<i32>, i32> {
        Traversal::new(
            |v: &Vec<i32>| v.clone(),
            |v: Vec<i32>, f: &dyn Fn(i32) -> i32| v.into_iter().map(f).collect(),
        )
    }

    #[test]
    fn traversal_get_all() {
        let trav = vec_each_traversal();
        assert_eq!(trav.get_all(&vec![1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn traversal_over() {
        let trav = vec_each_traversal();
        assert_eq!(trav.over(vec![1, 2, 3], |x| x * 10), vec![10, 20, 30]);
    }

    #[test]
    fn traversal_set() {
        let trav = vec_each_traversal();
        assert_eq!(trav.set(vec![1, 2, 3], 0), vec![0, 0, 0]);
    }

    #[test]
    fn traversal_transform_fnp() {
        let trav = vec_each_traversal();
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = trav.transform::<FnP>(double);
        assert_eq!(f(vec![1, 2, 3]), vec![2, 4, 6]);
    }

    #[test]
    fn traversal_transform_forget() {
        let trav = vec_each_traversal();
        let to_string: Box<dyn Fn(i32) -> String> = Box::new(|x| x.to_string());
        let f = trav.transform::<ForgetF<String>>(to_string);
        // ForgetF with String Monoid concatenates
        assert_eq!(f(vec![1, 2, 3]), "123");
    }

    #[test]
    fn traversal_identity_law() {
        let trav = vec_each_traversal();
        let v = vec![1, 2, 3];
        assert_eq!(trav.over(v.clone(), |x| x), v);
    }

    // Composition: traverse into nested vecs
    #[test]
    fn traversal_composition() {
        let outer: SimpleTraversal<Vec<Vec<i32>>, Vec<i32>> = Traversal::new(
            |v: &Vec<Vec<i32>>| v.clone(),
            |v: Vec<Vec<i32>>, f: &dyn Fn(Vec<i32>) -> Vec<i32>| {
                v.into_iter().map(f).collect::<Vec<_>>()
            },
        );
        let inner = vec_each_traversal();
        let composed = outer.then(inner);
        assert_eq!(composed.get_all(&vec![vec![1, 2], vec![3]]), vec![1, 2, 3]);
        assert_eq!(
            composed.over(vec![vec![1, 2], vec![3]], |x| x * 10),
            vec![vec![10, 20], vec![30]]
        );
    }

    #[test]
    fn traversal_from_lens() {
        use crate::lens::Lens;

        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let lens = Lens::new(|p: &Point| p.x, |p: Point, x| Point { x, ..p });
        let trav = lens.to_traversal();
        let p = Point { x: 1, y: 2 };
        assert_eq!(trav.get_all(&p), vec![1]);
        assert_eq!(trav.over(p, |x| x + 10), Point { x: 11, y: 2 });
    }

    #[test]
    fn traversal_from_prism() {
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
        let trav = prism.to_traversal();
        assert_eq!(trav.get_all(&Val::Int(5)), vec![5]);
        assert_eq!(trav.get_all(&Val::Str("hi".into())), Vec::<i32>::new());
        assert_eq!(trav.over(Val::Int(5), |x| x * 2), Val::Int(10));
        assert_eq!(
            trav.over(Val::Str("hi".into()), |x| x * 2),
            Val::Str("hi".into())
        );
    }

    #[test]
    fn traversal_composition_law() {
        // over(over(s, f), g) == over(s, g . f)
        let trav = vec_each_traversal();
        let v = vec![1, 2, 3];
        let left = trav.over(trav.over(v.clone(), |x| x + 1), |x| x * 2);
        let right = trav.over(v, |x| (x + 1) * 2);
        assert_eq!(left, right);
    }
}
