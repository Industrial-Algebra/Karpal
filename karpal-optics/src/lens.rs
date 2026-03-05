use std::rc::Rc;

use crate::optic::Optic;
use karpal_profunctor::strong::Strong;

/// A composed lens built from two lenses chained together.
///
/// Unlike [`Lens`], which stores `fn` pointers, a composed lens stores
/// boxed closures because closure composition cannot produce `fn` pointers.
///
/// For profunctor-level composition, use nested [`Lens::transform`] calls
/// instead: `outer.transform::<P>(inner.transform::<P>(pab))`. This avoids
/// the need for `Rc`/`Arc` to share closures.
pub struct ComposedLens<S, T, X, Y> {
    getter: Box<dyn Fn(&S) -> X>,
    setter: Box<dyn Fn(S, Y) -> T>,
}

/// A simple (monomorphic) composed lens where `S == T` and `X == Y`.
pub type SimpleComposedLens<S, X> = ComposedLens<S, S, X, X>;

impl<S, T, X, Y> Optic for ComposedLens<S, T, X, Y> {}

impl<S, T, X, Y> ComposedLens<S, T, X, Y> {
    pub fn get(&self, s: &S) -> X {
        (self.getter)(s)
    }

    pub fn set(&self, s: S, y: Y) -> T {
        (self.setter)(s, y)
    }
}

impl<S: Clone, T, X, Y> ComposedLens<S, T, X, Y> {
    pub fn over(&self, s: S, f: impl FnOnce(X) -> Y) -> T {
        let x = (self.getter)(&s);
        (self.setter)(s, f(x))
    }

    /// Chain another lens to focus deeper.
    pub fn then<U, V>(self, inner: Lens<X, Y, U, V>) -> ComposedLens<S, T, U, V>
    where
        S: 'static,
        T: 'static,
        X: 'static,
        Y: 'static,
        U: 'static,
        V: 'static,
    {
        let outer_getter: Rc<dyn Fn(&S) -> X> = self.getter.into();
        let outer_setter = self.setter;
        let inner_getter = inner.getter;
        let inner_setter = inner.setter;
        let og = Rc::clone(&outer_getter);
        ComposedLens {
            getter: Box::new(move |s: &S| inner_getter(&outer_getter(s))),
            setter: Box::new(move |s: S, v: V| {
                let x = og(&s);
                let y = inner_setter(x, v);
                (outer_setter)(s, y)
            }),
        }
    }
}

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

    /// Chain another lens to focus deeper, producing a [`ComposedLens`].
    pub fn then<X, Y>(self, inner: Lens<A, B, X, Y>) -> ComposedLens<S, T, X, Y>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
        X: 'static,
        Y: 'static,
    {
        let outer_getter = self.getter;
        let outer_setter = self.setter;
        let inner_getter = inner.getter;
        let inner_setter = inner.setter;
        ComposedLens {
            getter: Box::new(move |s: &S| inner_getter(&outer_getter(s))),
            setter: Box::new(move |s: S, y: Y| {
                let a = outer_getter(&s);
                let b = inner_setter(a, y);
                outer_setter(s, b)
            }),
        }
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

    // --- Composition tests ---

    #[derive(Debug, Clone, PartialEq)]
    struct Address {
        street: String,
        city: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Company {
        name: String,
        ceo: Person,
    }

    fn company_ceo_lens() -> SimpleLens<Company, Person> {
        Lens::new(|c: &Company| c.ceo.clone(), |c, ceo| Company { ceo, ..c })
    }

    fn address_city_lens() -> SimpleLens<Address, String> {
        Lens::new(
            |a: &Address| a.city.clone(),
            |a, city| Address { city, ..a },
        )
    }

    fn address_street_lens() -> SimpleLens<Address, String> {
        Lens::new(
            |a: &Address| a.street.clone(),
            |a, street| Address { street, ..a },
        )
    }

    fn sample_company() -> Company {
        Company {
            name: "Acme".to_string(),
            ceo: sample_person(),
        }
    }

    // Two-deep composition: Company → ceo → name
    #[test]
    fn composed_get() {
        let lens = company_ceo_lens().then(person_name_lens());
        assert_eq!(lens.get(&sample_company()), "Alice");
    }

    #[test]
    fn composed_set() {
        let lens = company_ceo_lens().then(person_name_lens());
        let updated = lens.set(sample_company(), "Bob".to_string());
        assert_eq!(updated.ceo.name, "Bob");
        assert_eq!(updated.ceo.age, 30);
        assert_eq!(updated.name, "Acme");
    }

    #[test]
    fn composed_over() {
        let lens = company_ceo_lens().then(person_age_lens());
        let updated = lens.over(sample_company(), |age| age + 1);
        assert_eq!(updated.ceo.age, 31);
        assert_eq!(updated.ceo.name, "Alice");
    }

    // Three-deep: use a PersonWithAddr for a longer chain.

    #[derive(Debug, Clone, PartialEq)]
    struct PersonWithAddr {
        name: String,
        addr: Address,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Org {
        title: String,
        lead: PersonWithAddr,
    }

    fn org_lead_lens() -> SimpleLens<Org, PersonWithAddr> {
        Lens::new(|o: &Org| o.lead.clone(), |o, lead| Org { lead, ..o })
    }

    fn pwa_addr_lens() -> SimpleLens<PersonWithAddr, Address> {
        Lens::new(
            |p: &PersonWithAddr| p.addr.clone(),
            |p, addr| PersonWithAddr { addr, ..p },
        )
    }

    fn sample_org() -> Org {
        Org {
            title: "R&D".to_string(),
            lead: PersonWithAddr {
                name: "Alice".to_string(),
                addr: Address {
                    street: "123 Main St".to_string(),
                    city: "Springfield".to_string(),
                },
            },
        }
    }

    #[test]
    fn three_deep_get() {
        let lens = org_lead_lens()
            .then(pwa_addr_lens())
            .then(address_city_lens());
        assert_eq!(lens.get(&sample_org()), "Springfield");
    }

    #[test]
    fn three_deep_set() {
        let lens = org_lead_lens()
            .then(pwa_addr_lens())
            .then(address_city_lens());
        let updated = lens.set(sample_org(), "Shelbyville".to_string());
        assert_eq!(updated.lead.addr.city, "Shelbyville");
        assert_eq!(updated.lead.addr.street, "123 Main St");
        assert_eq!(updated.lead.name, "Alice");
    }

    #[test]
    fn three_deep_over() {
        let lens = org_lead_lens()
            .then(pwa_addr_lens())
            .then(address_street_lens());
        let updated = lens.over(sample_org(), |s| s.to_uppercase());
        assert_eq!(updated.lead.addr.street, "123 MAIN ST");
    }

    // Law tests on composed lens
    #[test]
    fn composed_law_get_put() {
        let lens = company_ceo_lens().then(person_name_lens());
        let c = sample_company();
        let result = lens.set(c.clone(), lens.get(&c));
        assert_eq!(result, c);
    }

    #[test]
    fn composed_law_put_get() {
        let lens = company_ceo_lens().then(person_name_lens());
        let result = lens.set(sample_company(), "Bob".to_string());
        assert_eq!(lens.get(&result), "Bob");
    }

    #[test]
    fn composed_law_put_put() {
        let lens = company_ceo_lens().then(person_name_lens());
        let c = sample_company();
        let left = lens.set(
            lens.set(c.clone(), "Bob".to_string()),
            "Charlie".to_string(),
        );
        let right = lens.set(c, "Charlie".to_string());
        assert_eq!(left, right);
    }

    // Profunctor equivalence: outer.transform(inner.transform(f)) matches composed.over
    #[test]
    fn profunctor_composition_equivalence() {
        let outer = company_ceo_lens();
        let inner = person_age_lens();
        let composed = company_ceo_lens().then(person_age_lens());

        let increment: Box<dyn Fn(u32) -> u32> = Box::new(|age| age + 1);
        let transform_fn = outer.transform::<FnP>(inner.transform::<FnP>(increment));

        let c = sample_company();
        let via_profunctor = transform_fn(c.clone());
        let via_composed = composed.over(c, |age| age + 1);
        assert_eq!(via_profunctor, via_composed);
    }
}
