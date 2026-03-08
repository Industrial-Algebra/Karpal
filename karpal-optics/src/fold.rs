use crate::optic::Optic;
use karpal_core::Monoid;

/// A read-only multi-focus optic.
///
/// Like a `Traversal` but without the ability to modify.
pub struct Fold<S, A> {
    #[allow(clippy::type_complexity)]
    fold_fn: Box<dyn Fn(&S) -> Vec<A>>,
}

impl<S, A> Optic for Fold<S, A> {}

impl<S, A> Fold<S, A> {
    pub fn new(fold_fn: impl Fn(&S) -> Vec<A> + 'static) -> Self {
        Self {
            fold_fn: Box::new(fold_fn),
        }
    }

    pub fn get_all(&self, s: &S) -> Vec<A> {
        (self.fold_fn)(s)
    }

    /// Map each focus to a monoid value and combine them.
    pub fn fold_map<R: Monoid>(&self, s: &S, f: impl Fn(A) -> R) -> R {
        (self.fold_fn)(s)
            .into_iter()
            .map(&f)
            .fold(R::empty(), |acc, r| acc.combine(r))
    }

    /// Check if any focus satisfies a predicate.
    pub fn any(&self, s: &S, f: impl Fn(&A) -> bool) -> bool {
        (self.fold_fn)(s).iter().any(&f)
    }

    /// Check if all foci satisfy a predicate.
    pub fn all(&self, s: &S, f: impl Fn(&A) -> bool) -> bool {
        (self.fold_fn)(s).iter().all(&f)
    }

    /// Find the first focus satisfying a predicate.
    pub fn find(&self, s: &S, f: impl Fn(&A) -> bool) -> Option<A> {
        (self.fold_fn)(s).into_iter().find(|a| f(a))
    }

    /// Count the number of foci.
    pub fn length(&self, s: &S) -> usize {
        (self.fold_fn)(s).len()
    }

    /// Compose with another fold for deeper read-only access.
    pub fn then<B>(self, inner: Fold<A, B>) -> ComposedFold<S, B>
    where
        S: 'static,
        A: 'static,
        B: 'static,
    {
        let outer_fn = self.fold_fn;
        let inner_fn = inner.fold_fn;
        ComposedFold {
            fold_fn: Box::new(move |s| {
                outer_fn(s).into_iter().flat_map(|a| inner_fn(&a)).collect()
            }),
        }
    }
}

/// A composed fold using boxed closures.
pub struct ComposedFold<S, A> {
    #[allow(clippy::type_complexity)]
    fold_fn: Box<dyn Fn(&S) -> Vec<A>>,
}

impl<S, A> Optic for ComposedFold<S, A> {}

impl<S, A> ComposedFold<S, A> {
    pub fn get_all(&self, s: &S) -> Vec<A> {
        (self.fold_fn)(s)
    }

    pub fn fold_map<R: Monoid>(&self, s: &S, f: impl Fn(A) -> R) -> R {
        (self.fold_fn)(s)
            .into_iter()
            .map(&f)
            .fold(R::empty(), |acc, r| acc.combine(r))
    }

    pub fn length(&self, s: &S) -> usize {
        (self.fold_fn)(s).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_fold() -> Fold<Vec<i32>, i32> {
        Fold::new(|v: &Vec<i32>| v.clone())
    }

    #[test]
    fn fold_get_all() {
        let fold = vec_fold();
        assert_eq!(fold.get_all(&vec![1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn fold_map_sum() {
        let fold = vec_fold();
        let sum: i32 = fold.fold_map(&vec![1, 2, 3], |x| x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn fold_map_string() {
        let fold = vec_fold();
        let result: String = fold.fold_map(&vec![1, 2, 3], |x| x.to_string());
        assert_eq!(result, "123");
    }

    #[test]
    fn fold_any() {
        let fold = vec_fold();
        assert!(fold.any(&vec![1, 2, 3], |x| *x > 2));
        assert!(!fold.any(&vec![1, 2, 3], |x| *x > 5));
    }

    #[test]
    fn fold_all() {
        let fold = vec_fold();
        assert!(fold.all(&vec![1, 2, 3], |x| *x > 0));
        assert!(!fold.all(&vec![1, 2, 3], |x| *x > 1));
    }

    #[test]
    fn fold_find() {
        let fold = vec_fold();
        assert_eq!(fold.find(&vec![1, 2, 3], |x| *x > 1), Some(2));
        assert_eq!(fold.find(&vec![1, 2, 3], |x| *x > 5), None);
    }

    #[test]
    fn fold_length() {
        let fold = vec_fold();
        assert_eq!(fold.length(&vec![1, 2, 3]), 3);
        assert_eq!(fold.length(&Vec::<i32>::new()), 0);
    }

    #[test]
    fn fold_from_traversal() {
        use crate::traversal::Traversal;
        let trav = Traversal::new(
            |v: &Vec<i32>| v.clone(),
            |v: Vec<i32>, f: &dyn Fn(i32) -> i32| v.into_iter().map(f).collect::<Vec<_>>(),
        );
        let fold = trav.to_fold();
        assert_eq!(fold.get_all(&vec![1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn fold_from_lens() {
        use crate::lens::Lens;

        #[derive(Clone)]
        struct Point {
            x: i32,
        }
        let lens = Lens::new(|p: &Point| p.x, |_p: Point, x| Point { x });
        let fold = lens.to_fold();
        assert_eq!(fold.get_all(&Point { x: 42 }), vec![42]);
    }
}
