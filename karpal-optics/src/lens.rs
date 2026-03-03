use crate::optic::Optic;
use karpal_profunctor::strong::Strong;

/// A van Laarhoven–style lens encoded with getter/setter function pointers.
///
/// `S` — source type, `T` — modified source type,
/// `A` — focus type, `B` — replacement type.
///
/// For simple (non-polymorphic) lenses, use [`SimpleLens`].
pub struct Lens<S, T, A, B> {
    getter: fn(&S) -> A,
    setter: fn(S, B) -> T,
}

/// A simple (monomorphic) lens where `S == T` and `A == B`.
pub type SimpleLens<S, A> = Lens<S, S, A, A>;

impl<S, T, A, B> Optic for Lens<S, T, A, B> {}

impl<S, T, A, B> Lens<S, T, A, B> {
    pub fn new(getter: fn(&S) -> A, setter: fn(S, B) -> T) -> Self {
        Self { getter, setter }
    }

    pub fn get(&self, s: &S) -> A {
        (self.getter)(s)
    }

    pub fn set(&self, s: S, b: B) -> T {
        (self.setter)(s, b)
    }
}

impl<S: Clone, T, A, B> Lens<S, T, A, B> {
    pub fn over(&self, s: S, f: impl FnOnce(A) -> B) -> T {
        let a = (self.getter)(&s);
        (self.setter)(s, f(a))
    }

    /// Profunctor encoding: transform a `P<A, B>` into a `P<S, T>` via this lens.
    ///
    /// This is the key operation that connects concrete lenses to the profunctor
    /// hierarchy. Given any `Strong` profunctor `P` and a value `pab: P<A, B>`,
    /// `transform` produces `P<S, T>` by:
    ///
    /// 1. `first(pab)` lifts to `P<(A, S), (B, S)>`
    /// 2. `dimap` pre-composes with `s -> (get(s), s)` and post-composes with `(b, s) -> set(s, b)`
    pub fn transform<P: Strong>(&self, pab: P::P<A, B>) -> P::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        let getter = self.getter;
        let setter = self.setter;
        let first_pab = P::first::<A, B, S>(pab);
        P::dimap(
            move |s: S| {
                let a = getter(&s);
                (a, s)
            },
            move |(b, s)| setter(s, b),
            first_pab,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_profunctor::FnP;

    #[derive(Debug, Clone, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    fn person_name_lens() -> SimpleLens<Person, String> {
        Lens::new(|p: &Person| p.name.clone(), |p, name| Person { name, ..p })
    }

    fn person_age_lens() -> SimpleLens<Person, u32> {
        Lens::new(|p: &Person| p.age, |p, age| Person { age, ..p })
    }

    fn sample_person() -> Person {
        Person {
            name: "Alice".to_string(),
            age: 30,
        }
    }

    #[test]
    fn lens_get() {
        let lens = person_name_lens();
        assert_eq!(lens.get(&sample_person()), "Alice");
    }

    #[test]
    fn lens_set() {
        let lens = person_name_lens();
        let updated = lens.set(sample_person(), "Bob".to_string());
        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.age, 30);
    }

    #[test]
    fn lens_over() {
        let lens = person_age_lens();
        let updated = lens.over(sample_person(), |age| age + 1);
        assert_eq!(updated.age, 31);
        assert_eq!(updated.name, "Alice");
    }

    // Lens laws
    // GetPut: set(s, get(s)) == s
    #[test]
    fn law_get_put() {
        let lens = person_name_lens();
        let p = sample_person();
        let result = lens.set(p.clone(), lens.get(&p));
        assert_eq!(result, p);
    }

    // PutGet: get(set(s, b)) == b
    #[test]
    fn law_put_get() {
        let lens = person_name_lens();
        let result = lens.set(sample_person(), "Bob".to_string());
        assert_eq!(lens.get(&result), "Bob");
    }

    // PutPut: set(set(s, b1), b2) == set(s, b2)
    #[test]
    fn law_put_put() {
        let lens = person_name_lens();
        let p = sample_person();
        let left = lens.set(
            lens.set(p.clone(), "Bob".to_string()),
            "Charlie".to_string(),
        );
        let right = lens.set(p, "Charlie".to_string());
        assert_eq!(left, right);
    }

    // Integration test: run lens through FnP profunctor
    #[test]
    fn lens_transform_fnp() {
        let lens = person_age_lens();
        let increment: Box<dyn Fn(u32) -> u32> = Box::new(|age| age + 1);
        let transform_fn = lens.transform::<FnP>(increment);
        let result = transform_fn(sample_person());
        assert_eq!(result.age, 31);
        assert_eq!(result.name, "Alice");
    }

    #[test]
    fn lens_transform_fnp_name() {
        let lens = person_name_lens();
        let upper: Box<dyn Fn(String) -> String> = Box::new(|s| s.to_uppercase());
        let transform_fn = lens.transform::<FnP>(upper);
        let result = transform_fn(sample_person());
        assert_eq!(result.name, "ALICE");
        assert_eq!(result.age, 30);
    }
}
